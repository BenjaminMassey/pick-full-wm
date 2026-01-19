use x11rb::CURRENT_TIME;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::{
    Allow, ButtonPressEvent, ClientMessageEvent, ConnectionExt, DestroyNotifyEvent, KeyButMask,
    KeyReleaseEvent, MapRequestEvent,
};

pub fn map_request(state: &mut crate::state::State, event: MapRequestEvent) {
    if !crate::safety::window_exists(state, event.window) {
        if let Err(e) = state.conn.allow_events(Allow::ASYNC_BOTH, CURRENT_TIME) {
            eprintln!("events::map_request(..) allow events error: {:?}", e);
        }
        if let Err(e) = state.conn.flush() {
            eprintln!("events::map_request(..) flush error: {:?}", e);
        }
        return;
    }
    if let Err(e) = state.conn.map_window(event.window) {
        eprintln!("events::map_request(..) map window error: {:?}", e);
    }
    if let Some(key) = crate::windows::get_key_hint_window(state, event.window) {
        if let Some(entry) = state.mut_workspace().key_hint_windows.get_mut(&key) {
            *entry = event.window;
        } else {
            state
                .mut_workspace()
                .key_hint_windows
                .insert(key, event.window);
        }
        crate::windows::layout_side_space(state);
        return;
    }
    if crate::windows::is_help_window(state, event.window) {
        crate::ewmh::set_active(state, event.window);
        if let Err(e) = state.conn.flush() {
            eprintln!("events::map_request(..) flush error: {:?}", e);
        }
        state.mut_workspace().help_window = Some(event.window);
        return;
    }
    if crate::windows::is_excepted_window(state, event.window) {
        return;
    }
    if state.workspace().main_window.is_none() {
        crate::windows::fill_main_space(state, event.window);
    } else {
        crate::windows::send_side_space(state, event.window);
    }
}

pub fn button(state: &mut crate::state::State, event: ButtonPressEvent) {
    if !crate::safety::window_exists(state, event.event)
        || crate::windows::is_excepted_window(state, event.child)
    {
        if let Err(e) = state.conn.allow_events(Allow::REPLAY_POINTER, CURRENT_TIME) {
            eprintln!("events::button(..) allow events error: {:?}", e);
        }
        if let Err(e) = state.conn.flush() {
            eprintln!("events::button(..) flush error: {:?}", e);
        }
        return;
    }
    if event.detail == 1 {
        // left click
        if let Some(existing) = state.workspace().main_window {
            if existing == event.child {
                if let Err(e) = state.conn.allow_events(Allow::REPLAY_POINTER, CURRENT_TIME) {
                    eprintln!("events::button(..) allow events errors: {:?}", e)
                };
                if let Err(e) = state.conn.flush() {
                    eprintln!("events::button(..) flush error: {:?}", e);
                }
                return;
            }
            crate::windows::remove_side_window(state, event.child);
            crate::windows::fill_main_space(state, event.child);
            crate::windows::send_side_space(state, existing);
            if let Err(e) = state.conn.allow_events(Allow::ASYNC_POINTER, CURRENT_TIME) {
                eprintln!("events::button(..) allow events error: {:?}", e);
            }
            if let Err(e) = state.conn.flush() {
                eprintln!("events::button(..) flush error: {:?}", e);
            }
        }
    } else if event.detail == 3 {
        // right click
        if let Some(existing) = state.workspace().main_window
            && existing == event.child
        {
            if let Err(e) = state.conn.allow_events(Allow::REPLAY_POINTER, CURRENT_TIME) {
                eprintln!("events::button(..) allow events errors: {:?}", e)
            };
            if let Err(e) = state.conn.flush() {
                eprintln!("events::button(..) flush error: {:?}", e);
            }
            return;
        }
        if let Err(e) = state.conn.destroy_window(event.child) {
            eprintln!("events::button(..) destroy window error: {:?}", e);
        }
        crate::windows::layout_side_space(state);
        if let Err(e) = state.conn.allow_events(Allow::ASYNC_POINTER, CURRENT_TIME) {
            eprintln!("events::button(..) allow events error: {:?}", e);
        }
        if let Err(e) = state.conn.flush() {
            eprintln!("events::button(..) flush error: {:?}", e);
        }
    }
}

pub fn key(state: &mut crate::state::State, event: KeyReleaseEvent) {
    let keysym = keycode_to_keysym(state, event.detail);
    let mod4_pressed = event.state.contains(KeyButMask::MOD4);

    let launcher_key = crate::keymap::parse_string(&state.settings.bindings.launcher);
    if let Some(launcher_key) = launcher_key {
        if keysym == Some(launcher_key) && mod4_pressed {
            crate::windows::run_command(&state.settings.applications.launcher);
        }
    }

    for (index, key) in state.settings.bindings.swaps.clone().iter().enumerate() {
        if index >= state.workspace().side_windows.len() {
            continue;
        }
        let swap_key = crate::keymap::parse_string(key);
        if let Some(swap_key) = swap_key {
            if keysym == Some(swap_key) && mod4_pressed {
                let target = state.workspace().side_windows[index];
                if let Some(target) = target {
                    let existing = state.workspace().main_window.clone();
                    crate::windows::remove_side_window(state, target);
                    crate::windows::fill_main_space(state, target);
                    if let Some(existing) = existing {
                        crate::windows::send_side_space(state, existing);
                    }
                }
            }
        }
    }

    let close_key = crate::keymap::parse_string(&state.settings.bindings.close_main);
    if let Some(close_key) = close_key {
        if keysym == Some(close_key) && mod4_pressed {
            if let Some(main) = state.workspace().main_window {
                if let Err(e) = state.conn.destroy_window(main) {
                    eprintln!("events::key(..) destroy window error: {:?}", e);
                }
                if let Err(e) = state.conn.allow_events(Allow::ASYNC_POINTER, CURRENT_TIME) {
                    eprintln!("events::key(..) allow events error: {:?}", e);
                }
                if let Err(e) = state.conn.flush() {
                    eprintln!("events::key(..) flush error: {:?}", e);
                }
            }
        }
    }

    let full_key = crate::keymap::parse_string(&state.settings.bindings.fullscreen);
    if let Some(full_key) = full_key {
        if keysym == Some(full_key) && mod4_pressed {
            if let Some(main) = state.workspace().main_window {
                state.mut_workspace().fullscreen = !state.workspace().fullscreen;
                if state.workspace().fullscreen {
                    crate::windows::fullscreen(state, main);
                } else {
                    crate::windows::fill_main_space(state, main);
                }
            }
        }
    }

    let help_key = crate::keymap::parse_string(&state.settings.bindings.help);
    if let Some(help_key) = help_key {
        if keysym == Some(help_key) && mod4_pressed {
            crate::binaries::help_window();
        }
    }

    let term_key = crate::keymap::parse_string(&state.settings.bindings.terminal);
    if let Some(term_key) = term_key {
        if keysym == Some(term_key) && mod4_pressed {
            crate::windows::run_command(&state.settings.applications.terminal);
        }
    }

    for (index, key) in state
        .settings
        .bindings
        .workspaces
        .clone()
        .iter()
        .enumerate()
    {
        let workspace_key = crate::keymap::parse_string(key);
        if let Some(workspace_key) = workspace_key {
            if keysym == Some(workspace_key) && mod4_pressed && state.current_workspace != index {
                state.current_workspace = index;
                crate::windows::switch_workspace(state);
            }
        }
    }

    let monitor_key = crate::keymap::parse_string(&state.settings.bindings.monitor);
    if let Some(monitor_key) = monitor_key {
        if keysym == Some(monitor_key) && mod4_pressed {
            let index = (state.current_monitor + 1) % state.monitors.len();
            let target = &state.monitors[index];

            // Calculate center position, clamping to i16 range to prevent overflow
            let x = (target.position.0 + (target.sizes.screen.0 as f32 * 0.5) as i32)
                .clamp(-32768, 32767) as i16;
            let y = (target.position.1 + (target.sizes.screen.1 as f32 * 0.5) as i32)
                .clamp(-32768, 32767) as i16;

            if let Err(e) = state.conn.warp_pointer(0u32, state.root, 0, 0, 0, 0, x, y) {
                eprintln!("events::key(..) warp pointer error: {:?}", e);
            }
            if let Err(e) = state.conn.flush() {
                eprintln!("events::key(..) flush error: {:?}", e);
            }
            state.current_monitor = index;
            crate::windows::focus_main(state);
        }
    }
}

pub fn destroy(state: &mut crate::state::State, event: DestroyNotifyEvent) {
    for i in 0..state.monitor().workspaces.len() {
        if let Some(help) = state.monitor().workspaces[i].help_window
            && event.window == help
        {
            if let Some(main_window) = state.monitor().workspaces[i].main_window
                && state.current_workspace == i
            {
                crate::ewmh::set_active(state, main_window);
                if let Err(e) = state.conn.flush() {
                    eprintln!("events::destroy(..) flush error: {:?}", e);
                }
            }
            state.mut_monitor().workspaces[i].help_window = None;
            return;
        }
        let real_workspace = state.current_workspace.clone(); // TODO: gross, for windows.rs calls
        state.current_workspace = i; // TODO: gross, for windows.rs calls
        if let Some(main_window) = state.monitor().workspaces[i].main_window {
            if event.window == main_window {
                if !state.monitor().workspaces[i].side_windows.is_empty() {
                    if let Some(target) = state.monitor().workspaces[i].side_windows[0]
                        && state.current_workspace == i
                    {
                        crate::windows::remove_side_window(state, target);
                        crate::windows::fill_main_space(state, target);
                    }
                } else {
                    state.mut_workspace().main_window = None;
                }
            } else {
                crate::windows::remove_side_window(state, event.window);
            }
        }
        state.current_workspace = real_workspace; // TODO: gross, for windows.rs calls
    }
    crate::windows::layout_side_space(state);
    if state.workspace().main_window.is_none() {
        crate::ewmh::clear_active(state);
    }
}

pub fn client_message(state: &mut crate::state::State, event: ClientMessageEvent) {
    if event.type_ == state.atoms._NET_CURRENT_DESKTOP {
        let requested_workspace = event.data.as_data32()[0] as usize;
        if state.current_workspace != requested_workspace {
            state.current_workspace = requested_workspace;
            crate::windows::switch_workspace(state);
        }
    }
}

fn keycode_to_keysym(state: &crate::state::State, keycode: u8) -> Option<u32> {
    let setup = state.conn.setup();
    let min_keycode = setup.min_keycode;
    let max_keycode = setup.max_keycode;

    let mapping = state
        .conn
        .get_keyboard_mapping(min_keycode, max_keycode - min_keycode + 1)
        .ok()?
        .reply()
        .ok()?;

    let keysyms_per_keycode = mapping.keysyms_per_keycode as usize;
    let index = (keycode - min_keycode) as usize;

    if index * keysyms_per_keycode < mapping.keysyms.len() {
        let keysym = mapping.keysyms[index * keysyms_per_keycode];
        if keysym != 0 {
            return Some(keysym);
        }
    }
    None
}
