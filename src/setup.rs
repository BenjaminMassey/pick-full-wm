use libc::{c_int, c_uint};
use std::ffi::CString;
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

pub fn init_ewmh(state: &mut crate::state::State) {
    unsafe {
        let root = xlib::XDefaultRootWindow(state.display);

        // Create a dummy window for EWMH compliance check
        let check_window = xlib::XCreateSimpleWindow(state.display, root, 0, 0, 1, 1, 0, 0, 0);

        // Declare which EWMH atoms we support
        let supported_atoms = [
            "_NET_ACTIVE_WINDOW",
            "_NET_WM_NAME",
            "_NET_CLIENT_LIST",
            "_NET_SUPPORTING_WM_CHECK",
        ];

        let mut atom_values: Vec<xlib::Atom> = Vec::new();
        for atom_name in &supported_atoms {
            let atom = xlib::XInternAtom(
                state.display,
                CString::new(*atom_name).unwrap().as_ptr(),
                xlib::False,
            );
            atom_values.push(atom);
        }

        // Set _NET_SUPPORTED on root window
        let net_supported = xlib::XInternAtom(
            state.display,
            CString::new("_NET_SUPPORTED").unwrap().as_ptr(),
            xlib::False,
        );
        xlib::XChangeProperty(
            state.display,
            root,
            net_supported,
            xlib::XA_ATOM,
            32,
            xlib::PropModeReplace,
            atom_values.as_ptr() as *const u8,
            atom_values.len() as i32,
        );

        // Set _NET_SUPPORTING_WM_CHECK on root and check window
        let net_supporting_wm_check = xlib::XInternAtom(
            state.display,
            CString::new("_NET_SUPPORTING_WM_CHECK").unwrap().as_ptr(),
            xlib::False,
        );
        xlib::XChangeProperty(
            state.display,
            root,
            net_supporting_wm_check,
            xlib::XA_WINDOW,
            32,
            xlib::PropModeReplace,
            &check_window as *const xlib::Window as *const u8,
            1,
        );
        xlib::XChangeProperty(
            state.display,
            check_window,
            net_supporting_wm_check,
            xlib::XA_WINDOW,
            32,
            xlib::PropModeReplace,
            &check_window as *const xlib::Window as *const u8,
            1,
        );

        // Set WM name on check window
        let net_wm_name = xlib::XInternAtom(
            state.display,
            CString::new("_NET_WM_NAME").unwrap().as_ptr(),
            xlib::False,
        );
        let utf8_string = xlib::XInternAtom(
            state.display,
            CString::new("UTF8_STRING").unwrap().as_ptr(),
            xlib::False,
        );
        let wm_name = CString::new("Pick-Full-WM").unwrap();
        xlib::XChangeProperty(
            state.display,
            check_window,
            net_wm_name,
            utf8_string,
            8,
            xlib::PropModeReplace,
            wm_name.as_ptr() as *const u8,
            wm_name.as_bytes().len() as i32,
        );

        xlib::XSync(state.display, xlib::False);
    }
}
