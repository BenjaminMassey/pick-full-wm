use libc::{c_int, c_uint};
use std::{ffi::CStr, ptr};
use x11::xlib;

const EXCEPTIONS: &str = "polybar,rofi";

pub fn map_request(state: &mut crate::state::State) {
    let event: xlib::XMapRequestEvent = From::from(state.event);
    if event.window == 0 {
        return;
    }
    unsafe { xlib::XMapWindow(state.display, event.window) };
    if is_excepted_window(state, event.window) {
        return;
    }
    if state.main_window.is_none() {
        fill_main_space(state, event.window);
    } else {
        send_side_space(state, event.window);
    }
}

pub fn button(state: &mut crate::state::State) {
    let event: xlib::XButtonEvent = From::from(state.event);
    if event.subwindow == 0 || is_excepted_window(state, event.subwindow) {
        return;
    }
    if event.button == 1 {
        // left click
        if let Some(existing) = state.main_window {
            if existing == event.subwindow {
                // TODO: make sure clicking gets passed
                return;
            }
            remove_side_window(state, event.subwindow);
            fill_main_space(state, event.subwindow);
            send_side_space(state, existing);
        }
    } else if event.button == 3 {
        // right click
        if let Some(existing) = state.main_window
            && existing == event.subwindow
        {
            // TODO: make sure clicking gets passed
            return;
        }
        unsafe { xlib::XDestroyWindow(state.display, event.subwindow) };
    }
}

pub fn destroy(state: &mut crate::state::State) {
    let event: xlib::XDestroyWindowEvent = From::from(state.event);
    if let Some(main_window) = state.main_window {
        if event.window == main_window {
            if !state.side_windows.is_empty() {
                if let Some(target) = state.side_windows[0] {
                    remove_side_window(state, target);
                    fill_main_space(state, target);
                }
            } else {
                state.main_window = None;
            }
        } else {
            remove_side_window(state, event.window);
        }
    }
    layout_side_space(state);
}

fn fill_main_space(state: &mut crate::state::State, window: xlib::Window) {
    println!(
        "fill_main_space {} 0,0 {}x{}",
        window, state.sizes.main.0 as c_uint, state.sizes.main.1 as c_uint,
    );
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
    let section_size = state.sizes.side.1 as f32 / state.side_windows.len() as f32;
    for (index, window) in state.side_windows.iter().enumerate() {
        if let Some(window) = window {
            let section_pos = section_size * index as f32;
            println!(
                "layout_side_space {} {},{} {}x{}",
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

fn is_excepted_window(state: &mut crate::state::State, window: xlib::Window) -> bool {
    if let Some(name) = get_window_name(state, window) {
        println!("window name: {}", &name);
        for exception in EXCEPTIONS.split(",") {
            if name.contains(exception) {
                return true;
            }
        }
    }
    false
}

fn get_window_name(state: &mut crate::state::State, window: xlib::Window) -> Option<String> {
    let mut name_ptr: *mut i8 = ptr::null_mut();
    let fetch = unsafe { xlib::XFetchName(state.display, window, &mut name_ptr) };
    if fetch != 0 && !name_ptr.is_null() {
        let c_name = unsafe { CStr::from_ptr(name_ptr) };
        let name = c_name.to_string_lossy().into_owned();
        unsafe { xlib::XFree(name_ptr as *mut _) };
        Some(name)
    } else {
        None
    }
}
