use x11rb::connection::Connection;
use x11rb::protocol::xproto::{ButtonIndex, ConnectionExt, EventMask, GrabMode, ModMask};

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
    grab_keys(state);
}

// (re)load the keyboard mapping and grab every bound key; also used on MappingNotify
pub fn grab_keys(state: &mut crate::state::State) {
    refresh_keyboard_mapping(state);
    if let Err(e) = state.conn.ungrab_key(0u8, state.root, ModMask::ANY) {
        log::error!("setup::grab_keys(..) ungrab key error: {:?}", e);
    }
    let mut shifts: Vec<String> = vec![];
    shifts.push(state.settings.bindings.monitor.clone());
    shifts.push(state.settings.bindings.close_main.clone());
    for workspace_key in &state.settings.bindings.workspaces {
        shifts.push(workspace_key.clone());
    }
    for k in crate::keymap::get_key_strings(state) {
        let keysym = crate::keymap::parse_string(&k.clone());
        if let Some(keysym) = keysym {
            if let Some(keycode) = keysym_to_keycode(state, keysym) {
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
            log::error!("unknown key in settings: {}", k);
        }
    }
    if let Err(e) = state.conn.flush() {
        log::error!("setup::grab_keys(..) flush error: {:?}", e);
    }
}

fn refresh_keyboard_mapping(state: &mut crate::state::State) {
    let min_keycode = state.conn.setup().min_keycode;
    let max_keycode = state.conn.setup().max_keycode;
    state.keyboard_mapping = match state
        .conn
        .get_keyboard_mapping(min_keycode, max_keycode - min_keycode + 1)
    {
        Ok(cookie) => match cookie.reply() {
            Ok(reply) => Some(reply),
            Err(e) => {
                log::error!("setup::refresh_keyboard_mapping(..) reply error: {:?}", e);
                None
            }
        },
        Err(e) => {
            log::error!("setup::refresh_keyboard_mapping(..) request error: {:?}", e);
            None
        }
    };
}

fn keysym_to_keycode(state: &crate::state::State, keysym: u32) -> Option<u8> {
    let min_keycode = state.conn.setup().min_keycode;
    let mapping = state.keyboard_mapping.as_ref()?;

    let keysyms_per_keycode = mapping.keysyms_per_keycode as usize;
    for (idx, mapped) in mapping.keysyms.iter().enumerate() {
        if *mapped == keysym {
            return Some((min_keycode as usize + idx / keysyms_per_keycode) as u8);
        }
    }
    None
}
