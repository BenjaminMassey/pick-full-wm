use x11rb::connection::Connection;
use x11rb::protocol::xproto::ConnectionExt;

pub fn switch(state: &mut crate::state::State) {
    for monitor in &state.monitors {
        for index in 0..monitor.workspaces.len() {
            if index == state.current_workspace {
                for main in &monitor.workspaces[index].main_area {
                    if let Err(e) = state.conn.map_window(*main) {
                        log::error!("windows::switch_workspace(..) map window error: {:?}", e);
                    }
                }
                for area in &monitor.workspaces[index].side_areas {
                    for window in area {
                        if let Err(e) = state.conn.map_window(*window) {
                            log::error!("windows::switch_workspace(..) map window error: {:?}", e);
                        }
                    }
                }
                if let Some(help) = monitor.workspaces[index].help_window {
                    if let Err(e) = state.conn.map_window(help) {
                        log::error!("windows::switch_workspace(..) map window error: {:?}", e);
                    }
                }
                for (_, window) in &monitor.workspaces[index].key_hint_windows {
                    if let Err(e) = state.conn.map_window(*window) {
                        log::error!("windows::switch_workspace(..) map window error: {:?}", e);
                    }
                }
                for window in &monitor.workspaces[index].floatings {
                    if let Err(e) = state.conn.map_window(*window) {
                        log::error!("windows::switch_workspace(..) map window error: {:?}", e);
                    }
                }
            } else {
                for main in &monitor.workspaces[index].main_area {
                    if let Err(e) = state.conn.unmap_window(*main) {
                        log::error!("windows::switch_workspace(..) unmap window error: {:?}", e);
                    }
                }
                for area in &monitor.workspaces[index].side_areas {
                    for window in area {
                        if let Err(e) = state.conn.unmap_window(*window) {
                            log::error!(
                                "windows::switch_workspace(..) unmap window error: {:?}",
                                e
                            );
                        }
                    }
                }
                if let Some(help) = monitor.workspaces[index].help_window {
                    if let Err(e) = state.conn.unmap_window(help) {
                        log::error!("windows::switch_workspace(..) unmap window error: {:?}", e);
                    }
                }
                for (_, window) in &monitor.workspaces[index].key_hint_windows {
                    if let Err(e) = state.conn.unmap_window(*window) {
                        log::error!("windows::switch_workspace(..) unmap window error: {:?}", e);
                    }
                }
                for window in &monitor.workspaces[index].floatings {
                    if let Err(e) = state.conn.unmap_window(*window) {
                        log::error!("windows::switch_workspace(..) unmap window error: {:?}", e);
                    }
                }
            }
        }
    }
    if let Err(e) = state.conn.flush() {
        log::error!("windows::switch_workspace(..) flush error: {:?}", e);
    }
    crate::ewmh::update_workspace(state);
    crate::ewmh::clear_active(state);
    state.split_queued = false;
    for i in 0..state.monitors.len() {
        let real_monitor = state.current_monitor; // TODO: shouldn't need temp like this
        state.current_monitor = i; // TODO: shouldn't need temp like this
        let main = state.workspace().main_area.clone();
        if !main.is_empty() {
            crate::windows::core::fill_main_space(state, main);
        }
        crate::windows::layout::layout_side_space(state);
        state.current_monitor = real_monitor; // TODO: shouldn't need temp like this
    }
    crate::windows::core::focus_main(state);
}
