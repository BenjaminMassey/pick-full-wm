use x11rb::connection::Connection;
use x11rb::protocol::xproto::ConnectionExt;

pub fn switch(state: &mut crate::state::State) {
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
            crate::windows::core::fill_main_space(state, main);
        }
        crate::windows::layout::layout_side_space(state);
        state.current_monitor = real_monitor; // TODO: shouldn't need temp like this
    }
    crate::windows::core::focus_main(state);
}
