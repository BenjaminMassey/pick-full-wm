use x11rb::CURRENT_TIME;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::{Allow, ConnectionExt, DestroyNotifyEvent, MapRequestEvent};

pub fn map_request(state: &mut crate::state::State, event: MapRequestEvent) {
    println!(
        "Map Window: {:?} ({})",
        crate::windows::gets::window_name(state, event.window),
        event.window,
    );
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
    if let Some(key) = crate::windows::gets::key_hint_window(state, event.window) {
        if let Some(entry) = state.mut_workspace().key_hint_windows.get_mut(&key) {
            let old_key = entry.clone();
            *entry = event.window;
            if old_key != event.window && crate::safety::window_exists(state, old_key) {
                if let Err(e) = state.conn.destroy_window(old_key) {
                    eprintln!("events::map_request(..) destroy window error: {:?}", e);
                }
                if let Err(e) = state.conn.flush() {
                    eprintln!("events::map_request(..) flush error: {:?}", e);
                }
            }
        } else {
            state
                .mut_workspace()
                .key_hint_windows
                .insert(key, event.window);
        }
        crate::windows::layout::layout_side_space(state);
        return;
    }
    if crate::windows::checks::is_help_window(state, event.window) {
        crate::ewmh::set_active(state, event.window);
        if let Err(e) = state.conn.flush() {
            eprintln!("events::map_request(..) flush error: {:?}", e);
        }
        state.mut_workspace().help_window = Some(event.window);
        return;
    }
    if crate::windows::checks::is_excepted_window(state, event.window) {
        return;
    }
    if crate::windows::checks::is_popup(state, event.window) {
        state.mut_workspace().floatings.push(event.window);
        crate::windows::layout::center_window(state, event.window);
        return;
    }
    if let Some(main) = state.workspace().main_window {
        if state.settings.layout.new_to_main {
            crate::windows::core::send_side_space(state, main);
            crate::windows::core::fill_main_space(state, event.window);
        } else {
            crate::windows::core::send_side_space(state, event.window);
        }
    } else {
        crate::windows::core::fill_main_space(state, event.window);
    }
}

pub fn destroy(state: &mut crate::state::State, event: DestroyNotifyEvent) {
    println!(
        "Destroy Window: {:?} ({})",
        crate::windows::gets::window_name(state, event.window),
        event.window,
    );
    for i in 0..state.monitor().workspaces.len() {
        if state.monitor().workspaces[i]
            .floatings
            .contains(&event.window)
        {
            crate::windows::core::remove_floating(state, event.window);
            crate::windows::core::focus_main(state);
            return;
        }
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
                        crate::windows::core::remove_side_window(state, target);
                        crate::windows::core::fill_main_space(state, target);
                    }
                } else {
                    state.mut_workspace().main_window = None;
                }
            } else {
                crate::windows::core::remove_side_window(state, event.window);
            }
        }
        state.current_workspace = real_workspace; // TODO: gross, for windows.rs calls
    }
    crate::windows::layout::layout_side_space(state);
    if state.workspace().main_window.is_none() {
        crate::ewmh::clear_active(state);
    }
}
