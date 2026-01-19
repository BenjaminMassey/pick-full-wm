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
                        .y(positions[i].1),
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
        reapply_float_windows(state);
    }
    if let Err(e) = state.conn.flush() {
        eprintln!("windows::focus_main(..) flush error: {:?}", e);
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
                for window in &monitor.workspaces[index].floatings {
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
                for window in &monitor.workspaces[index].floatings {
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

pub fn is_popup(state: &crate::state::State, window: Window) -> bool {
    // Check 1: Override-redirect windows (menus, tooltips)
    if let Ok(attrs) = state.conn.get_window_attributes(window) {
        if let Ok(attrs_reply) = attrs.reply() {
            if attrs_reply.override_redirect {
                return true;
            }
        }
    }

    // Check 2: Transient windows (dialogs with parent)
    if let Ok(prop) = state.conn.get_property(
        false,
        window,
        AtomEnum::WM_TRANSIENT_FOR,
        AtomEnum::WINDOW,
        0,
        1,
    ) {
        if let Ok(reply) = prop.reply() {
            if reply.value_len > 0 {
                return true;
            }
        }
    }

    // Check 3: Window type hints
    if let Ok(prop) = state.conn.get_property(
        false,
        window,
        state.atoms._NET_WM_WINDOW_TYPE,
        AtomEnum::ATOM,
        0,
        1024,
    ) {
        if let Ok(reply) = prop.reply() {
            if reply.value_len > 0 {
                // Parse as list of atoms
                let window_types: Vec<x11rb::protocol::xproto::Atom> = reply
                    .value32()
                    .map(|iter| iter.collect())
                    .unwrap_or_default();

                // Check if any type matches popup/dialog types
                for &window_type in &window_types {
                    if window_type == state.atoms._NET_WM_WINDOW_TYPE_DIALOG
                        || window_type == state.atoms._NET_WM_WINDOW_TYPE_UTILITY
                        || window_type == state.atoms._NET_WM_WINDOW_TYPE_SPLASH
                        || window_type == state.atoms._NET_WM_WINDOW_TYPE_NOTIFICATION
                        || window_type == state.atoms._NET_WM_WINDOW_TYPE_POPUP_MENU
                        || window_type == state.atoms._NET_WM_WINDOW_TYPE_DROPDOWN_MENU
                        || window_type == state.atoms._NET_WM_WINDOW_TYPE_TOOLTIP
                    {
                        return true;
                    }
                }
            }
        }
    }

    false
}

pub fn reapply_float_windows(state: &mut crate::state::State) {
    for window in state.workspace().floatings.clone() {
        if crate::safety::window_exists(state, window) {
            if let Err(e) = state.conn.configure_window(
                window,
                &ConfigureWindowAux::new().stack_mode(StackMode::ABOVE),
            ) {
                eprintln!(
                    "windows::reapply_float_windows(..) raise window error: {:?}",
                    e
                );
            }

            if let Err(e) = state.conn.flush() {
                eprintln!("windows::reapply_float_windows(..) flush error: {:?}", e);
            }
        } else {
            remove_floating(state, window);
            layout_side_space(state);
        }
    }
}

pub fn remove_floating(state: &mut crate::state::State, window: Window) {
    let mut removes: Vec<usize> = vec![];
    for (index, floating) in state.workspace().floatings.iter().enumerate() {
        if floating == &window {
            removes.push(index);
        }
    }
    for remove in removes {
        state.mut_workspace().floatings.remove(remove);
    }
}

pub fn center_window(state: &mut crate::state::State, window: Window) {
    if let Ok(geometry) = state.conn.get_geometry(window)
        && let Ok(geometry) = geometry.reply()
    {
        if let Err(e) = state.conn.configure_window(
            window,
            &ConfigureWindowAux::new()
                .x(state.monitor().position.0
                    + ((state.monitor().sizes.screen.0 - geometry.width as i32) as f32 / 2f32)
                        as i32)
                .y(state.monitor().position.1
                    + ((state.monitor().sizes.screen.1 - geometry.height as i32) as f32 / 2f32)
                        as i32),
        ) {
            eprintln!("windows::center_window(..) move window error: {:?}", e);
        }
    }

    if let Err(e) = state.conn.flush() {
        eprintln!("windows::center_window(..) flush error: {:?}", e);
    }
}

pub fn audit_side_windows(state: &mut crate::state::State) {
    let mut change = false;
    for window in state.workspace().side_windows.clone() {
        if let Some(window) = window
            && !crate::safety::window_exists(state, window)
        {
            remove_side_window(state, window);
            change = true;
        }
    }
    if change {
        layout_side_space(state);
    }
}

pub fn audit_main(state: &mut crate::state::State) {
    if let Some(main) = state.workspace().main_window
        && !crate::safety::window_exists(state, main)
    {
        state.mut_workspace().main_window = None;
        audit_side_windows(state);
        if !state.workspace().side_windows.is_empty()
            && let Some(target) = state.workspace().side_windows[0]
        {
            fill_main_space(state, target);
            remove_side_window(state, target);
        }
    }
}

pub fn full_audit(state: &mut crate::state::State) {
    audit_main(state);
    audit_side_windows(state);
    crate::ewmh::update_client_list(state);
}

pub fn get_monitor_index(state: &crate::state::State, window: Window) -> usize {
    for (monitor_index, monitor) in state.monitors.iter().enumerate() {
        for workspace in &monitor.workspaces {
            if let Some(main) = workspace.main_window
                && main == window
            {
                return monitor_index;
            }
            for side_window in &workspace.side_windows {
                if let Some(side) = side_window
                    && side == &window
                {
                    return monitor_index;
                }
            }
            for floating in &workspace.floatings {
                if floating == &window {
                    return monitor_index;
                }
            }
            for (_, key_window) in &workspace.key_hint_windows {
                if key_window == &window {
                    return monitor_index;
                }
            }
            if let Some(help) = workspace.help_window
                && help == window
            {
                return monitor_index;
            }
        }
    }
    0
}
