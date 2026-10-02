use x11rb::CURRENT_TIME;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::ClientMessageEvent;
use x11rb::protocol::xproto::{Allow, ConnectionExt};

pub fn message(state: &mut crate::state::State, event: ClientMessageEvent) {
    if event.type_ == state.atoms._NET_CURRENT_DESKTOP {
        // Switch to workspace
        let requested_workspace = event.data.as_data32()[0] as usize;
        if requested_workspace >= state.monitor().workspaces.len() {
            log::warn!(
                "Ignored request for invalid workspace #{}.",
                requested_workspace
            );
            return;
        }
        if state.current_workspace != requested_workspace {
            state.current_workspace = requested_workspace;
            crate::windows::workspaces::switch(state);
        }
    } else if event.type_ == state.atoms._NET_ACTIVE_WINDOW {
        // Activate/focus a window
        let monitor_index = crate::windows::gets::monitor_index(state, event.window);
        let workspace = &state.monitors[monitor_index].workspaces[state.current_workspace];
        let is_side = workspace.side_windows.contains(&Some(event.window));
        let is_floating = workspace.floatings.contains(&event.window);
        if !is_side && !is_floating {
            // unmanaged (override-redirect menus, etc) or already main
            log::info!("Ignored activate request for window {}.", event.window);
            return;
        }
        if monitor_index != state.current_monitor {
            state.current_monitor = monitor_index;
        }
        if is_floating {
            log::info!("Activate request for floating window {}.", event.window);
            crate::ewmh::set_active(state, event.window);
            if let Err(e) = state.conn.flush() {
                log::error!("events::client::message(..) flush error: {:?}", e);
            }
            return;
        }
        log::info!("Activate request for side window {}.", event.window);
        if let Some(existing) = state.workspace().main_window {
            if existing == event.window {
                return;
            }
            let index = crate::windows::core::remove_side_window(state, event.window);
            crate::windows::core::fill_main_space(state, event.window);
            if state.settings.layout.swap_not_stack {
                crate::windows::core::send_side_space(state, existing, Some(index));
            } else {
                crate::windows::core::send_side_space(state, existing, None);
            }
        }
    } else if event.type_ == state.atoms._NET_CLOSE_WINDOW {
        // Request to close a window (from panel, pager, etc.)
        crate::windows::core::close_window(state, event.window);
        if let Err(e) = state.conn.allow_events(Allow::ASYNC_POINTER, CURRENT_TIME) {
            log::error!("events::client::message(..) allow events error: {:?}", e);
        }
    } else if event.type_ == state.atoms._NET_WM_DESKTOP {
        // Move window to a specific workspace
        let target_workspace = event.data.as_data32()[0] as usize;
        if target_workspace >= state.monitor().workspaces.len() {
            // also covers 0xFFFFFFFF ("all desktops"), which isn't supported
            log::warn!(
                "Ignored move of window {} to invalid workspace #{}.",
                event.window,
                target_workspace
            );
            return;
        }
        let monitor_index = crate::windows::gets::monitor_index(state, event.window);
        let source_workspace = state.monitors[monitor_index]
            .workspaces
            .iter()
            .position(|w| {
                w.main_window == Some(event.window) || w.side_windows.contains(&Some(event.window))
            });
        let Some(source_workspace) = source_workspace else {
            log::info!(
                "Ignored workspace move of unmanaged window {}.",
                event.window
            );
            return;
        };
        if source_workspace == target_workspace {
            return;
        }
        log::info!(
            "Moving window {} from workspace #{} to #{}.",
            event.window,
            source_workspace,
            target_workspace
        );
        let real_current_monitor = state.current_monitor; // TODO: gross temp set
        let real_current_workspace = state.current_workspace; // TODO: gross temp set
        state.current_monitor = monitor_index; // TODO: gross temp set
        state.current_workspace = source_workspace; // TODO: gross temp set
        if state.workspace().main_window == Some(event.window) {
            if !state.workspace().side_windows.is_empty()
                && let Some(new_main) = state.workspace().side_windows[0]
            {
                crate::windows::core::remove_side_window(state, new_main);
                crate::windows::core::fill_main_space(state, new_main);
            } else {
                state.mut_workspace().main_window = None;
            }
        } else {
            crate::windows::core::remove_side_window(state, event.window);
        }
        state.current_workspace = target_workspace; // TODO: gross temp set
        if let Some(move_aside) = state.workspace().main_window.clone() {
            crate::windows::core::send_side_space(state, move_aside, None);
        }
        crate::windows::core::fill_main_space(state, event.window);
        state.current_monitor = real_current_monitor; // TODO: gross temp set
        state.current_workspace = real_current_workspace; // TODO: gross temp set
        crate::windows::core::focus_main(state);
        crate::windows::audits::full(state);
        crate::windows::workspaces::switch(state);
    } else {
        // Unknown client message type
        log::info!("Unhandled ClientMessage type: {:?}", event.type_);
    }
}
