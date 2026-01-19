use x11rb::CURRENT_TIME;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::{
    AtomEnum, ButtonIndex, ChangeWindowAttributesAux, ConnectionExt, CreateWindowAux, EventMask,
    GrabMode, ModMask, PropMode, WindowClass,
};
use x11rb::rust_connection::RustConnection;
use x11rb::wrapper::ConnectionExt as WrapperConnectionExt;

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
        match std::process::Command::new("dbus-launch")
            .arg("--exit-with-session")
            .spawn()
        {
            Ok(_) => println!("Successfully launched dbus-launch"),
            Err(e) => eprintln!("Warning: Failed to launch dbus-launch: {}", e),
        }
    }
}

pub fn mouse_input(state: &mut crate::state::State) {
    let event_mask =
        EventMask::BUTTON_PRESS | EventMask::BUTTON_RELEASE | EventMask::POINTER_MOTION;

    // Left mouse button
    state
        .conn
        .grab_button(
            true,
            state.root,
            event_mask,
            GrabMode::SYNC,
            GrabMode::SYNC,
            0u32,
            0u32,
            ButtonIndex::M1,
            ModMask::ANY,
        )
        .expect("Failed to grab button 1");

    // Right mouse button
    state
        .conn
        .grab_button(
            true,
            state.root,
            event_mask,
            GrabMode::SYNC,
            GrabMode::SYNC,
            0u32,
            0u32,
            ButtonIndex::M3,
            ModMask::ANY,
        )
        .expect("Failed to grab button 3");
}

pub fn key_input(state: &mut crate::state::State) {
    std::thread::sleep(std::time::Duration::from_millis(500));
    for k in crate::keymap::get_key_strings(state) {
        let keysym = crate::keymap::parse_string(&k);
        if let Some(keysym) = keysym {
            if let Some(keycode) = keysym_to_keycode(&state.conn, state.root, keysym) {
                state
                    .conn
                    .grab_key(
                        true,
                        state.root,
                        ModMask::M4,
                        keycode,
                        GrabMode::ASYNC,
                        GrabMode::ASYNC,
                    )
                    .expect("Failed to grab key");
            }
        } else {
            eprintln!("unknown key in settings: {}", k);
        }
    }
    let keysym = crate::keymap::parse_string(&state.settings.bindings.monitor);
    if let Some(keysym) = keysym {
        if let Some(keycode) = keysym_to_keycode(&state.conn, state.root, keysym) {
            state
                .conn
                .grab_key(
                    true,
                    state.root,
                    ModMask::M4 | ModMask::SHIFT,
                    keycode,
                    GrabMode::ASYNC,
                    GrabMode::ASYNC,
                )
                .expect("Failed to grab key");
        }
    } else {
        eprintln!(
            "unknown key in settings: {}",
            &state.settings.bindings.monitor
        );
    }
}

fn keysym_to_keycode(conn: &RustConnection, _root: u32, keysym: u32) -> Option<u8> {
    let setup = conn.setup();
    let min_keycode = setup.min_keycode;
    let max_keycode = setup.max_keycode;

    let mapping = conn
        .get_keyboard_mapping(min_keycode, max_keycode - min_keycode + 1)
        .ok()?
        .reply()
        .ok()?;

    let keysyms_per_keycode = mapping.keysyms_per_keycode as usize;
    for i in 0..=(max_keycode - min_keycode) as usize {
        for j in 0..keysyms_per_keycode {
            let idx = i * keysyms_per_keycode + j;
            if idx < mapping.keysyms.len() && mapping.keysyms[idx] == keysym {
                return Some((min_keycode as usize + i) as u8);
            }
        }
    }
    None
}

pub fn windows(state: &mut crate::state::State) {
    state
        .conn
        .change_window_attributes(
            state.root,
            &ChangeWindowAttributesAux::new()
                .event_mask(EventMask::SUBSTRUCTURE_REDIRECT | EventMask::SUBSTRUCTURE_NOTIFY),
        )
        .expect("Failed to select input on root");

    state.conn.sync().expect("Failed to sync");
}

pub fn connect() -> (RustConnection, usize) {
    let (conn, screen_num) = x11rb::connect(None).expect("Failed to connect to X server");

    let screen = &conn.setup().roots[screen_num];
    let root = screen.root;

    conn.ungrab_keyboard(CURRENT_TIME)
        .expect("Failed to ungrab keyboard");
    conn.ungrab_pointer(CURRENT_TIME)
        .expect("Failed to ungrab pointer");
    conn.ungrab_server().expect("Failed to ungrab server");
    conn.ungrab_key(0u8, root, ModMask::ANY)
        .expect("Failed to ungrab keys");

    let black_pixel = screen.black_pixel;
    conn.change_window_attributes(
        root,
        &ChangeWindowAttributesAux::new().background_pixel(black_pixel),
    )
    .expect("Failed to set background");

    conn.clear_area(false, root, 0i16, 0i16, 0u16, 0u16)
        .expect("Failed to clear window");

    conn.sync().expect("Failed to sync");

    (conn, screen_num)
}

pub fn init_ewmh(state: &mut crate::state::State) {
    // Create a dummy window for EWMH compliance check
    let check_window = state
        .conn
        .generate_id()
        .expect("Failed to generate window id");
    state
        .conn
        .create_window(
            0,
            check_window,
            state.root,
            0,
            0,
            1,
            1,
            0,
            WindowClass::INPUT_OUTPUT,
            0,
            &CreateWindowAux::new(),
        )
        .expect("Failed to create check window");

    // Set _NET_SUPPORTED on root window
    let supported_atoms = [
        state.atoms._NET_ACTIVE_WINDOW,
        state.atoms._NET_WM_NAME,
        state.atoms._NET_CLIENT_LIST,
        state.atoms._NET_SUPPORTING_WM_CHECK,
        state.atoms._NET_NUMBER_OF_DESKTOPS,
        state.atoms._NET_CURRENT_DESKTOP,
        state.atoms._NET_DESKTOP_NAMES,
    ];

    state
        .conn
        .change_property32(
            PropMode::REPLACE,
            state.root,
            state.atoms._NET_SUPPORTED,
            AtomEnum::ATOM,
            &supported_atoms,
        )
        .expect("Failed to set _NET_SUPPORTED");

    // Set _NET_SUPPORTING_WM_CHECK on root and check window
    state
        .conn
        .change_property32(
            PropMode::REPLACE,
            state.root,
            state.atoms._NET_SUPPORTING_WM_CHECK,
            AtomEnum::WINDOW,
            &[check_window],
        )
        .expect("Failed to set _NET_SUPPORTING_WM_CHECK on root");

    state
        .conn
        .change_property32(
            PropMode::REPLACE,
            check_window,
            state.atoms._NET_SUPPORTING_WM_CHECK,
            AtomEnum::WINDOW,
            &[check_window],
        )
        .expect("Failed to set _NET_SUPPORTING_WM_CHECK on check window");

    // Set WM name on check window
    let wm_name = b"Pick-Full-WM";
    state
        .conn
        .change_property8(
            PropMode::REPLACE,
            check_window,
            state.atoms._NET_WM_NAME,
            state.atoms.UTF8_STRING,
            wm_name,
        )
        .expect("Failed to set _NET_WM_NAME");

    // Set number of desktops
    let num_desktops = state.monitor().workspaces.len() as u32;
    state
        .conn
        .change_property32(
            PropMode::REPLACE,
            state.root,
            state.atoms._NET_NUMBER_OF_DESKTOPS,
            AtomEnum::CARDINAL,
            &[num_desktops],
        )
        .expect("Failed to set _NET_NUMBER_OF_DESKTOPS");

    // Set current desktop
    state
        .conn
        .change_property32(
            PropMode::REPLACE,
            state.root,
            state.atoms._NET_CURRENT_DESKTOP,
            AtomEnum::CARDINAL,
            &[0u32],
        )
        .expect("Failed to set _NET_CURRENT_DESKTOP");

    // Set desktop names
    let names: Vec<u8> = state
        .settings
        .bindings
        .workspaces
        .iter()
        .flat_map(|s| {
            let mut bytes = s.as_bytes().to_vec();
            bytes.push(0);
            bytes
        })
        .collect();

    state
        .conn
        .change_property8(
            PropMode::REPLACE,
            state.root,
            state.atoms._NET_DESKTOP_NAMES,
            state.atoms.UTF8_STRING,
            &names,
        )
        .expect("Failed to set _NET_DESKTOP_NAMES");

    state.conn.sync().expect("Failed to sync");
}
