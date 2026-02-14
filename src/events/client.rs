use x11rb::protocol::xproto::ClientMessageEvent;

pub fn message(state: &mut crate::state::State, event: ClientMessageEvent) {
    if event.type_ == state.atoms._NET_CURRENT_DESKTOP {
        let requested_workspace = event.data.as_data32()[0] as usize;
        if state.current_workspace != requested_workspace {
            state.current_workspace = requested_workspace;
            crate::windows::workspaces::switch(state);
        }
    } else if event.type_ == state.atoms._NET_ACTIVE_WINDOW {
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
    }
}
