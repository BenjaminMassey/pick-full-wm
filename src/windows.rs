use x11rb::connection::Connection;
use x11rb::protocol::xproto::{AtomEnum, ConfigureWindowAux, ConnectionExt, StackMode, Window};

pub fn fill_main_space(state: &mut crate::state::State, window: Window) {
    let width =
        if state.settings.layout.conditional_full && state.workspace().side_windows.is_empty() {
            state.monitor().sizes.screen.0
        } else {
            state.monitor().sizes.main.0
        };

    if let Err(e) = state.conn.configure_window(
        window,
        &ConfigureWindowAux::new()
            .x(state.monitor().position.0)
            .y(state.monitor().position.1)
            .width(width as u32)
            .height(state.monitor().sizes.main.1 as u32),
    ) {
        eprintln!("windows::fill_main_space(..) move window error: {:?}", e);
    }

    if let Err(e) = state.conn.flush() {
        eprintln!("windows::fill_main_space(..) flush error: {:?}", e);
    }

    state.mut_workspace().main_window = Some(window);
    focus_main(state);
}

pub fn send_side_space(state: &mut crate::state::State, window: Window) {
    remove_side_window(state, window);
    state.mut_workspace().side_windows.push(Some(window));
    layout_side_space(state);
}

pub fn remove_side_window(state: &mut crate::state::State, window: Window) {
    let mut removes: Vec<usize> = vec![];
    for (index, side_window) in state.workspace().side_windows.iter().enumerate() {
        if side_window.is_none() || side_window.unwrap() == window {
            removes.push(index);
        }
    }
    for index in removes {
        state.mut_workspace().side_windows.remove(index);
    }
}

pub fn layout_side_space(state: &mut crate::state::State) {
    let mut positions: Vec<(i32, i32)> = vec![];
    let section_size =
        state.monitor().sizes.side.1 as f32 / state.workspace().side_windows.len() as f32;
    for (index, window) in state.workspace().side_windows.iter().enumerate() {
        if let Some(window) = window {
            let section_pos = section_size * index as f32;
            println!(
                "layout_side_space {} {},{} {}x{}",
                window,
                state.monitor().sizes.main.0,
                section_pos as i32,
                state.monitor().sizes.side.0 as u32,
                section_size as u32,
            );
            let position = (
                state.monitor().position.0 + state.monitor().sizes.main.0,
                state.monitor().position.1 + section_pos as i32,
            );
            positions.push(position);

            if let Err(e) = state.conn.configure_window(
                *window,
                &ConfigureWindowAux::new()
                    .x(position.0)
                    .y(position.1)
                    .width(state.monitor().sizes.side.0 as u32) // TODO: take into account x offset
                    .height(section_size as u32),
            ) {
                eprintln!("windows::layout_side_space(..) move window error: {:?}", e);
            }
        }
    }
    if let Err(e) = state.conn.flush() {
        eprintln!("windows::fill_main_space(..) flush error: {:?}", e);
    }
    audit_key_hints(state, &positions);
    if let Some(main) = state.workspace().main_window {
        fill_main_space(state, main);
    }
}

fn audit_key_hints(state: &mut crate::state::State, positions: &[(i32, i32)]) {
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
                        .width(50) // TODO: connect to src/bin/key_hint.rs sizing directly
                        .height(50), // TODO: connect to src/bin/key_hint.rs sizing directly
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
                crate::binaries::key_hint(k);
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

pub fn fullscreen(state: &mut crate::state::State, window: Window) {
    if let Err(e) = state.conn.configure_window(
        window,
        &ConfigureWindowAux::new()
            .x(state.monitor().position.0
                + ((state.monitor().sizes.main.0 + state.monitor().sizes.side.0)
                    - state.monitor().sizes.screen.0))
            .y(state.monitor().position.1
                + (state.monitor().sizes.main.1 - state.monitor().sizes.screen.1))
            .width(state.monitor().sizes.screen.0 as u32)
            .height(state.monitor().sizes.screen.1 as u32),
    ) {
        eprintln!("windows::fullscreen(..) move window error: {:?}", e);
    }

    if let Err(e) = state.conn.flush() {
        eprintln!("windows::fullscreen(..) flush error: {:?}", e);
    }
    focus_main(state);
}

pub fn is_excepted_window(state: &mut crate::state::State, window: Window) -> bool {
    if is_help_window(state, window) || get_key_hint_window(state, window).is_some() {
        return true;
    }
    if let Some(name) = get_window_name(state, window) {
        println!("window name: {}", &name);
        for exception in &state.settings.applications.excluded {
            if name.contains(exception) {
                return true;
            }
        }
    }
    false
}

pub fn is_help_window(state: &mut crate::state::State, window: Window) -> bool {
    if let Some(name) = get_window_name(state, window)
        && name.contains("pfwm help")
    {
        return true;
    }
    false
}

pub fn get_key_hint_window(state: &mut crate::state::State, window: Window) -> Option<String> {
    if let Some(name) = get_window_name(state, window)
        && name.contains("key_hint")
    {
        let pieces: Vec<&str> = name.split(' ').collect();
        if pieces.len() == 2 {
            return Some(pieces[1].to_owned());
        }
    }
    None
}

pub fn get_window_name(state: &mut crate::state::State, window: Window) -> Option<String> {
    let reply = state
        .conn
        .get_property(false, window, AtomEnum::WM_NAME, AtomEnum::STRING, 0, 1024)
        .ok()?
        .reply()
        .ok()?;

    if reply.value.is_empty() {
        return None;
    }

    String::from_utf8(reply.value).ok()
}

pub fn run_command(command: &str) {
    match std::process::Command::new("sh")
        .arg("-c")
        .arg(command)
        .spawn()
    {
        Ok(_) => println!("Run command: \"{}\"", command),
        Err(e) => eprintln!("Failed to run command \"{}\": {}", command, e),
    };
}

pub fn focus_main(state: &mut crate::state::State) {
    if let Some(window) = state.workspace().main_window
        && crate::safety::window_exists(state, window)
    {
        crate::ewmh::set_active(state, window);
    }
}

pub fn switch_workspace(state: &mut crate::state::State) {
    for monitor in &state.monitors {
        for index in 0..monitor.workspaces.len() {
            if index == state.current_workspace {
                if let Some(main) = monitor.workspaces[index].main_window {
                    if let Err(e) = state.conn.map_window(main) {
                        eprintln!("windows::switch_workspace(..) map window error: {:?}", e);
                    }
                }
                for window in &monitor.workspaces[index].side_windows {
                    if let Some(window) = window {
                        if let Err(e) = state.conn.map_window(*window) {
                            eprintln!("windows::switch_workspace(..) map window error: {:?}", e);
                        }
                    }
                }
                if let Some(help) = monitor.workspaces[index].help_window {
                    if let Err(e) = state.conn.map_window(help) {
                        eprintln!("windows::switch_workspace(..) map window error: {:?}", e);
                    }
                }
                for (_, window) in &monitor.workspaces[index].key_hint_windows {
                    if let Err(e) = state.conn.map_window(*window) {
                        eprintln!("windows::switch_workspace(..) map window error: {:?}", e);
                    }
                }
            } else {
                if let Some(main) = monitor.workspaces[index].main_window {
                    if let Err(e) = state.conn.unmap_window(main) {
                        eprintln!("windows::switch_workspace(..) unmap window error: {:?}", e);
                    }
                }
                for window in &monitor.workspaces[index].side_windows {
                    if let Some(window) = window {
                        if let Err(e) = state.conn.unmap_window(*window) {
                            eprintln!("windows::switch_workspace(..) unmap window error: {:?}", e);
                        }
                    }
                }
                if let Some(help) = monitor.workspaces[index].help_window {
                    if let Err(e) = state.conn.unmap_window(help) {
                        eprintln!("windows::switch_workspace(..) unmap window error: {:?}", e);
                    }
                }
                for (_, window) in &monitor.workspaces[index].key_hint_windows {
                    if let Err(e) = state.conn.unmap_window(*window) {
                        eprintln!("windows::switch_workspace(..) unmap window error: {:?}", e);
                    }
                }
            }
        }
    }
    if let Err(e) = state.conn.flush() {
        eprintln!("windows::switch_workspace(..) flush error: {:?}", e);
    }
    crate::ewmh::update_workspace(state);
    crate::ewmh::clear_active(state);
    for i in 0..state.monitors.len() {
        let real_monitor = state.current_monitor; // TODO: shouldn't need temp like this
        state.current_monitor = i; // TODO: shouldn't need temp like this
        if let Some(main) = state.workspace().main_window {
            fill_main_space(state, main);
        }
        layout_side_space(state);
        state.current_monitor = real_monitor; // TODO: shouldn't need temp like this
    }
    crate::windows::focus_main(state);
}
