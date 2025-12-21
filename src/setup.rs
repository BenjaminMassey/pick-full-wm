use libc::{c_int, c_uint};
use x11::xlib;

pub fn input(state: &mut crate::state::State) {
    unsafe {
        xlib::XGrabButton(
            state.display,
            1,
            0,
            xlib::XDefaultRootWindow(state.display),
            true as c_int,
            (xlib::ButtonPressMask | xlib::ButtonReleaseMask | xlib::PointerMotionMask) as c_uint,
            xlib::GrabModeAsync,
            xlib::GrabModeAsync,
            0,
            0,
        ); // left mouse button
    };
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
