use libc::{c_int, c_uint};
use std::{ffi::CStr, ptr};
use x11::xlib;

pub fn fill_main_space(state: &mut crate::state::State, window: xlib::Window) {
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

pub fn send_side_space(state: &mut crate::state::State, window: xlib::Window) {
    remove_side_window(state, window);
    state.side_windows.push(Some(window));
    layout_side_space(state);
}

pub fn remove_side_window(state: &mut crate::state::State, window: xlib::Window) {
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

pub fn layout_side_space(state: &mut crate::state::State) {
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

pub fn is_excepted_window(state: &mut crate::state::State, window: xlib::Window) -> bool {
    if let Some(name) = get_window_name(state, window) {
        println!("window name: {}", &name);
        for exception in &state.settings.applications.excluded {
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

pub fn run_command(command: &str) {
    match std::process::Command::new("sh")
        .arg("-c")
        .arg(command)
        .spawn()
    {
        Ok(_) => println!("Run command: \"{}\"", command),
        Err(e) => eprintln!("Failed to run command \"{}\": {}", command, e),
    };
}
