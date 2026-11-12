use x11rb::connection::Connection;
use x11rb::protocol::xproto::{ConfigureWindowAux, ConnectionExt, StackMode, Window};

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
                    log::error!("windows::audit_key_hints(..) move window error: {:?}", e);
                }

                if !state.workspace().is_fullscreen()
                    && let Err(e) = state.conn.configure_window(
                        state.workspace().key_hint_windows[k],
                        &ConfigureWindowAux::new().stack_mode(StackMode::ABOVE),
                    )
                {
                    log::error!("windows::audit_key_hints(..) raise window error: {:?}", e);
                }

                if let Err(e) = state.conn.flush() {
                    log::error!("windows::audit_key_hints(..) flush error: {:?}", e);
                }
            } else {
                state.mut_workspace().key_hint_windows.insert(k.clone(), 0); // TODO: this is silly
                crate::binaries::key_hint(k); // will get captured by event.rs map_window
            }
        } else if let Some(key_hint) = state.workspace().key_hint_windows.get(k) {
            if let Err(e) = state.conn.destroy_window(*key_hint) {
                log::error!("windows::audit_key_hints(..) destroy error: {:?}", e);
            }
            if let Err(e) = state.conn.flush() {
                log::error!("windows::audit_key_hints(..) flush error: {:?}", e);
            }
            let _ = state.mut_workspace().key_hint_windows.remove(k);
        }
    }
}

// one split box per side area on the current monitor: spare ones stay unmapped for reuse
pub fn split_boxes(state: &mut crate::state::State, positions: &[(i32, i32)]) {
    if !state.settings.layout.split_box {
        return;
    }
    let index = state.current_monitor;
    let split_boxes: Vec<Window> = state.monitors[index]
        .split_boxes
        .iter()
        .copied()
        .filter(|w| crate::safety::window_exists(state, *w))
        .collect();
    state.monitors[index].split_boxes = split_boxes.clone();
    // will get captured by event.rs map_window
    while state.monitors[index].split_boxes.len() + state.monitors[index].split_boxes_pending
        < positions.len()
    {
        state.monitors[index].split_boxes_pending += 1;
        crate::binaries::split_box();
    }
    for (i, split_box) in split_boxes.iter().enumerate() {
        if i < positions.len() {
            if let Err(e) = state.conn.configure_window(
                *split_box,
                &ConfigureWindowAux::new()
                    .x(positions[i].0)
                    .y(positions[i].1)
                    .width(crate::windows::layout::BOX_SIZE as u32)
                    .height(crate::windows::layout::BOX_SIZE as u32),
            ) {
                log::error!("windows::audit_split_boxes(..) move window error: {:?}", e);
            }
            if let Err(e) = state.conn.map_window(*split_box) {
                log::error!("windows::audit_split_boxes(..) map window error: {:?}", e);
            }
            if !state.workspace().is_fullscreen()
                && let Err(e) = state.conn.configure_window(
                    *split_box,
                    &ConfigureWindowAux::new().stack_mode(StackMode::ABOVE),
                )
            {
                log::error!("windows::audit_split_boxes(..) raise window error: {:?}", e);
            }
        } else if let Err(e) = state.conn.unmap_window(*split_box) {
            log::error!("windows::audit_split_boxes(..) unmap window error: {:?}", e);
        }
    }
    if let Err(e) = state.conn.flush() {
        log::error!("windows::audit_split_boxes(..) flush error: {:?}", e);
    }
}

pub fn side_windows(state: &mut crate::state::State) {
    let mut change = false;
    for area in state.workspace().side_areas.clone() {
        for window in area {
            if !crate::safety::window_exists(state, window) {
                crate::windows::core::remove_side_window(state, window);
                change = true;
            }
        }
    }
    if change {
        crate::windows::layout::layout_side_space(state);
    }
}

pub fn main_space(state: &mut crate::state::State) {
    for main in state.workspace().main_area.clone() {
        if !crate::safety::window_exists(state, main) {
            crate::windows::audits::side_windows(state);
            // same path as destroy / unmap: shrinks the main area (promoting side #0 once empty)
            state.all_windows.remove(&main);
            crate::windows::core::release_window(state, main);
        }
    }
}

pub fn full(state: &mut crate::state::State) {
    crate::windows::audits::main_space(state);
    crate::windows::audits::side_windows(state);
    crate::ewmh::update_client_list(state);
    crate::ewmh::desktop_assignments(state);
}
