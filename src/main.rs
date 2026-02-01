use x11rb::connection::Connection;
use x11rb::protocol::Event;

mod atoms;
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
    setup::internal::dbus_init();
    let mut state = state::State::init();
    setup::ewmh::init(&mut state);
    setup::internal::run_startups(&mut state);
    setup::input::mouse(&mut state);
    setup::input::keys(&mut state);
    setup::internal::windows(&mut state);

    loop {
        let event = state
            .conn
            .wait_for_event()
            .expect("Failed to wait for event");
        calc::update_current_monitor(&mut state);
        match event {
            Event::MapRequest(e) => {
                println!("MapRequest event!");
                events::window::map_request(&mut state, e);
            }
            Event::ButtonPress(e) => {
                println!("ButtonPress event!");
                events::input::button(&mut state, e);
            }
            Event::KeyRelease(e) => {
                println!("KeyRelease event!");
                events::input::key(&mut state, e);
            }
            Event::DestroyNotify(e) => {
                println!("DestroyNotify event!");
                events::window::destroy(&mut state, e);
            }
            Event::ClientMessage(e) => {
                println!("ClientMessage event!");
                events::client::message(&mut state, e);
            }
            _ => {}
        }
        crate::windows::audits::full(&mut state);
    }
}
