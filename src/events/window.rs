use x11rb::CURRENT_TIME;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::{
    Allow, CONFIGURE_NOTIFY_EVENT, ConfigureNotifyEvent, ConfigureRequestEvent, ConfigureWindowAux,
    ConnectionExt, DestroyNotifyEvent, EventMask, MapRequestEvent,
};

pub fn map_request(state: &mut crate::state::State, event: MapRequestEvent) {
    if state.all_windows.contains(&event.window) {
        log::warn!(
            "Skipping remap of handled window \"{:?}\" ({}).",
            crate::windows::gets::window_name(state, event.window),
            event.window
        );
        return;
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
        return;
    }
    if crate::windows::checks::is_excepted_window(state, event.window) {
        return;
    }
    if crate::windows::checks::is_popup(state, event.window) {
        log::info!("Window {} should be a popup.", event.window);
        state.mut_workspace().floatings.push(event.window);
        crate::windows::layout::center_window(state, event.window);
        return;
    }
    if let Some(main) = state.workspace().main_window {
        if state.settings.layout.new_to_main {
            crate::windows::core::send_side_space(state, main, None);
            crate::windows::core::fill_main_space(state, event.window);
        } else {
            crate::windows::core::send_side_space(state, event.window, None);
        }
    } else {
        crate::windows::core::fill_main_space(state, event.window);
    }
    crate::windows::layout::place_close_boxes(state);
    crate::windows::layout::place_monitor_boxes(state);
}

pub fn destroy(state: &mut crate::state::State, event: DestroyNotifyEvent) {
    log::info!(
        "Destroy Window: {:?} ({})",
        crate::windows::gets::window_name(state, event.window),
        event.window,
    );
    state.all_windows.remove(&event.window);
    for i in 0..state.monitor().workspaces.len() {
        if state.monitor().workspaces[i]
            .floatings
            .contains(&event.window)
        {
            crate::windows::core::remove_floating(state, event.window);
            crate::windows::core::focus_main(state);
            return;
        }
        if let Some(help) = state.monitor().workspaces[i].help_window
            && event.window == help
        {
            if let Some(main_window) = state.monitor().workspaces[i].main_window
                && state.current_workspace == i
            {
                crate::ewmh::set_active(state, main_window);
                if let Err(e) = state.conn.flush() {
                    log::error!("events::destroy(..) flush error: {:?}", e);
                }
            }
            state.mut_monitor().workspaces[i].help_window = None;
            return;
        }
        let real_workspace = state.current_workspace.clone(); // TODO: gross, for windows.rs calls
        state.current_workspace = i; // TODO: gross, for windows.rs calls
        if let Some(main_window) = state.monitor().workspaces[i].main_window {
            if event.window == main_window {
                if !state.monitor().workspaces[i].side_windows.is_empty() {
                    if let Some(target) = state.monitor().workspaces[i].side_windows[0]
                        && state.current_workspace == i
                    {
                        crate::windows::core::remove_side_window(state, target);
                        crate::windows::core::fill_main_space(state, target);
                    } else {
                        state.mut_workspace().main_window = None;
                    }
                } else {
                    state.mut_workspace().main_window = None;
                }
            } else {
                crate::windows::core::remove_side_window(state, event.window);
            }
        }
        state.current_workspace = real_workspace; // TODO: gross, for windows.rs calls
    }
    crate::windows::layout::layout_side_space(state);
    if state.workspace().main_window.is_none() {
        crate::ewmh::clear_active(state);
    }
    crate::windows::layout::place_close_boxes(state);
    crate::windows::layout::place_monitor_boxes(state);
}

pub fn configure_request(state: &mut crate::state::State, event: ConfigureRequestEvent) {
    let mut is_tiled = false;
    let mut is_floating = false;
    for monitor in &state.monitors {
        for workspace in &monitor.workspaces {
            if workspace.main_window == Some(event.window)
                || workspace.side_windows.contains(&Some(event.window))
            {
                is_tiled = true;
            }
            if workspace.floatings.contains(&event.window) {
                is_floating = true;
            }
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
        "Denied configure request for tiled window {}.",
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
