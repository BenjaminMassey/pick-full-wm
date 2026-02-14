use x11rb::protocol::xproto::{AtomEnum, ConnectionExt, Window};

pub fn is_excepted_window(state: &mut crate::state::State, window: Window) -> bool {
    if is_help_window(state, window)
        || crate::windows::gets::key_hint_window(state, window).is_some()
    {
        return true;
    }
    if let Some(name) = crate::windows::gets::window_name(state, window) {
        for exception in &state.settings.applications.excluded {
            if name.contains(exception) {
                return true;
            }
        }
    }
    false
}

pub fn is_help_window(state: &mut crate::state::State, window: Window) -> bool {
    if let Some(name) = crate::windows::gets::window_name(state, window)
        && name.contains("pfwm help")
    {
        return true;
    }
    false
}

pub fn is_close_box(state: &mut crate::state::State, window: Window) -> bool {
    if let Some(name) = crate::windows::gets::window_name(state, window)
        && name.contains("pfwm close")
    {
        return true;
    }
    false
}

pub fn is_popup(state: &crate::state::State, window: Window) -> bool {
    // Check 1: Override-redirect windows (menus, tooltips)
    if let Ok(attrs) = state.conn.get_window_attributes(window) {
        if let Ok(attrs_reply) = attrs.reply() {
            if attrs_reply.override_redirect {
                return true;
            }
        }
    }

    // Check 2: Transient windows (dialogs with parent)
    if let Ok(prop) = state.conn.get_property(
        false,
        window,
        AtomEnum::WM_TRANSIENT_FOR,
        AtomEnum::WINDOW,
        0,
        1,
    ) {
        if let Ok(reply) = prop.reply() {
            if reply.value_len > 0 {
                return true;
            }
        }
    }

    // Check 3: Window type hints
    if let Ok(prop) = state.conn.get_property(
        false,
        window,
        state.atoms._NET_WM_WINDOW_TYPE,
        AtomEnum::ATOM,
        0,
        1024,
    ) {
        if let Ok(reply) = prop.reply() {
            if reply.value_len > 0 {
                // Parse as list of atoms
                let window_types: Vec<x11rb::protocol::xproto::Atom> = reply
                    .value32()
                    .map(|iter| iter.collect())
                    .unwrap_or_default();

                // Check if any type matches popup/dialog types
                for &window_type in &window_types {
                    if window_type == state.atoms._NET_WM_WINDOW_TYPE_DIALOG
                        || window_type == state.atoms._NET_WM_WINDOW_TYPE_UTILITY
                        || window_type == state.atoms._NET_WM_WINDOW_TYPE_SPLASH
                        || window_type == state.atoms._NET_WM_WINDOW_TYPE_NOTIFICATION
                        || window_type == state.atoms._NET_WM_WINDOW_TYPE_POPUP_MENU
                        || window_type == state.atoms._NET_WM_WINDOW_TYPE_DROPDOWN_MENU
                        || window_type == state.atoms._NET_WM_WINDOW_TYPE_TOOLTIP
                    {
                        return true;
                    }
                }
            }
        }
    }

    false
}
