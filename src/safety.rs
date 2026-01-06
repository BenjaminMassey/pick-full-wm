use std::sync::atomic::{AtomicBool, Ordering};
use x11::xlib;

static X_ERROR_OCCURRED: AtomicBool = AtomicBool::new(false);

extern "C" fn x_error_handler(
    display: *mut xlib::Display,
    error_event: *mut xlib::XErrorEvent,
) -> i32 {
    unsafe {
        let error = &*error_event;
        eprintln!(
            "X Error: request_code={} error_code={} resource_id={}",
            error.request_code, error.error_code, error.resourceid
        );
        X_ERROR_OCCURRED.store(true, Ordering::SeqCst);
        xlib::XAllowEvents(display, xlib::AsyncBoth, xlib::CurrentTime);
    }
    0 // continue
} // TODO: any way to have access to state such that we can cleanup windows?

pub fn setup_error_handler() {
    unsafe {
        xlib::XSetErrorHandler(Some(x_error_handler));
    }
}

pub fn window_exists(state: &crate::state::State, window: xlib::Window) -> bool {
    if window == 0 {
        // TODO: I believe this to make sense for x11
        return false;
    }

    X_ERROR_OCCURRED.store(false, Ordering::SeqCst);

    unsafe {
        let mut attrs: xlib::XWindowAttributes = std::mem::zeroed();
        xlib::XGetWindowAttributes(state.display, window, &mut attrs);
        xlib::XSync(state.display, xlib::False);
    }

    !X_ERROR_OCCURRED.load(Ordering::SeqCst)
}
