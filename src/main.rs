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
    let mut state = state::State::init();
    setup::internal::logging(&state);
    setup::internal::dbus_init();
    setup::ewmh::init(&mut state);
    setup::internal::custom_startups(&mut state);
    setup::input::mouse(&mut state);
    setup::input::keys(&mut state);
    setup::internal::windows(&mut state);
    setup::internal::startups(&mut state);
    log::info!("Finished with initialization.");

    loop {
        let event = state
            .conn
            .wait_for_event()
            .expect("Failed to wait for event");
        calc::update_current_monitor(&mut state);
        match event {
            Event::MapRequest(e) => {
                events::window::map_request(&mut state, e);
            }
            Event::ButtonPress(e) => {
                events::input::button(&mut state, e);
            }
            Event::KeyRelease(e) => {
                events::input::key(&mut state, e);
            }
            Event::DestroyNotify(e) => {
                events::window::destroy(&mut state, e);
            }
            Event::ClientMessage(e) => {
                events::client::message(&mut state, e);
            }
            _ => {}
        }
        crate::windows::audits::full(&mut state);
    }
}
