use libc::{c_int, c_uint};
use x11::xlib;

pub fn map_request(state: &mut crate::state::State) {
    let event: xlib::XMapRequestEvent = From::from(state.event);
    if state.main_window.is_none() {
        fill_main_space(state, event.window);
    } else {
        send_side_space(state, event.window);
    }
    unsafe { xlib::XMapWindow(state.display, event.window) };
}

pub fn button(state: &mut crate::state::State) {
    let event: xlib::XButtonEvent = From::from(state.event);
    if event.subwindow == 0 {
        return;
    }
    if let Some(existing) = state.main_window {
        if existing == event.subwindow {
            // TODO: make sure clicking gets passed
            return;
        }
        remove_side_window(state, event.subwindow);
        fill_main_space(state, event.subwindow);
        send_side_space(state, existing);
    }
}

fn fill_main_space(state: &mut crate::state::State, window: xlib::Window) {
    unsafe {
        xlib::XMoveResizeWindow(
            state.display,
            window,
            0,
            0,
            state.sizes.main.0 as c_uint,
            state.sizes.main.1 as c_uint,
        );
    }
    state.main_window = Some(window);
}

fn send_side_space(state: &mut crate::state::State, window: xlib::Window) {
    remove_side_window(state, window);
    state.side_windows.push(Some(window));
    layout_side_space(state);
}

fn remove_side_window(state: &mut crate::state::State, window: xlib::Window) {
    let mut removes: Vec<usize> = vec![];
    for (index, side_window) in state.side_windows.iter().enumerate() {
        if side_window.is_none() || side_window.unwrap() == window {
            removes.push(index);
        }
    }
    for index in removes {
        state.side_windows.remove(index);
    }
}

fn layout_side_space(state: &mut crate::state::State) {
    let section_size = state.sizes.screen.1 as f32 / state.side_windows.len() as f32; // TODO: should be using size?
    for (index, window) in state.side_windows.iter().enumerate() {
        if let Some(window) = window {
            let section_pos = section_size * index as f32;
            println!(
                "placing window {} at {},{} {}x{}",
                window,
                state.sizes.main.0 as c_int,
                section_pos as c_int, // TODO: investigate cast
                state.sizes.side.0 as c_uint,
                section_size as c_uint, // TODO: investigate cast
            );
            unsafe {
                xlib::XMoveResizeWindow(
                    state.display,
                    *window,
                    state.sizes.main.0 as c_int,
                    section_pos as c_int, // TODO: investigate cast
                    state.sizes.side.0 as c_uint,
                    section_size as c_uint, // TODO: investigate cast
                );
            }
        }
    }
}
