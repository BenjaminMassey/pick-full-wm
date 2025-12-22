extern crate libc;
extern crate x11;

use x11::xlib;

mod calc;
mod events;
mod setup;
mod state;

fn main() {
    let mut state = state::State::init();

    setup::input(&mut state);
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
                xlib::DestroyNotify => {
                    println!("DestroyNotify event!");
                    events::destroy(&mut state);
                }
                _ => {}
            };
        }
    }
}
