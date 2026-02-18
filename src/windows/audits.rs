use x11rb::connection::Connection;
use x11rb::protocol::xproto::{ConfigureWindowAux, ConnectionExt, StackMode};

pub fn key_hints(state: &mut crate::state::State, positions: &[(i32, i32)]) {
    if !state.settings.bindings.key_hints {
        return;
    }
    for (i, k) in state.settings.bindings.swaps.clone().iter().enumerate() {
        if i < positions.len() {
            if state.workspace().key_hint_windows.contains_key(k) {
                if let Err(e) = state.conn.configure_window(
                    state.workspace().key_hint_windows[k],
                    &ConfigureWindowAux::new()
                        .x(positions[i].0)
                        .y(positions[i].1)
                        .width(60)
                        .height(60), // TODO: not hardcoded size
                ) {
                    eprintln!("windows::audit_key_hints(..) move window error: {:?}", e);
                }

                if let Err(e) = state.conn.configure_window(
                    state.workspace().key_hint_windows[k],
                    &ConfigureWindowAux::new().stack_mode(StackMode::ABOVE),
                ) {
                    eprintln!("windows::audit_key_hints(..) raise window error: {:?}", e);
                }

                if let Err(e) = state.conn.flush() {
                    eprintln!("windows::audit_key_hints(..) flush error: {:?}", e);
                }
            } else {
                state.mut_workspace().key_hint_windows.insert(k.clone(), 0); // TODO: this is silly
                crate::binaries::key_hint(k); // will get captured by event.rs map_window
            }
        } else if let Some(key_hint) = state.workspace().key_hint_windows.get(k) {
            if let Err(e) = state.conn.destroy_window(*key_hint) {
                eprintln!("windows::audit_key_hints(..) destroy error: {:?}", e);
            }
            if let Err(e) = state.conn.flush() {
                eprintln!("windows::audit_key_hints(..) flush error: {:?}", e);
            }
            let _ = state.mut_workspace().key_hint_windows.remove(k);
        }
    }
}

pub fn side_windows(state: &mut crate::state::State) {
    let mut change = false;
    for window in state.workspace().side_windows.clone() {
        if let Some(window) = window
            && !crate::safety::window_exists(state, window)
        {
            crate::windows::core::remove_side_window(state, window);
            change = true;
        }
    }
    if change {
        crate::windows::layout::layout_side_space(state);
    }
}

pub fn main_space(state: &mut crate::state::State) {
    if let Some(main) = state.workspace().main_window
        && !crate::safety::window_exists(state, main)
    {
        state.mut_workspace().main_window = None;
        crate::windows::audits::side_windows(state);
        if !state.workspace().side_windows.is_empty()
            && let Some(target) = state.workspace().side_windows[0]
        {
            crate::windows::core::fill_main_space(state, target);
            crate::windows::core::remove_side_window(state, target);
        }
    }
}

pub fn full(state: &mut crate::state::State) {
    crate::windows::audits::main_space(state);
    crate::windows::audits::side_windows(state);
    crate::ewmh::update_client_list(state);
}
