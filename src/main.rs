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
    setup::dbus_init();
    let mut state = state::State::init();
    setup::init_ewmh(&mut state);
    setup::run_startups(&mut state);
    setup::mouse_input(&mut state);
    setup::key_input(&mut state);
    setup::windows(&mut state);

    loop {
        let event = state.conn.wait_for_event().expect("Failed to wait for event");
        calc::update_current_monitor(&mut state);
        match event {
            Event::MapRequest(e) => {
                println!("MapRequest event!");
                events::map_request(&mut state, e);
            }
            Event::ButtonPress(e) => {
                println!("ButtonPress event!");
                events::button(&mut state, e);
            }
            Event::KeyRelease(e) => {
                println!("KeyRelease event!");
                events::key(&mut state, e);
            }
            Event::DestroyNotify(e) => {
                println!("DestroyNotify event!");
                events::destroy(&mut state, e);
            }
            Event::ClientMessage(e) => {
                println!("ClientMessage event!");
                events::client_message(&mut state, e);
            }
            _ => {}
        }
    }
}
