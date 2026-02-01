use x11rb::connection::Connection;
use x11rb::protocol::xproto::{ConfigureWindowAux, ConnectionExt, Window};

pub fn fill_main_space(state: &mut crate::state::State, window: Window) {
    let width =
        if state.settings.layout.conditional_full && state.workspace().side_windows.is_empty() {
            state.monitor().sizes.screen.0
        } else {
            state.monitor().sizes.main.0
        };

    if let Err(e) = state.conn.configure_window(
        window,
        &ConfigureWindowAux::new()
            .x(state.monitor().position.0)
            .y(state.monitor().position.1)
            .width(width as u32)
            .height(state.monitor().sizes.main.1 as u32),
    ) {
        eprintln!("windows::fill_main_space(..) move window error: {:?}", e);
    }

    if let Err(e) = state.conn.flush() {
        eprintln!("windows::fill_main_space(..) flush error: {:?}", e);
    }

    state.mut_workspace().main_window = Some(window);
    focus_main(state);
}

pub fn send_side_space(state: &mut crate::state::State, window: Window) {
    remove_side_window(state, window);
    state.mut_workspace().side_windows.push(Some(window));
    crate::windows::layout::layout_side_space(state);
}

pub fn remove_side_window(state: &mut crate::state::State, window: Window) {
    let mut removes: Vec<usize> = vec![];
    for (index, side_window) in state.workspace().side_windows.iter().enumerate() {
        if side_window.is_none() || side_window.unwrap() == window {
            removes.push(index);
        }
    }
    for index in removes {
        state.mut_workspace().side_windows.remove(index);
    }
}

pub fn focus_main(state: &mut crate::state::State) {
    if let Some(window) = state.workspace().main_window
        && crate::safety::window_exists(state, window)
    {
        crate::ewmh::set_active(state, window);
        crate::windows::layout::reapply_float_windows(state);
    }
    if let Err(e) = state.conn.flush() {
        eprintln!("windows::focus_main(..) flush error: {:?}", e);
    }
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
