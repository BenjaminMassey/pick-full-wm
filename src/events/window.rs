use x11rb::CURRENT_TIME;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::{
    Allow, CONFIGURE_NOTIFY_EVENT, ChangeWindowAttributesAux, ConfigWindow, ConfigureNotifyEvent,
    ConfigureRequestEvent, ConfigureWindowAux, ConnectionExt, DestroyNotifyEvent, EventMask,
    MapRequestEvent, UnmapNotifyEvent,
};

pub fn map_request(state: &mut crate::state::State, event: MapRequestEvent) {
    if state.all_windows.contains(&event.window) {
        // app hid the window (unmap) and is now showing it again
        let mut slot: Option<(usize, usize)> = None;
        for (monitor_index, monitor) in state.monitors.iter().enumerate() {
            for (workspace_index, workspace) in monitor.workspaces.iter().enumerate() {
                if workspace.main_area.contains(&event.window)
                    || workspace.side_index(event.window).is_some()
                {
                    slot = Some((monitor_index, workspace_index));
                }
            }
        }
        if let Some((monitor_index, workspace_index)) = slot {
            log::info!(
                "Remapping handled window \"{:?}\" ({}).",
                crate::windows::gets::window_name(state, event.window),
                event.window
            );
            // hidden workspaces get mapped by workspaces::switch(..) later
            if workspace_index == state.current_workspace {
                if let Err(e) = state.conn.map_window(event.window) {
                    log::error!("events::map_request(..) map window error: {:?}", e);
                }
                let real_current_monitor = state.current_monitor; // TODO: gross temp set
                state.current_monitor = monitor_index; // TODO: gross temp set
                crate::windows::layout::layout_side_space(state);
                state.current_monitor = real_current_monitor; // TODO: gross temp set
            }
            return;
        }
        log::warn!(
            "Handled window \"{:?}\" ({}) lost its slot: mapping as new.",
            crate::windows::gets::window_name(state, event.window),
            event.window
        );
        state.all_windows.remove(&event.window);
    }
    log::info!(
        "Map Window: {:?} ({})",
        crate::windows::gets::window_name(state, event.window),
        event.window,
    );
    if !crate::safety::window_exists(state, event.window) {
        log::error!("Map request for non-existent window {}.", event.window);
        if let Err(e) = state.conn.allow_events(Allow::ASYNC_BOTH, CURRENT_TIME) {
            log::error!("events::map_request(..) allow events error: {:?}", e);
        }
        if let Err(e) = state.conn.flush() {
            log::error!("events::map_request(..) flush error: {:?}", e);
        }
        return;
    }
    if let Err(e) = state.conn.map_window(event.window) {
        log::error!("events::map_request(..) map window error: {:?}", e);
    }
    if let Some(key) = crate::windows::gets::key_hint_window(state, event.window) {
        log::info!("Window {} should be a key hint window.", event.window);
        if let Some(entry) = state.mut_workspace().key_hint_windows.get_mut(&key) {
            let old_key = entry.clone();
            *entry = event.window;
            if old_key != event.window && crate::safety::window_exists(state, old_key) {
                if let Err(e) = state.conn.destroy_window(old_key) {
                    log::error!("events::map_request(..) destroy window error: {:?}", e);
                }
                if let Err(e) = state.conn.flush() {
                    log::error!("events::map_request(..) flush error: {:?}", e);
                }
            }
        } else {
            state
                .mut_workspace()
                .key_hint_windows
                .insert(key, event.window);
        }
        crate::windows::layout::layout_side_space(state);
        return;
    }
    if crate::windows::checks::is_help_window(state, event.window) {
        log::info!("Window {} should be a help window.", event.window);
        crate::ewmh::set_active(state, event.window);
        if let Err(e) = state.conn.flush() {
            log::error!("events::map_request(..) flush error: {:?}", e);
        }
        state.mut_workspace().help_window = Some(event.window);
        return;
    }
    if crate::windows::checks::is_close_box(state, event.window) {
        log::info!("Window {} should be a close box.", event.window);
        for i in 0..state.monitors.len() {
            if state.monitors[i].close_box.is_none() {
                log::info!("connecting close box {} to monitor {}", event.window, i);
                state.monitors[i].close_box = Some(event.window);
                break;
            }
        }
        crate::windows::layout::place_close_boxes(state);
        return;
    }
    if crate::windows::checks::is_pop_box(state, event.window) {
        log::info!("Window {} should be a pop box.", event.window);
        for i in 0..state.monitors.len() {
            if state.monitors[i].pop_box.is_none() {
                log::info!("connecting pop box {} to monitor {}", event.window, i);
                state.monitors[i].pop_box = Some(event.window);
                break;
            }
        }
        crate::windows::layout::place_pop_boxes(state);
        return;
    }
    if crate::windows::checks::is_split_box(state, event.window) {
        log::info!("Window {} should be a split box.", event.window);
        // goes to whichever monitor spawned one (see windows::audits::split_boxes(..))
        let monitor_index = state
            .monitors
            .iter()
            .position(|m| m.split_boxes_pending > 0)
            .unwrap_or(state.current_monitor);
        let monitor = &mut state.monitors[monitor_index];
        monitor.split_boxes_pending = monitor.split_boxes_pending.saturating_sub(1);
        monitor.split_boxes.push(event.window);
        let real_current_monitor = state.current_monitor; // TODO: gross temp set
        state.current_monitor = monitor_index; // TODO: gross temp set
        crate::windows::layout::layout_side_space(state);
        state.current_monitor = real_current_monitor; // TODO: gross temp set
        return;
    }
    if crate::windows::checks::is_monitor_box(state, event.window) {
        log::info!("Window {} should be a monitor box.", event.window);
        for i in 0..state.monitors.len() {
            if state.monitors[i].monitor_box.is_none() {
                log::info!("connecting monitor box {} to monitor {}", event.window, i);
                state.monitors[i].monitor_box = Some(event.window);
                break;
            }
        }
        crate::windows::layout::place_monitor_boxes(state);
        crate::windows::layout::place_pop_boxes(state);
        return;
    }
    if crate::windows::checks::is_dock(state, event.window) {
        log::info!("Window {} should be a dock.", event.window);
        if !state.docks.contains(&event.window) {
            state.docks.push(event.window);
        }
        crate::windows::layout::raise_docks(state);
        return;
    }
    if crate::windows::checks::is_excepted_window(state, event.window) {
        return;
    }
    if crate::windows::checks::is_popup(state, event.window) {
        log::info!("Window {} should be a popup.", event.window);
        // apps can send a second map request before the first takes effect
        if !state.workspace().floatings.contains(&event.window) {
            state.mut_workspace().floatings.push(event.window);
        }
        crate::windows::layout::center_window(state, event.window);
        if crate::windows::checks::takes_focus(state, event.window) {
            crate::ewmh::set_active(state, event.window);
        }
        return;
    }
    // pointer crossings for focus follows mouse (see windows::core::focus_hovered(..))
    if state.settings.bindings.focus_follows_mouse
        && let Err(e) = state.conn.change_window_attributes(
            event.window,
            &ChangeWindowAttributesAux::new().event_mask(EventMask::ENTER_WINDOW),
        )
    {
        log::error!("events::map_request(..) select input error: {:?}", e);
    }
    let main = state.workspace().main_area.clone();
    if !main.is_empty() {
        // a fullscreen main keeps the screen: new windows open on the side
        if state.settings.layout.new_to_main && !state.workspace().is_fullscreen() {
            state.mut_workspace().main_area = vec![];
            crate::windows::core::send_side_space(state, main, None);
            crate::windows::core::fill_main_space(state, vec![event.window]);
        } else {
            crate::windows::core::send_side_space(state, vec![event.window], None);
        }
    } else {
        crate::windows::core::fill_main_space(state, vec![event.window]);
    }
    crate::windows::layout::place_close_boxes(state);
    crate::windows::layout::place_monitor_boxes(state);
    crate::windows::layout::place_pop_boxes(state);
}

pub fn destroy(state: &mut crate::state::State, event: DestroyNotifyEvent) {
    log::info!(
        "Destroy Window: {:?} ({})",
        crate::windows::gets::window_name(state, event.window),
        event.window,
    );
    state.all_windows.remove(&event.window);
    crate::windows::core::release_window(state, event.window);
    crate::ewmh::restore_lost_focus(state);
}

pub fn unmap(state: &mut crate::state::State, event: UnmapNotifyEvent) {
    // the wm only unmaps windows on hidden workspaces (workspaces::switch(..)),
    // so an unmap on the visible workspace means the app hid its window
    if let Some((_, workspace_index)) = crate::windows::core::find_window(state, event.window)
        && workspace_index == state.current_workspace
    {
        log::info!("Window {} hidden by client.", event.window);
        state.all_windows.remove(&event.window);
        crate::windows::core::release_window(state, event.window);
    }
    crate::ewmh::restore_lost_focus(state);
}

pub fn configure_request(state: &mut crate::state::State, event: ConfigureRequestEvent) {
    let mut is_tiled = false;
    let mut is_floating = false;
    for monitor in &state.monitors {
        for workspace in &monitor.workspaces {
            if workspace.main_area.contains(&event.window)
                || workspace.side_index(event.window).is_some()
            {
                is_tiled = true;
            }
            if workspace.floatings.contains(&event.window) {
                is_floating = true;
            }
            // wm helpers are placed and raised by the wm, so treat them like tiled windows
            if workspace
                .key_hint_windows
                .values()
                .any(|w| *w == event.window)
            {
                is_tiled = true;
            }
        }
        if monitor.close_box == Some(event.window)
            || monitor.monitor_box == Some(event.window)
            || monitor.pop_box == Some(event.window)
            || monitor.split_boxes.contains(&event.window)
        {
            is_tiled = true;
        }
    }
    if is_floating {
        // honor size (and stacking), but keep popups centered
        let mut aux = ConfigureWindowAux::from_configure_request(&event);
        aux.x = None;
        aux.y = None;
        if let Err(e) = state.conn.configure_window(event.window, &aux) {
            log::error!(
                "events::configure_request(..) configure window error: {:?}",
                e
            );
        }
        crate::windows::layout::center_window(state, event.window);
        return;
    }
    if !is_tiled {
        // not ours to lay out (excluded, unmapped, etc): honor the request
        if event.value_mask.contains(ConfigWindow::STACK_MODE) {
            log::info!("Honored restack request from window {}.", event.window);
        }
        if let Err(e) = state.conn.configure_window(
            event.window,
            &ConfigureWindowAux::from_configure_request(&event),
        ) {
            log::error!(
                "events::configure_request(..) configure window error: {:?}",
                e
            );
        }
        if let Err(e) = state.conn.flush() {
            log::error!("events::configure_request(..) flush error: {:?}", e);
        }
        return;
    }
    // tiled: deny, but tell the client its real geometry (ICCCM 4.1.5)
    log::info!(
        "Denied configure request for managed window {}.",
        event.window
    );
    let geometry = match state.conn.get_geometry(event.window) {
        Ok(cookie) => match cookie.reply() {
            Ok(reply) => reply,
            Err(_) => return,
        },
        Err(_) => return,
    };
    let notify = ConfigureNotifyEvent {
        response_type: CONFIGURE_NOTIFY_EVENT,
        sequence: 0,
        event: event.window,
        window: event.window,
        above_sibling: x11rb::NONE,
        x: geometry.x,
        y: geometry.y,
        width: geometry.width,
        height: geometry.height,
        border_width: geometry.border_width,
        override_redirect: false,
    };
    if let Err(e) = state
        .conn
        .send_event(false, event.window, EventMask::STRUCTURE_NOTIFY, notify)
    {
        log::error!("events::configure_request(..) send event error: {:?}", e);
    }
    if let Err(e) = state.conn.flush() {
        log::error!("events::configure_request(..) flush error: {:?}", e);
    }
}
