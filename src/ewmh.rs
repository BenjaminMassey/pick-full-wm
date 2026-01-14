use std::ffi::CString;
use x11::xlib;

pub fn set_active(state: &mut crate::state::State, window: xlib::Window) {
    unsafe {
        let net_active_window = xlib::XInternAtom(
            state.display,
            CString::new("_NET_ACTIVE_WINDOW").unwrap().as_ptr(),
            xlib::False,
        );
        xlib::XChangeProperty(
            state.display,
            xlib::XDefaultRootWindow(state.display),
            net_active_window,
            xlib::XA_WINDOW,
            32,
            xlib::PropModeReplace,
            &window as *const xlib::Window as *const u8,
            1,
        );
        if crate::safety::window_exists(state, window) {
            xlib::XSetInputFocus(
                state.display,
                window,
                xlib::RevertToPointerRoot,
                xlib::CurrentTime,
            );
            xlib::XRaiseWindow(state.display, window);
        }
        xlib::XFlush(state.display);
    }
}

pub fn clear_active(state: &mut crate::state::State) {
    set_active(state, 0);
}

pub fn update_workspace(state: &crate::state::State) {
    unsafe {
        let net_current_desktop = xlib::XInternAtom(
            state.display,
            CString::new("_NET_CURRENT_DESKTOP").unwrap().as_ptr(),
            xlib::False
        );
        let current_desktop = state.current_workspace as u64;
        xlib::XChangeProperty(
            state.display,
            xlib::XDefaultRootWindow(state.display),
            net_current_desktop,
            xlib::XA_CARDINAL,
            32,
            xlib::PropModeReplace,
            &current_desktop as *const u64 as *const u8,
            1,
        );
        xlib::XFlush(state.display);
    }
}
