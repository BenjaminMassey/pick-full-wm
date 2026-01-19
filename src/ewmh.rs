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
        eprintln!(
            "ewmh::set_active(..) change property NET_ACTIVE_WINDOW error: {:?}",
            e
        );
    }

    if crate::safety::window_exists(state, window) {
        if let Err(e) = state
            .conn
            .set_input_focus(InputFocus::POINTER_ROOT, window, CURRENT_TIME)
        {
            eprintln!("ewmh::set_active(..) set input focus error: {:?}", e);
        }

        if let Err(e) = state.conn.configure_window(
            window,
            &ConfigureWindowAux::new().stack_mode(StackMode::ABOVE),
        ) {
            eprintln!("ewmh::set_active(..) configure window stack error: {:?}", e);
        }
    }

    if let Err(e) = state.conn.flush() {
        eprintln!("ewmh::set_active(..) flush error: {:?}", e);
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
        eprintln!(
            "ewmh::update_workspace(..) change property NET_CURRENT_DESKTOP error: {:?}",
            e
        );
    }

    if let Err(e) = state.conn.flush() {
        eprintln!("ewmh::update_workspace(..) flush error: {:?}", e);
    }
}
