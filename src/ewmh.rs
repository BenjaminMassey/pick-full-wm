use x11rb::CURRENT_TIME;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::{
    AtomEnum, ConfigureWindowAux, ConnectionExt, InputFocus, PropMode, StackMode, Window,
};
use x11rb::wrapper::ConnectionExt as WrapperConnectionExt;

pub fn set_active(state: &mut crate::state::State, window: Window) {
    if let Err(e) = state.conn.change_property32(
        PropMode::REPLACE,
        state.root,
        state.atoms._NET_ACTIVE_WINDOW,
        AtomEnum::WINDOW,
        &[window],
    ) {
        log::error!(
            "ewmh::set_active(..) change property NET_ACTIVE_WINDOW error: {:?}",
            e
        );
    }

    if crate::safety::window_exists(state, window) {
        if let Err(e) = state
            .conn
            .set_input_focus(InputFocus::POINTER_ROOT, window, CURRENT_TIME)
        {
            log::error!("ewmh::set_active(..) set input focus error: {:?}", e);
        }

        if let Err(e) = state.conn.configure_window(
            window,
            &ConfigureWindowAux::new().stack_mode(StackMode::ABOVE),
        ) {
            log::error!("ewmh::set_active(..) configure window stack error: {:?}", e);
        }
    }

    if let Err(e) = state.conn.flush() {
        log::error!("ewmh::set_active(..) flush error: {:?}", e);
    }
}

pub fn clear_active(state: &mut crate::state::State) {
    set_active(state, 0);
}

pub fn update_workspace(state: &crate::state::State) {
    if let Err(e) = state.conn.change_property32(
        PropMode::REPLACE,
        state.root,
        state.atoms._NET_CURRENT_DESKTOP,
        AtomEnum::CARDINAL,
        &[state.current_workspace as u32],
    ) {
        log::error!(
            "ewmh::update_workspace(..) change property NET_CURRENT_DESKTOP error: {:?}",
            e
        );
    }

    if let Err(e) = state.conn.flush() {
        log::error!("ewmh::update_workspace(..) flush error: {:?}", e);
    }
}

pub fn update_client_list(state: &crate::state::State) {
    let client_list: Vec<Window> = state.all_windows.iter().map(|w| w.to_owned()).collect();

    if let Err(e) = state.conn.change_property32(
        PropMode::REPLACE,
        state.root,
        state.atoms._NET_CLIENT_LIST,
        AtomEnum::WINDOW,
        &client_list,
    ) {
        log::error!(
            "ewmh::update_client_list(..) change property NET_CLIENT_LIST error: {:?}",
            e
        );
    }

    if let Err(e) = state.conn.change_property32(
        PropMode::REPLACE,
        state.root,
        state.atoms._NET_CLIENT_LIST_STACKING,
        AtomEnum::WINDOW,
        &client_list,
    ) {
        log::error!(
            "ewmh::update_client_list(..) change property NET_CLIENT_LIST_STACKING error: {:?}",
            e
        );
    }

    if let Err(e) = state.conn.flush() {
        log::error!("ewmh::update_client_list(..) flush error: {:?}", e);
    }
}

pub fn desktop_assignments(state: &crate::state::State) {
    for monitor in &state.monitors {
        for (workspace_index, workspace) in monitor.workspaces.iter().enumerate() {
            if let Some(main) = workspace.main_window {
                set_window_desktop(state, main, workspace_index);
            }
            for side_window in &workspace.side_windows {
                if let Some(side) = side_window {
                    set_window_desktop(state, *side, workspace_index);
                }
            }
            if let Some(help) = workspace.help_window {
                set_window_desktop(state, help, workspace_index);
            }
        }
    }
}

fn set_window_desktop(state: &crate::state::State, window: Window, desktop: usize) {
    if let Err(e) = state.conn.change_property32(
        PropMode::REPLACE,
        window,
        state.atoms._NET_WM_DESKTOP,
        AtomEnum::CARDINAL,
        &[desktop as u32],
    ) {
        log::error!(
            "ewmh::set_window_desktop(..) change property NET_WM_DESKTOP error: {:?}",
            e
        );
    }

    if let Err(e) = state.conn.flush() {
        log::error!("ewmh::set_window_desktop(..) flush error: {:?}", e);
    }
}
