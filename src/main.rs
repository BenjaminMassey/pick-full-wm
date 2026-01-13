extern crate libc;
extern crate x11;

use x11::xlib;

mod binaries;
mod calc;
mod events;
mod ewmh;
mod keymap;
mod safety;
mod settings;
mod setup;
mod state;
mod windows;

fn main() {
    safety::setup_error_handler();
    setup::dbus_init();
    let mut state = state::State::init();
    setup::run_startups(&mut state);
    setup::mouse_input(&mut state);
    setup::key_input(&mut state);
    setup::windows(&mut state);

    loop {
        unsafe {
            xlib::XNextEvent(state.display, &mut state.event);

            match state.event.get_type() {
                xlib::MapRequest => {
                    println!("MapRequest event!");
                    events::map_request(&mut state);
                }
                xlib::ButtonPress => {
                    println!("ButtonPress event!");
                    events::button(&mut state);
                }
                xlib::KeyRelease => {
                    println!("KeyRelease event!");
                    events::key(&mut state);
                }
                xlib::DestroyNotify => {
                    println!("DestroyNotify event!");
                    events::destroy(&mut state);
                }
                _ => {}
            };
        }
    }
}
