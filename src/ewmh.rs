use x11rb::connection::Connection;
use x11rb::protocol::xproto::{
    AtomEnum, ConfigureWindowAux, ConnectionExt, InputFocus, PropMode, StackMode, Window,
};
use x11rb::wrapper::ConnectionExt as WrapperConnectionExt;
use x11rb::CURRENT_TIME;

pub fn set_active(state: &mut crate::state::State, window: Window) {
    state
        .conn
        .change_property32(
            PropMode::REPLACE,
            state.root,
            state.atoms._NET_ACTIVE_WINDOW,
            AtomEnum::WINDOW,
            &[window],
        )
        .expect("Failed to set _NET_ACTIVE_WINDOW");

    if crate::safety::window_exists(state, window) {
        state
            .conn
            .set_input_focus(InputFocus::POINTER_ROOT, window, CURRENT_TIME)
            .expect("Failed to set input focus");

        state
            .conn
            .configure_window(window, &ConfigureWindowAux::new().stack_mode(StackMode::ABOVE))
            .expect("Failed to raise window");
    }

    state.conn.flush().expect("Failed to flush");
}

pub fn clear_active(state: &mut crate::state::State) {
    set_active(state, 0);
}

pub fn update_workspace(state: &crate::state::State) {
    state
        .conn
        .change_property32(
            PropMode::REPLACE,
            state.root,
            state.atoms._NET_CURRENT_DESKTOP,
            AtomEnum::CARDINAL,
            &[state.current_workspace as u32],
        )
        .expect("Failed to set _NET_CURRENT_DESKTOP");

    state.conn.flush().expect("Failed to flush");
}
