extern crate libc;
extern crate x11;

use libc::{c_int, c_uint};
use std::mem::zeroed;
use x11::xlib;

mod pf;

const SIZE_CONFIG: &str = "80%x100%";
//const SIZE_CONFIG: &str = "1800x1080";

fn main() {
    let mut arg0 = 0x0 as i8;
    let display: *mut xlib::Display = unsafe { xlib::XOpenDisplay(&mut arg0) };

    let mut attr: xlib::XWindowAttributes = unsafe { zeroed() };

    if display.is_null() {
        eprintln!("Display \"{}\" is null.", arg0);
        std::process::exit(1);
    }

    let display_size: (c_int, c_int) = (
        unsafe { xlib::XDisplayWidth(display, arg0.into()) },
        unsafe { xlib::XDisplayHeight(display, arg0.into()) },
    );

    println!("Display Size: {}x{}", display_size.0, display_size.1);

    let full_size = pf::get_full_size(display_size.0 as f32, display_size.1 as f32, SIZE_CONFIG);

    println!("Calculated Full Size: {}x{}", full_size.0, full_size.1);

    let _sides_size = (display_size.0 - full_size.0, display_size.1 - full_size.1); // TODO: use this

    grab_x_keys(display);

    // Setup ability to grab created windows
    let root = unsafe { xlib::XDefaultRootWindow(display) };
    unsafe {
        xlib::XSelectInput(
            display,
            root,
            xlib::SubstructureRedirectMask | xlib::SubstructureNotifyMask,
        )
    };
    unsafe {
        xlib::XSync(display, 0 /* False */)
    };

    let mut event: xlib::XEvent = unsafe { zeroed() };

    loop {
        unsafe {
            xlib::XNextEvent(display, &mut event);

            match event.get_type() {
                xlib::MapRequest => {
                    println!("MapRequest event!");
                    let xmap: xlib::XMapRequestEvent = From::from(event);
                    xlib::XMoveResizeWindow(
                        display,
                        xmap.window,
                        0,
                        0,
                        full_size.0 as c_uint,
                        full_size.1 as c_uint,
                    );
                    xlib::XMapWindow(display, xmap.window);
                }
                /* TODO: keypresses for hotswapping windows
                xlib::KeyPress => {
                    let xkey: xlib::XKeyEvent = From::from(event);
                    if xkey.subwindow != 0 {
                        xlib::XRaiseWindow(display, xkey.subwindow);
                    }
                }
                */
                xlib::ButtonPress => {
                    println!("ButtonPress event!");
                    let xbutton: xlib::XButtonEvent = From::from(event);
                    if xbutton.subwindow != 0 {
                        println!("clicked window #{}", xbutton.subwindow);
                        xlib::XGetWindowAttributes(display, xbutton.subwindow, &mut attr);
                        xlib::XMoveResizeWindow(
                            display,
                            xbutton.subwindow,
                            0,
                            0,
                            full_size.0 as c_uint,
                            full_size.1 as c_uint,
                        );
                    }
                }
                _ => {}
            };
        }
    }
}

fn grab_x_keys(display: *mut xlib::Display) {
    unsafe {
        xlib::XGrabButton(
            display,
            1,
            0,
            xlib::XDefaultRootWindow(display),
            true as c_int,
            (xlib::ButtonPressMask | xlib::ButtonReleaseMask | xlib::PointerMotionMask) as c_uint,
            xlib::GrabModeAsync,
            xlib::GrabModeAsync,
            0,
            0,
        ); // left mouse button
    };
}
