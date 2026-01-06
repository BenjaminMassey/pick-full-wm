use libc::c_uint;
use x11::{keysym, xlib};

pub fn map_request(state: &mut crate::state::State) {
    let event: xlib::XMapRequestEvent = From::from(state.event);
    if !crate::safety::window_exists(state, event.window) {
        unsafe { xlib::XAllowEvents(state.display, xlib::AsyncBoth, xlib::CurrentTime) };
        return;
    }
    unsafe { xlib::XMapWindow(state.display, event.window) };
    if crate::windows::is_excepted_window(state, event.window) {
        return;
    }
    if state.main_window.is_none() {
        crate::windows::fill_main_space(state, event.window);
    } else {
        crate::windows::send_side_space(state, event.window);
    }
}

pub fn button(state: &mut crate::state::State) {
    let event: xlib::XButtonEvent = From::from(state.event);
    if !crate::safety::window_exists(state, event.window) || crate::windows::is_excepted_window(state, event.subwindow) {
        unsafe { xlib::XAllowEvents(state.display, xlib::ReplayPointer, xlib::CurrentTime) };
        return;
    }
    if event.button == 1 {
        // left click
        if let Some(existing) = state.main_window {
            if existing == event.subwindow {
                unsafe {
                    xlib::XAllowEvents(state.display, xlib::ReplayPointer, xlib::CurrentTime)
                };
                return;
            }
            crate::windows::remove_side_window(state, event.subwindow);
            crate::windows::fill_main_space(state, event.subwindow);
            crate::windows::send_side_space(state, existing);
            unsafe { xlib::XAllowEvents(state.display, xlib::AsyncPointer, xlib::CurrentTime) };
        }
    } else if event.button == 3 {
        // right click
        if let Some(existing) = state.main_window
            && existing == event.subwindow
        {
            unsafe { xlib::XAllowEvents(state.display, xlib::ReplayPointer, xlib::CurrentTime) };
            return;
        }
        unsafe { xlib::XDestroyWindow(state.display, event.subwindow) };
        unsafe { xlib::XAllowEvents(state.display, xlib::AsyncPointer, xlib::CurrentTime) };
    }
}

pub fn key(state: &mut crate::state::State) {
    let event: xlib::XKeyReleasedEvent = From::from(state.event);
    let super_l = unsafe {
        event.keycode == xlib::XKeysymToKeycode(state.display, keysym::XK_Super_L as u64) as c_uint
    };
    let super_r = unsafe {
        event.keycode == xlib::XKeysymToKeycode(state.display, keysym::XK_Super_R as u64) as c_uint
    };
    if super_l || super_r {
        crate::windows::run_command(&state.settings.applications.launcher);
    }
}

pub fn destroy(state: &mut crate::state::State) {
    let event: xlib::XDestroyWindowEvent = From::from(state.event);
    if let Some(main_window) = state.main_window {
        if event.window == main_window {
            if !state.side_windows.is_empty() {
                if let Some(target) = state.side_windows[0] {
                    crate::windows::remove_side_window(state, target);
                    crate::windows::fill_main_space(state, target);
                }
            } else {
                state.main_window = None;
            }
        } else {
            crate::windows::remove_side_window(state, event.window);
        }
    }
    crate::windows::layout_side_space(state);
}
