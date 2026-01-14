use libc::{c_int, c_uint};
use std::{ffi::CStr, ptr};
use x11::xlib;

pub fn fill_main_space(state: &mut crate::state::State, window: xlib::Window) {
    let width =
        if state.settings.layout.conditional_full && state.workspace().side_windows.is_empty() {
            state.sizes.screen.0
        } else {
            state.sizes.main.0
        };
    unsafe {
        xlib::XMoveResizeWindow(
            state.display,
            window,
            state.position.0,
            state.position.1,
            width as c_uint,
            state.sizes.main.1 as c_uint,
        );
    }
    state.mut_workspace().main_window = Some(window);
    focus_main(state);
}

pub fn send_side_space(state: &mut crate::state::State, window: xlib::Window) {
    remove_side_window(state, window);
    state.mut_workspace().side_windows.push(Some(window));
    layout_side_space(state);
}

pub fn remove_side_window(state: &mut crate::state::State, window: xlib::Window) {
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

pub fn layout_side_space(state: &mut crate::state::State) {
    let mut positions: Vec<(c_int, c_int)> = vec![];
    let section_size = state.sizes.side.1 as f32 / state.workspace().side_windows.len() as f32;
    for (index, window) in state.workspace().side_windows.iter().enumerate() {
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
            let position = (
                (state.position.0 + state.sizes.main.0 as i32) as c_int,
                (state.position.1 + section_pos as i32) as c_int, // TODO: investigate cast
            );
            positions.push(position);
            unsafe {
                xlib::XMoveResizeWindow(
                    state.display,
                    *window,
                    position.0,
                    position.1,
                    (state.sizes.side.0 as i32 - state.position.0) as c_uint,
                    section_size as c_uint, // TODO: investigate cast
                );
            }
        }
    }
    audit_key_hints(state, &positions);
    if let Some(main) = state.workspace().main_window {
        fill_main_space(state, main);
    }
}

fn audit_key_hints(state: &mut crate::state::State, positions: &[(c_int, c_int)]) {
    if !state.settings.bindings.key_hints {
        return;
    }
    for (i, k) in state.settings.bindings.swaps.clone().iter().enumerate() {
        if i < positions.len() {
            if state.workspace().key_hint_windows.contains_key(k) {
                unsafe {
                    xlib::XMoveResizeWindow(
                        state.display,
                        state.workspace().key_hint_windows[k],
                        positions[i].0,
                        positions[i].1,
                        50 as c_uint, // TODO: connect to src/bin/key_hint.rs settings
                        50 as c_uint,
                    );
                    xlib::XRaiseWindow(state.display, state.workspace().key_hint_windows[k]);
                    xlib::XFlush(state.display);
                }
            } else {
                crate::binaries::key_hint(k);
            }
        } else if let Some(key_hint) = state.workspace().key_hint_windows.get(k) {
            unsafe {
                xlib::XDestroyWindow(state.display, *key_hint);
                xlib::XFlush(state.display);
            }
            let _ = state.mut_workspace().key_hint_windows.remove(k);
        }
    }
}

pub fn fullscreen(state: &mut crate::state::State, window: xlib::Window) {
    unsafe {
        xlib::XMoveResizeWindow(
            state.display,
            window,
            0,
            0,
            state.sizes.screen.0 as c_uint,
            state.sizes.screen.1 as c_uint,
        );
    }
    focus_main(state);
}

pub fn is_excepted_window(state: &mut crate::state::State, window: xlib::Window) -> bool {
    if is_help_window(state, window) || get_key_hint_window(state, window).is_some() {
        return true;
    }
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

pub fn is_help_window(state: &mut crate::state::State, window: xlib::Window) -> bool {
    if let Some(name) = get_window_name(state, window)
        && name.contains("pfwm help")
    {
        return true;
    }
    false
}

pub fn get_key_hint_window(
    state: &mut crate::state::State,
    window: xlib::Window,
) -> Option<String> {
    if let Some(name) = get_window_name(state, window)
        && name.contains("key_hint")
    {
        let pieces: Vec<&str> = name.split(" ").collect();
        if pieces.len() == 2 {
            return Some(pieces[1].to_owned());
        }
    }
    None
}

pub fn get_window_name(state: &mut crate::state::State, window: xlib::Window) -> Option<String> {
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

fn focus_main(state: &mut crate::state::State) {
    if let Some(window) = state.workspace().main_window
        && crate::safety::window_exists(state, window)
    {
        crate::ewmh::set_active(state, window);
    }
}

pub fn switch_workspace(state: &mut crate::state::State) {
    for index in 0..state.workspaces.len() {
        if index == state.current_workspace {
            if let Some(main) = state.workspaces[index].main_window {
                unsafe { xlib::XMapWindow(state.display, main) };
            }
            for window in &state.workspaces[index].side_windows {
                if let Some(window) = window {
                    unsafe { xlib::XMapWindow(state.display, *window) };
                }
            }
            if let Some(help) = state.workspaces[index].help_window {
                unsafe { xlib::XMapWindow(state.display, help) };
            }
            for (_, window) in &state.workspaces[index].key_hint_windows {
                unsafe { xlib::XMapWindow(state.display, *window) };
            }
        } else {
            if let Some(main) = state.workspaces[index].main_window {
                unsafe { xlib::XUnmapWindow(state.display, main) };
            }
            for window in &state.workspaces[index].side_windows {
                if let Some(window) = window {
                    unsafe { xlib::XUnmapWindow(state.display, *window) };
                }
            }
            if let Some(help) = state.workspaces[index].help_window {
                unsafe { xlib::XUnmapWindow(state.display, help) };
            }
            for (_, window) in &state.workspaces[index].key_hint_windows {
                unsafe { xlib::XUnmapWindow(state.display, *window) };
            }
        }
    }
    unsafe { xlib::XFlush(state.display) };
    crate::ewmh::update_workspace(state);
    crate::ewmh::clear_active(state);
    if let Some(main) = state.workspace().main_window {
        fill_main_space(state, main);
    }
    layout_side_space(state);
}
