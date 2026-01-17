use libc::{c_int, c_uint};
use std::{
    ffi::{CStr, CString},
    ptr,
};
use x11::xlib;

pub fn fill_main_space(state: &mut crate::state::State, window: xlib::Window) {
    let width =
        if state.settings.layout.conditional_full && state.workspace().side_windows.is_empty() {
            state.monitor().sizes.screen.0
        } else {
            state.monitor().sizes.main.0
        };
    unsafe {
        xlib::XMoveResizeWindow(
            state.display,
            window,
            state.monitor().position.0,
            state.monitor().position.1,
            width as c_uint,
            state.monitor().sizes.main.1 as c_uint,
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
    let section_size =
        state.monitor().sizes.side.1 as f32 / state.workspace().side_windows.len() as f32;
    for (index, window) in state.workspace().side_windows.iter().enumerate() {
        if let Some(window) = window {
            let section_pos = section_size * index as f32;
            println!(
                "layout_side_space {} {},{} {}x{}",
                window,
                state.monitor().sizes.main.0 as c_int,
                section_pos as c_int, // TODO: investigate cast
                state.monitor().sizes.side.0 as c_uint,
                section_size as c_uint, // TODO: investigate cast
            );
            let position = (
                (state.monitor().position.0 + state.monitor().sizes.main.0 as i32) as c_int,
                (state.monitor().position.1 + section_pos as i32) as c_int, // TODO: investigate cast
            );
            positions.push(position);
            unsafe {
                xlib::XMoveResizeWindow(
                    state.display,
                    *window,
                    position.0,
                    position.1,
                    state.monitor().sizes.side.0 as c_uint, // TODO: take into account x offset
                    section_size as c_uint,                 // TODO: investigate cast
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
                    xlib::XMoveWindow(
                        state.display,
                        state.workspace().key_hint_windows[k],
                        positions[i].0,
                        positions[i].1,
                    );
                    xlib::XRaiseWindow(state.display, state.workspace().key_hint_windows[k]);
                    xlib::XFlush(state.display);
                }
            } else {
                state.mut_workspace().key_hint_windows.insert(k.clone(), 0); // TODO: this is silly
                crate::binaries::key_hint(k); // will get captured by event.rs map_window
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
            state.monitor().position.0
                + ((state.monitor().sizes.main.0 + state.monitor().sizes.side.0)
                    - state.monitor().sizes.screen.0),
            state.monitor().position.1
                + (state.monitor().sizes.main.1 - state.monitor().sizes.screen.1),
            state.monitor().sizes.screen.0 as c_uint,
            state.monitor().sizes.screen.1 as c_uint,
        );
    }
    focus_main(state);
}

pub fn is_excepted_window(state: &mut crate::state::State, window: xlib::Window) -> bool {
    if is_help_window(state, window) || get_key_hint_window(state, window).is_some() {
        return true;
    }
    if let Some(name) = get_window_name(state, window) {
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

pub fn focus_main(state: &mut crate::state::State) {
    if let Some(window) = state.workspace().main_window
        && crate::safety::window_exists(state, window)
    {
        crate::ewmh::set_active(state, window);
        reapply_float_windows(state);
    }
    unsafe { xlib::XFlush(state.display) };
}

pub fn switch_workspace(state: &mut crate::state::State) {
    for monitor in &state.monitors {
        for index in 0..monitor.workspaces.len() {
            if index == state.current_workspace {
                if let Some(main) = monitor.workspaces[index].main_window {
                    unsafe { xlib::XMapWindow(state.display, main) };
                }
                for window in &monitor.workspaces[index].side_windows {
                    if let Some(window) = window {
                        unsafe { xlib::XMapWindow(state.display, *window) };
                    }
                }
                if let Some(help) = monitor.workspaces[index].help_window {
                    unsafe { xlib::XMapWindow(state.display, help) };
                }
                for (_, window) in &monitor.workspaces[index].key_hint_windows {
                    unsafe { xlib::XMapWindow(state.display, *window) };
                }
                for window in &monitor.workspaces[index].floatings {
                    unsafe { xlib::XMapWindow(state.display, *window) };
                }
            } else {
                if let Some(main) = monitor.workspaces[index].main_window {
                    unsafe { xlib::XUnmapWindow(state.display, main) };
                }
                for window in &monitor.workspaces[index].side_windows {
                    if let Some(window) = window {
                        unsafe { xlib::XUnmapWindow(state.display, *window) };
                    }
                }
                if let Some(help) = monitor.workspaces[index].help_window {
                    unsafe { xlib::XUnmapWindow(state.display, help) };
                }
                for (_, window) in &monitor.workspaces[index].key_hint_windows {
                    unsafe { xlib::XUnmapWindow(state.display, *window) };
                }
                for window in &monitor.workspaces[index].floatings {
                    unsafe { xlib::XUnmapWindow(state.display, *window) };
                }
            }
        }
    }
    unsafe { xlib::XFlush(state.display) };
    crate::ewmh::update_workspace(state);
    crate::ewmh::clear_active(state);
    for i in 0..state.monitors.len() {
        let real_monitor = state.current_monitor; // TODO: gross
        state.current_monitor = i; // TODO: gross
        if let Some(main) = state.workspace().main_window {
            fill_main_space(state, main);
        }
        layout_side_space(state);
        state.current_monitor = real_monitor; // TODO: gross
    }
    crate::windows::focus_main(state);
}

pub fn is_popup(state: &crate::state::State, window: xlib::Window) -> bool {
    unsafe {
        // Check 1: Override-redirect windows (menus, tooltips)
        let mut attrs: xlib::XWindowAttributes = std::mem::zeroed();
        xlib::XGetWindowAttributes(state.display, window, &mut attrs);
        if attrs.override_redirect == xlib::True {
            return true;
        }

        // Check 2: Transient windows (dialogs with parent)
        let mut transient_for: xlib::Window = 0;
        if xlib::XGetTransientForHint(state.display, window, &mut transient_for) != 0 {
            return true;
        }

        // Check 3: Window type hints
        let net_wm_window_type = xlib::XInternAtom(
            state.display,
            CString::new("_NET_WM_WINDOW_TYPE").unwrap().as_ptr(),
            xlib::False,
        );

        let mut actual_type: xlib::Atom = 0;
        let mut actual_format: i32 = 0;
        let mut nitems: u64 = 0;
        let mut bytes_after: u64 = 0;
        let mut prop: *mut u8 = ptr::null_mut();

        let status = xlib::XGetWindowProperty(
            state.display,
            window,
            net_wm_window_type,
            0,
            1024,
            xlib::False,
            xlib::XA_ATOM,
            &mut actual_type,
            &mut actual_format,
            &mut nitems,
            &mut bytes_after,
            &mut prop,
        );

        if status == 0 && !prop.is_null() && nitems > 0 {
            let atoms = prop as *const xlib::Atom;
            let window_types = std::slice::from_raw_parts(atoms, nitems as usize);

            // Get atoms for types we want to exclude
            let dialog = xlib::XInternAtom(
                state.display,
                CString::new("_NET_WM_WINDOW_TYPE_DIALOG").unwrap().as_ptr(),
                xlib::False,
            );
            let utility = xlib::XInternAtom(
                state.display,
                CString::new("_NET_WM_WINDOW_TYPE_UTILITY")
                    .unwrap()
                    .as_ptr(),
                xlib::False,
            );
            let splash = xlib::XInternAtom(
                state.display,
                CString::new("_NET_WM_WINDOW_TYPE_SPLASH").unwrap().as_ptr(),
                xlib::False,
            );
            let notification = xlib::XInternAtom(
                state.display,
                CString::new("_NET_WM_WINDOW_TYPE_NOTIFICATION")
                    .unwrap()
                    .as_ptr(),
                xlib::False,
            );
            let popup_menu = xlib::XInternAtom(
                state.display,
                CString::new("_NET_WM_WINDOW_TYPE_POPUP_MENU")
                    .unwrap()
                    .as_ptr(),
                xlib::False,
            );
            let dropdown_menu = xlib::XInternAtom(
                state.display,
                CString::new("_NET_WM_WINDOW_TYPE_DROPDOWN_MENU")
                    .unwrap()
                    .as_ptr(),
                xlib::False,
            );
            let tooltip = xlib::XInternAtom(
                state.display,
                CString::new("_NET_WM_WINDOW_TYPE_TOOLTIP")
                    .unwrap()
                    .as_ptr(),
                xlib::False,
            );

            for &window_type in window_types {
                if window_type == dialog
                    || window_type == utility
                    || window_type == splash
                    || window_type == notification
                    || window_type == popup_menu
                    || window_type == dropdown_menu
                    || window_type == tooltip
                {
                    xlib::XFree(prop as *mut _);
                    return true;
                }
            }

            xlib::XFree(prop as *mut _);
        }

        false
    }
}

pub fn reapply_float_windows(state: &mut crate::state::State) {
    for window in state.workspace().floatings.clone() {
        if crate::safety::window_exists(state, window) {
            unsafe { xlib::XRaiseWindow(state.display, window) };
            unsafe { xlib::XFlush(state.display) };
        } else {
            remove_floating(state, &window);
            layout_side_space(state);
        }
    }
}

pub fn remove_floating(state: &mut crate::state::State, window: &xlib::Window) {
    let mut removes: Vec<usize> = vec![];
    for (index, floating) in state.workspace().floatings.iter().enumerate() {
        if floating == window {
            removes.push(index);
        }
    }
    for remove in removes {
        state.mut_workspace().floatings.remove(remove);
    }
}

pub fn center_window(state: &mut crate::state::State, window: xlib::Window) {
    unsafe {
        let mut attrs: xlib::XWindowAttributes = std::mem::zeroed();
        xlib::XGetWindowAttributes(state.display, window, &mut attrs);
        xlib::XMoveWindow(
            state.display,
            window,
            state.monitor().position.0
                + ((state.monitor().sizes.screen.0 as i32 - attrs.width) as f32 / 2f32) as i32,
            state.monitor().position.1
                + ((state.monitor().sizes.screen.1 as i32 - attrs.height) as f32 / 2f32) as i32,
        );
        xlib::XFlush(state.display);
    }
}

pub fn audit_side_windows(state: &mut crate::state::State) {
    let mut change = false;
    for window in state.workspace().side_windows.clone() {
        if let Some(window) = window
            && !crate::safety::window_exists(state, window)
        {
            remove_side_window(state, window);
            change = true;
        }
    }
    if change {
        layout_side_space(state);
    }
}

pub fn audit_main(state: &mut crate::state::State) {
    if let Some(main) = state.workspace().main_window
        && !crate::safety::window_exists(state, main)
    {
        state.mut_workspace().main_window = None;
        audit_side_windows(state);
        if !state.workspace().side_windows.is_empty()
            && let Some(target) = state.workspace().side_windows[0]
        {
            fill_main_space(state, target);
            remove_side_window(state, target);
        }
    }
}
pub fn full_audit(state: &mut crate::state::State) {
    audit_main(state);
    audit_side_windows(state);
}
