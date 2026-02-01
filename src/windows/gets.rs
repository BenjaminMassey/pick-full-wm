use x11rb::protocol::xproto::{AtomEnum, ConnectionExt, Window};

pub fn key_hint_window(state: &mut crate::state::State, window: Window) -> Option<String> {
    if let Some(name) = window_name(state, window)
        && name.contains("key_hint")
    {
        let pieces: Vec<&str> = name.split(' ').collect();
        if pieces.len() == 2 {
            return Some(pieces[1].to_owned());
        }
    }
    None
}

pub fn window_name(state: &mut crate::state::State, window: Window) -> Option<String> {
    let reply = state
        .conn
        .get_property(false, window, AtomEnum::WM_NAME, AtomEnum::STRING, 0, 1024)
        .ok()?
        .reply()
        .ok()?;

    if reply.value.is_empty() {
        return None;
    }

    String::from_utf8(reply.value).ok()
}

pub fn monitor_index(state: &crate::state::State, window: Window) -> usize {
    for (monitor_index, monitor) in state.monitors.iter().enumerate() {
        for workspace in &monitor.workspaces {
            if let Some(main) = workspace.main_window
                && main == window
            {
                return monitor_index;
            }
            for side_window in &workspace.side_windows {
                if let Some(side) = side_window
                    && side == &window
                {
                    return monitor_index;
                }
            }
            for floating in &workspace.floatings {
                if floating == &window {
                    return monitor_index;
                }
            }
            for (_, key_window) in &workspace.key_hint_windows {
                if key_window == &window {
                    return monitor_index;
                }
            }
            if let Some(help) = workspace.help_window
                && help == window
            {
                return monitor_index;
            }
        }
    }
    0
}
