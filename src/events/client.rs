use x11rb::CURRENT_TIME;
use x11rb::protocol::xproto::ClientMessageEvent;
use x11rb::protocol::xproto::{Allow, ConnectionExt};

pub fn message(state: &mut crate::state::State, event: ClientMessageEvent) {
    if event.type_ == state.atoms._NET_CURRENT_DESKTOP {
        // Switch to workspace
        let requested_workspace = event.data.as_data32()[0] as usize;
        if state.current_workspace != requested_workspace {
            state.current_workspace = requested_workspace;
            crate::windows::workspaces::switch(state);
        }
    } else if event.type_ == state.atoms._NET_ACTIVE_WINDOW {
        // Activate/focus a window
        let monitor_index = crate::windows::gets::monitor_index(state, event.window);
        if monitor_index != state.current_monitor {
            state.current_monitor = monitor_index;
        }
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
        if let Err(e) = state.conn.destroy_window(event.window) {
            eprintln!("events::client::message(..) destroy window error: {:?}", e);
        }
        if let Err(e) = state.conn.allow_events(Allow::ASYNC_POINTER, CURRENT_TIME) {
            eprintln!("events::client::message(..) allow events error: {:?}", e);
        }
    } else if event.type_ == state.atoms._NET_WM_DESKTOP {
        // Move window to a specific workspace
        let target_workspace = event.data.as_data32()[0] as usize;
        if !state.workspace().side_windows.is_empty()
            && let Some(new_main) = state.workspace().side_windows[0]
        {
            crate::windows::core::remove_side_window(state, new_main);
            crate::windows::core::fill_main_space(state, new_main);
        } else {
            state.mut_workspace().main_window = None;
        }
        let real_current_workspace = state.current_workspace; // TODO: gross temp set
        state.current_workspace = target_workspace; // TODO: gross temp set
        if let Some(move_aside) = state.workspace().main_window.clone() {
            crate::windows::core::send_side_space(state, move_aside, None);
        }
        crate::windows::core::fill_main_space(state, event.window);
        state.current_workspace = real_current_workspace; // TODO: gross temp set
        crate::windows::core::focus_main(state);
        crate::windows::audits::full(state);
        crate::windows::workspaces::switch(state);
    } else {
        // Unknown client message type
        println!("Unhandled ClientMessage type: {:?}", event.type_);
    }
}
