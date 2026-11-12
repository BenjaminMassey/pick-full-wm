use x11rb::connection::Connection;
use x11rb::protocol::xproto::{ConnectionExt, Window};

// Live settings reload (SUPER + SHIFT + R by default): re-reads the settings file and
// re-applies everything the running window manager can change while leaving every window
// open and in the workspace slot it was already in.
pub fn settings(state: &mut crate::state::State) {
    let (new_settings, warnings) = crate::settings::load_settings();
    for warning in warnings {
        log::warn!("{}", warning);
    }
    state.settings = new_settings;
    log::info!("Reloading settings.");

    apply_monitors(state);
    apply_helper_boxes(state);
    clear_key_hints(state);
    crate::setup::input::grab_keys(state);
    crate::ewmh::publish_desktops(state);
    // switch(..) re-maps, re-positions, and re-focuses everything with the new layout values
    crate::windows::workspaces::switch(state);
    log::info!("Finished with settings reload.");
}

// re-derive each monitor's sizes and origin from the new layout entries, keeping its windows
fn apply_monitors(state: &mut crate::state::State) {
    let mut monitor_infos = crate::state::get_monitor_infos(&state.conn, state.root);
    monitor_infos.sort_by_key(|m| m.position.0);
    if monitor_infos.is_empty() {
        log::warn!("No monitors found during reload: leaving monitor layout unchanged.");
        return;
    }

    let mut old_monitors = std::mem::take(&mut state.monitors);
    let workspace_count = state.settings.bindings.workspaces.len();

    for (index, monitor_info) in monitor_infos.iter().enumerate() {
        let mut warnings: Vec<String> = vec![];
        let mut monitor =
            crate::state::Monitor::new(&state.settings, monitor_info, index, &mut warnings);
        for warning in warnings {
            log::warn!("{}", warning);
        }
        if index < old_monitors.len() {
            // helper windows keep running where they are: reload only re-places them
            monitor.close_box = old_monitors[index].close_box;
            monitor.monitor_box = old_monitors[index].monitor_box;
            let old_workspaces = std::mem::take(&mut old_monitors[index].workspaces);
            if workspace_count > 0 {
                merge_workspaces(&mut monitor.workspaces, &old_workspaces, workspace_count);
            } else {
                // an empty workspaces list would leave the windows nowhere: keep what's running
                log::warn!("No bindings.workspaces entries: keeping the current workspace count.");
                monitor.workspaces = old_workspaces;
            }
        }
        state.monitors.push(monitor);
    }

    state.current_monitor = state.current_monitor.min(state.monitors.len() - 1);
    state.current_workspace = state
        .current_workspace
        .min(state.monitor().workspaces.len() - 1);
}

// fill the new workspaces by index, folding any extra old workspaces into the last one
fn merge_workspaces(
    new_workspaces: &mut [crate::state::Workspace],
    old_workspaces: &[crate::state::Workspace],
    count: usize,
) {
    for (index, old) in old_workspaces.iter().enumerate() {
        let target = &mut new_workspaces[index.min(count - 1)];
        if target.main_window.is_none() {
            target.main_window = old.main_window;
            target.fullscreen = old.fullscreen;
        } else if let Some(main) = old.main_window {
            target.side_windows.push(Some(main));
        }
        target.side_windows.extend(old.side_windows.iter().copied());
        target.floatings.extend(old.floatings.iter().copied());
        if target.help_window.is_none() {
            target.help_window = old.help_window;
        }
        for (key, window) in &old.key_hint_windows {
            target.key_hint_windows.insert(key.clone(), *window);
        }
    }
}

// spawn or dismiss the close / monitor boxes to match the new layout settings
fn apply_helper_boxes(state: &mut crate::state::State) {
    for index in 0..state.monitors.len() {
        let close_box = state.monitors[index].close_box;
        if state.settings.layout.close_box {
            if close_box.is_none() {
                crate::binaries::close_box();
            }
        } else if let Some(window) = close_box {
            state.monitors[index].close_box = None;
            dismiss_box(state, window, "close");
        }

        let monitor_box = state.monitors[index].monitor_box;
        if state.settings.layout.monitor_box && state.monitors.len() > 1 {
            if monitor_box.is_none() {
                crate::binaries::monitor_box();
            }
        } else if let Some(window) = monitor_box {
            state.monitors[index].monitor_box = None;
            dismiss_box(state, window, "monitor");
        }
    }
}

fn dismiss_box(state: &mut crate::state::State, window: Window, name: &str) {
    if crate::safety::window_exists(state, window) {
        log::info!("Dismissing {} box window {}.", name, window);
        crate::windows::core::kill_window(state, window);
    }
}

// hint windows are re-spawned by the audit from the new swap keys, so drop them all first
fn clear_key_hints(state: &mut crate::state::State) {
    let mut stale: Vec<Window> = vec![];
    for monitor in &state.monitors {
        for workspace in &monitor.workspaces {
            for window in workspace.key_hint_windows.values() {
                if *window != 0 {
                    stale.push(*window);
                }
            }
        }
    }
    for monitor in &mut state.monitors {
        for workspace in &mut monitor.workspaces {
            workspace.key_hint_windows.clear();
        }
    }
    for window in stale {
        if crate::safety::window_exists(state, window)
            && let Err(e) = state.conn.destroy_window(window)
        {
            log::error!("reload::clear_key_hints(..) destroy window error: {:?}", e);
        }
    }
    if let Err(e) = state.conn.flush() {
        log::error!("reload::clear_key_hints(..) flush error: {:?}", e);
    }
}
