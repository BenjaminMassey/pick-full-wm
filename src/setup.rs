use libc::{c_int, c_uint};
use x11::xlib;

pub fn run_startups(state: &mut crate::state::State) {
    for startup in &state.settings.applications.startups {
        crate::windows::run_command(startup);
    }
}

pub fn dbus_init() {
    if std::process::Command::new("which")
        .arg("dbus-launch")
        .status()
        .is_ok()
        && std::env::var("DBUS_SESSION_BUS_ADDRESS").is_err()
    {
        std::process::Command::new("dbus-launch")
            .arg("--exit-with-session")
            .spawn()
            .ok();
    }
} // helps with startup for certain apps like KDE ones

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
    std::thread::sleep(std::time::Duration::from_millis(500));
    // allows time for login manager to ungrab properly
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
pub fn display(arg0: i8) -> *mut xlib::Display {
    unsafe {
        let display = xlib::XOpenDisplay(&arg0);
        let root = xlib::XDefaultRootWindow(display);
        let screen = xlib::XDefaultScreen(display);

        xlib::XUngrabKeyboard(display, xlib::CurrentTime);
        xlib::XUngrabPointer(display, xlib::CurrentTime);
        xlib::XUngrabServer(display);
        xlib::XUngrabKey(display, xlib::AnyKey, xlib::AnyModifier as u32, root);

        xlib::XSetWindowBackground(display, root, xlib::XBlackPixel(display, screen));

        xlib::XClearWindow(display, root);

        xlib::XSync(display, xlib::False);

        display
    }
}
