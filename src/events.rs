use libc::c_uint;
use x11::xlib;

pub fn map_request(state: &mut crate::state::State) {
    let event: xlib::XMapRequestEvent = From::from(state.event);
    fill_main_space(state, event.window);
    unsafe { xlib::XMapWindow(state.display, event.window) };
}

pub fn button(state: &mut crate::state::State) {
    let event: xlib::XButtonEvent = From::from(state.event);
    if event.subwindow != 0 {
        fill_main_space(state, event.subwindow);
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
}
