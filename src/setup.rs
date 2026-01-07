use libc::{c_int, c_uint};
use x11::xlib;

pub fn run_startups(state: &mut crate::state::State) {
    for startup in &state.settings.applications.startups {
        crate::windows::run_command(startup);
    }
}

pub fn mouse_input(state: &mut crate::state::State) {
    unsafe {
        xlib::XGrabButton(
            state.display,
            1,
            0,
            xlib::XDefaultRootWindow(state.display),
            true as c_int,
            (xlib::ButtonPressMask | xlib::ButtonReleaseMask | xlib::PointerMotionMask) as c_uint,
            xlib::GrabModeSync,
            xlib::GrabModeSync,
            0,
            0,
        ); // left mouse button
        xlib::XGrabButton(
            state.display,
            3,
            0,
            xlib::XDefaultRootWindow(state.display),
            true as c_int,
            (xlib::ButtonPressMask | xlib::ButtonReleaseMask | xlib::PointerMotionMask) as c_uint,
            xlib::GrabModeSync,
            xlib::GrabModeSync,
            0,
            0,
        ); // right mouse button
    };
}

pub fn key_input(state: &mut crate::state::State) {
    for k in crate::keymap::get_key_strings(state) {
        let key = crate::keymap::parse_string(&k); 
        if let Some(key) = key {
            unsafe {
                xlib::XGrabKey(
                    state.display,
                    xlib::XKeysymToKeycode(state.display, key as u64) as i32,
                    xlib::Mod4Mask, // super key
                    xlib::XDefaultRootWindow(state.display),
                    xlib::True,
                    xlib::GrabModeAsync,
                    xlib::GrabModeAsync,
                )
            };
        } else {
            eprintln!("unknown key in settings: {}", k);
        }
    }
}

pub fn windows(state: &mut crate::state::State) {
    unsafe {
        let root = xlib::XDefaultRootWindow(state.display);
        xlib::XSelectInput(
            state.display,
            root,
            xlib::SubstructureRedirectMask | xlib::SubstructureNotifyMask,
        );
        xlib::XSync(state.display, 0 /* False */);
    };
}
