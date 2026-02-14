use x11rb::connection::Connection;
use x11rb::protocol::xproto::{ButtonIndex, ConnectionExt, EventMask, GrabMode, ModMask};
use x11rb::rust_connection::RustConnection;

pub fn mouse(state: &mut crate::state::State) {
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

pub fn keys(state: &mut crate::state::State) {
    std::thread::sleep(std::time::Duration::from_millis(500));
    let mut shifts: Vec<String> = vec![];
    shifts.push(state.settings.bindings.monitor.clone());
    for workspace_key in &state.settings.bindings.workspaces {
        shifts.push(workspace_key.clone());
    }
    for k in crate::keymap::get_key_strings(state) {
        let keysym = crate::keymap::parse_string(&k.clone());
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
                if shifts.contains(&k.clone()) {
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
            }
        } else {
            eprintln!("unknown key in settings: {}", k);
        }
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
