use x11rb::protocol::xproto::{ConnectionExt, Window};

pub fn window_exists(state: &crate::state::State, window: Window) -> bool {
    if window == 0 {
        return false;
    } // TODO: is this needed and proper?

    match state.conn.get_window_attributes(window) {
        Ok(cookie) => cookie.reply().is_ok(),
        Err(_) => false,
    }
}
