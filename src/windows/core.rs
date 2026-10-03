use x11rb::connection::Connection;
use x11rb::protocol::xproto::{
    AtomEnum, ClientMessageEvent, ConfigureWindowAux, ConnectionExt, EventMask, Window,
};

pub fn fill_main_space(state: &mut crate::state::State, window: Window) {
    log::info!("fill_main_space {}", window);
    if state.workspace().fullscreen == Some(window) {
        state.mut_workspace().main_window = Some(window);
        state.all_windows.insert(window);
        crate::windows::layout::fullscreen(state, window);
        return;
    }
    let side = &state.settings.layout.side_orientation;
    let fill = state.settings.layout.conditional_full;
    let width = if side == "both" {
        if fill {
            if state.workspace().side_windows.is_empty() {
                state.monitor().sizes.screen.0
            } else if state.workspace().side_windows.len() == 1 {
                state.monitor().sizes.main.0
                    + (state.monitor().sizes.side.0 as f32 * 0.5f32).floor() as i32
            } else {
                state.monitor().sizes.main.0
            }
        } else {
            state.monitor().sizes.main.0 // - state.monitor().sizes.side.0
        }
    } else {
        if fill && state.workspace().side_windows.is_empty() {
            state.monitor().sizes.screen.0
        } else {
            state.monitor().sizes.main.0
        }
    };

    if let Err(e) = state.conn.configure_window(
        window,
        &ConfigureWindowAux::new()
            .x(
                if side == "right" || (fill && state.workspace().side_windows.is_empty()) {
                    state.monitor().position.0
                } else {
                    if side == "both" {
                        state.monitor().position.0
                            + (state.monitor().sizes.side.0 as f32 * 0.5f32).floor() as i32
                    } else {
                        state.monitor().sizes.side.0 + state.monitor().position.0
                    }
                },
            )
            .y(state.monitor().position.1)
            .width(width as u32)
            .height(state.monitor().sizes.main.1 as u32),
    ) {
        log::error!("windows::fill_main_space(..) move window error: {:?}", e);
    }

    if let Err(e) = state.conn.flush() {
        log::error!("windows::fill_main_space(..) flush error: {:?}", e);
    }

    state.mut_workspace().main_window = Some(window);
    focus_main(state);
    state.all_windows.insert(window);
}

pub fn send_side_space(state: &mut crate::state::State, window: Window, index: Option<usize>) {
    remove_side_window(state, window);
    if let Some(index) = index {
        state
            .mut_workspace()
            .side_windows
            .insert(index, Some(window));
    } else {
        state.mut_workspace().side_windows.push(Some(window));
    }
    state.all_windows.insert(window);
    crate::windows::layout::layout_side_space(state);
}

pub fn remove_side_window(state: &mut crate::state::State, window: Window) -> usize {
    let side_windows = &mut state.mut_workspace().side_windows;
    side_windows.retain(|w| w.is_some());
    let index = side_windows.iter().position(|w| *w == Some(window));
    side_windows.retain(|w| *w != Some(window));
    index.unwrap_or(side_windows.len())
}

pub fn focus_main(state: &mut crate::state::State) {
    if let Some(window) = state.workspace().main_window
        && crate::safety::window_exists(state, window)
    {
        crate::ewmh::set_active(state, window);
        crate::windows::layout::reapply_float_windows(state);
    }
    if let Err(e) = state.conn.flush() {
        log::error!("windows::focus_main(..) flush error: {:?}", e);
    }
    crate::windows::layout::place_close_boxes(state);
    crate::windows::layout::place_monitor_boxes(state);
}

pub fn remove_floating(state: &mut crate::state::State, window: Window) {
    state.mut_workspace().floatings.retain(|w| *w != window);
}

// politely ask via WM_DELETE_WINDOW if supported, otherwise kill the client
pub fn close_window(state: &mut crate::state::State, window: Window) {
    let supports_delete = match state.conn.get_property(
        false,
        window,
        state.atoms.WM_PROTOCOLS,
        AtomEnum::ATOM,
        0,
        1024,
    ) {
        Ok(cookie) => match cookie.reply() {
            Ok(reply) => reply
                .value32()
                .is_some_and(|mut atoms| atoms.any(|a| a == state.atoms.WM_DELETE_WINDOW)),
            Err(_) => false,
        },
        Err(_) => false,
    };
    if supports_delete {
        log::info!("Sending WM_DELETE_WINDOW to window {}.", window);
        let event = ClientMessageEvent::new(
            32,
            window,
            state.atoms.WM_PROTOCOLS,
            [state.atoms.WM_DELETE_WINDOW, x11rb::CURRENT_TIME, 0, 0, 0],
        );
        if let Err(e) = state
            .conn
            .send_event(false, window, EventMask::NO_EVENT, event)
        {
            log::error!("windows::close_window(..) send event error: {:?}", e);
        }
        if let Err(e) = state.conn.flush() {
            log::error!("windows::close_window(..) flush error: {:?}", e);
        }
    } else {
        kill_window(state, window);
    }
}

// forcefully disconnect the window's client (for hung apps)
pub fn kill_window(state: &mut crate::state::State, window: Window) {
    log::info!("Killing client of window {}.", window);
    if let Err(e) = state.conn.kill_client(window) {
        log::error!("windows::kill_window(..) kill client error: {:?}", e);
    }
    if let Err(e) = state.conn.flush() {
        log::error!("windows::kill_window(..) flush error: {:?}", e);
    }
}

// (monitor, workspace) holding the window as main, side, floating, or help
pub fn find_window(state: &crate::state::State, window: Window) -> Option<(usize, usize)> {
    for (monitor_index, monitor) in state.monitors.iter().enumerate() {
        for (workspace_index, workspace) in monitor.workspaces.iter().enumerate() {
            if workspace.main_window == Some(window)
                || workspace.side_windows.contains(&Some(window))
                || workspace.floatings.contains(&window)
                || workspace.help_window == Some(window)
            {
                return Some((monitor_index, workspace_index));
            }
        }
    }
    None
}

// drop a gone (destroyed / hidden) window from its slot and fix up the layout
pub fn release_window(state: &mut crate::state::State, window: Window) {
    for monitor in &mut state.monitors {
        for workspace in &mut monitor.workspaces {
            if workspace.fullscreen == Some(window) {
                workspace.fullscreen = None;
            }
        }
    }
    let Some((monitor_index, workspace_index)) = find_window(state, window) else {
        return; // not tracked (menus, tooltips, key hints, etc): nothing to re-layout
    };
    let is_visible = workspace_index == state.current_workspace;
    let real_current_monitor = state.current_monitor; // TODO: gross temp set
    let real_current_workspace = state.current_workspace; // TODO: gross temp set
    state.current_monitor = monitor_index; // TODO: gross temp set
    state.current_workspace = workspace_index; // TODO: gross temp set
    if state.workspace().floatings.contains(&window) {
        remove_floating(state, window);
        if is_visible {
            focus_main(state);
        }
    } else if state.workspace().help_window == Some(window) {
        state.mut_workspace().help_window = None;
        if is_visible && let Some(main_window) = state.workspace().main_window {
            crate::ewmh::set_active(state, main_window);
            if let Err(e) = state.conn.flush() {
                log::error!("windows::release_window(..) flush error: {:?}", e);
            }
        }
    } else {
        if state.workspace().main_window == Some(window) {
            if !state.workspace().side_windows.is_empty()
                && let Some(target) = state.workspace().side_windows[0]
            {
                remove_side_window(state, target);
                if is_visible {
                    fill_main_space(state, target);
                } else {
                    // hidden: placed by workspaces::switch(..) later, don't steal focus
                    state.mut_workspace().main_window = Some(target);
                }
            } else {
                state.mut_workspace().main_window = None;
            }
        } else {
            remove_side_window(state, window);
        }
        if is_visible {
            crate::windows::layout::layout_side_space(state);
            if state.workspace().main_window.is_none() {
                crate::ewmh::clear_active(state);
            }
        }
    }
    state.current_monitor = real_current_monitor; // TODO: gross temp set
    state.current_workspace = real_current_workspace; // TODO: gross temp set
    crate::windows::layout::place_close_boxes(state);
    crate::windows::layout::place_monitor_boxes(state);
}
