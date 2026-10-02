use x11rb::connection::Connection;
use x11rb::protocol::xproto::{
    AtomEnum, ClientMessageEvent, ConfigureWindowAux, ConnectionExt, EventMask, Window,
};

pub fn fill_main_space(state: &mut crate::state::State, window: Window) {
    log::info!("fill_main_space {}", window);
    let side = &state.settings.layout.side_orientation;
    let fill = state.settings.layout.conditional_full;
    let width = if side == "both" {
        if fill {
            if state.workspace().side_windows.is_empty() {
                state.monitor().sizes.screen.0
            } else if state.workspace().side_windows.len() == 1 {
                state.monitor().sizes.main.0
                    + (state.monitor().sizes.side.0 as f32 * 0.5f32).floor() as i32
            } else {
                state.monitor().sizes.main.0
            }
        } else {
            state.monitor().sizes.main.0 // - state.monitor().sizes.side.0
        }
    } else {
        if fill && state.workspace().side_windows.is_empty() {
            state.monitor().sizes.screen.0
        } else {
            state.monitor().sizes.main.0
        }
    };

    if let Err(e) = state.conn.configure_window(
        window,
        &ConfigureWindowAux::new()
            .x(
                if side == "right" || (fill && state.workspace().side_windows.is_empty()) {
                    state.monitor().position.0
                } else {
                    if side == "both" {
                        state.monitor().position.0
                            + (state.monitor().sizes.side.0 as f32 * 0.5f32).floor() as i32
                    } else {
                        state.monitor().sizes.side.0 + state.monitor().position.0
                    }
                },
            )
            .y(state.monitor().position.1)
            .width(width as u32)
            .height(state.monitor().sizes.main.1 as u32),
    ) {
        log::error!("windows::fill_main_space(..) move window error: {:?}", e);
    }

    if let Err(e) = state.conn.flush() {
        log::error!("windows::fill_main_space(..) flush error: {:?}", e);
    }

    state.mut_workspace().main_window = Some(window);
    focus_main(state);
    state.all_windows.insert(window);
}

pub fn send_side_space(state: &mut crate::state::State, window: Window, index: Option<usize>) {
    remove_side_window(state, window);
    if let Some(index) = index {
        state
            .mut_workspace()
            .side_windows
            .insert(index, Some(window));
    } else {
        state.mut_workspace().side_windows.push(Some(window));
    }
    state.all_windows.insert(window);
    crate::windows::layout::layout_side_space(state);
}

pub fn remove_side_window(state: &mut crate::state::State, window: Window) -> usize {
    let side_windows = &mut state.mut_workspace().side_windows;
    side_windows.retain(|w| w.is_some());
    let index = side_windows.iter().position(|w| *w == Some(window));
    side_windows.retain(|w| *w != Some(window));
    index.unwrap_or(side_windows.len())
}

pub fn focus_main(state: &mut crate::state::State) {
    if let Some(window) = state.workspace().main_window
        && crate::safety::window_exists(state, window)
    {
        crate::ewmh::set_active(state, window);
        crate::windows::layout::reapply_float_windows(state);
    }
    if let Err(e) = state.conn.flush() {
        log::error!("windows::focus_main(..) flush error: {:?}", e);
    }
    crate::windows::layout::place_close_boxes(state);
    crate::windows::layout::place_monitor_boxes(state);
}

pub fn remove_floating(state: &mut crate::state::State, window: Window) {
    let mut removes: Vec<usize> = vec![];
    for (index, floating) in state.workspace().floatings.iter().enumerate() {
        if floating == &window {
            removes.push(index);
        }
    }
    for remove in removes {
        state.mut_workspace().floatings.remove(remove);
    }
}

// politely ask via WM_DELETE_WINDOW if supported, otherwise kill the client
pub fn close_window(state: &mut crate::state::State, window: Window) {
    let supports_delete = match state.conn.get_property(
        false,
        window,
        state.atoms.WM_PROTOCOLS,
        AtomEnum::ATOM,
        0,
        1024,
    ) {
        Ok(cookie) => match cookie.reply() {
            Ok(reply) => reply
                .value32()
                .is_some_and(|mut atoms| atoms.any(|a| a == state.atoms.WM_DELETE_WINDOW)),
            Err(_) => false,
        },
        Err(_) => false,
    };
    if supports_delete {
        log::info!("Sending WM_DELETE_WINDOW to window {}.", window);
        let event = ClientMessageEvent::new(
            32,
            window,
            state.atoms.WM_PROTOCOLS,
            [state.atoms.WM_DELETE_WINDOW, x11rb::CURRENT_TIME, 0, 0, 0],
        );
        if let Err(e) = state
            .conn
            .send_event(false, window, EventMask::NO_EVENT, event)
        {
            log::error!("windows::close_window(..) send event error: {:?}", e);
        }
    } else {
        log::info!("Killing client of window {}.", window);
        if let Err(e) = state.conn.kill_client(window) {
            log::error!("windows::close_window(..) kill client error: {:?}", e);
        }
    }
    if let Err(e) = state.conn.flush() {
        log::error!("windows::close_window(..) flush error: {:?}", e);
    }
}
