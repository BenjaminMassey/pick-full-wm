use x11rb::connection::Connection;
use x11rb::protocol::xproto::{AtomEnum, ClientMessageEvent, ConnectionExt, EventMask, Window};

pub fn fill_main_space(state: &mut crate::state::State, area: Vec<Window>) {
    log::info!("fill_main_space {:?}", area);
    state.all_windows.extend(&area);
    state.mut_workspace().main_area = area.clone();
    if state.workspace().is_fullscreen() {
        crate::windows::layout::fullscreen(state, &area);
        return;
    }
    let (x, width) = crate::windows::layout::main_geometry(
        state.monitor(),
        &state.settings,
        state.workspace().side_areas.len(),
    );

    crate::windows::layout::split_area(
        state,
        &area,
        (x, state.monitor().position.1),
        (width as u32, state.monitor().sizes.main.1 as u32),
    );

    if let Err(e) = state.conn.flush() {
        log::error!("windows::fill_main_space(..) flush error: {:?}", e);
    }

    focus_main(state);
}

pub fn send_side_space(state: &mut crate::state::State, area: Vec<Window>, index: Option<usize>) {
    for window in &area {
        remove_side_window(state, *window);
    }
    state.all_windows.extend(&area);
    if !area.is_empty() {
        let side_areas = &mut state.mut_workspace().side_areas;
        if let Some(index) = index {
            side_areas.insert(index.min(side_areas.len()), area);
        } else {
            side_areas.push(area);
        }
    }
    crate::windows::layout::layout_side_space(state);
}

// drop one window from whichever side area holds it (dropping the area once empty)
pub fn remove_side_window(state: &mut crate::state::State, window: Window) -> usize {
    let side_areas = &mut state.mut_workspace().side_areas;
    let index = side_areas.iter().position(|a| a.contains(&window));
    for area in side_areas.iter_mut() {
        area.retain(|w| *w != window);
    }
    side_areas.retain(|a| !a.is_empty());
    index.unwrap_or(side_areas.len())
}

// pull the whole side area at index out (empty if there isn't one)
pub fn take_side_area(state: &mut crate::state::State, index: usize) -> Vec<Window> {
    let side_areas = &mut state.mut_workspace().side_areas;
    if index < side_areas.len() {
        side_areas.remove(index)
    } else {
        vec![]
    }
}

// bring the side area at index to main: joining the main area if a split is queued,
// swapping places with it otherwise
pub fn swap_side_area(state: &mut crate::state::State, index: usize, focus: Option<Window>) {
    let area = take_side_area(state, index);
    if area.is_empty() {
        return;
    }
    let existing = state.workspace().main_area.clone();
    if state.split_queued && !existing.is_empty() {
        log::info!("Splitting side area #{} into main.", index);
        state.split_queued = false;
        state.mut_workspace().main_focus = focus.or(area.first().copied());
        let mut joined = existing;
        joined.extend(area);
        fill_main_space(state, joined);
        crate::windows::layout::layout_side_space(state);
        return;
    }
    state.split_queued = false;
    state.mut_workspace().main_focus = focus;
    fill_main_space(state, area);
    if !existing.is_empty() {
        if state.settings.layout.swap_not_stack {
            send_side_space(state, existing, Some(index));
        } else {
            send_side_space(state, existing, None);
        }
    }
}

// move the keyboard to the next window of a split main area: left to right, looping
pub fn cycle_split(state: &mut crate::state::State) {
    let main_area = state.workspace().main_area.clone();
    if main_area.len() < 2 {
        return;
    }
    let index = main_area
        .iter()
        .position(|w| Some(*w) == state.workspace().focused_main())
        .unwrap_or(0);
    let next = main_area[(index + 1) % main_area.len()];
    log::info!("Cycling split focus to main window {}.", next);
    state.mut_workspace().main_focus = Some(next);
    focus_main(state);
}

// focus follows the mouse within the main area only: side windows never take the keyboard
pub fn focus_hovered(state: &mut crate::state::State, window: Window) {
    if !state.workspace().main_area.contains(&window)
        || state.workspace().focused_main() == Some(window)
    {
        return;
    }
    log::info!("Pointer entered main window {}: focusing it.", window);
    state.mut_workspace().main_focus = Some(window);
    focus_main(state);
}

pub fn focus_main(state: &mut crate::state::State) {
    if let Some(window) = state.workspace().focused_main()
        && crate::safety::window_exists(state, window)
    {
        crate::ewmh::set_active(state, window);
        crate::windows::layout::reapply_float_windows(state);
        // an open dialog keeps the keyboard over its main window
        if let Some(floating) = focus_floating(state) {
            crate::ewmh::set_active(state, floating);
        }
    }
    if let Err(e) = state.conn.flush() {
        log::error!("windows::focus_main(..) flush error: {:?}", e);
    }
    crate::windows::layout::place_close_boxes(state);
    crate::windows::layout::place_monitor_boxes(state);
    crate::windows::layout::raise_docks(state);
}

// newest floating window on this workspace that should take keyboard focus
pub fn focus_floating(state: &crate::state::State) -> Option<Window> {
    state.workspace().floatings.iter().rev().copied().find(|w| {
        crate::safety::window_exists(state, *w) && crate::windows::checks::takes_focus(state, *w)
    })
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
            if workspace.main_area.contains(&window)
                || workspace.side_index(window).is_some()
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
        if is_visible && let Some(main_window) = state.workspace().focused_main() {
            crate::ewmh::set_active(state, main_window);
            if let Err(e) = state.conn.flush() {
                log::error!("windows::release_window(..) flush error: {:?}", e);
            }
        }
    } else {
        if state.workspace().main_area.contains(&window) {
            // the rest of a split main area stays: only an emptied one takes side #0
            state.mut_workspace().main_area.retain(|w| *w != window);
            if state.workspace().main_area.is_empty() && !state.workspace().side_areas.is_empty() {
                let target = take_side_area(state, 0);
                if is_visible {
                    fill_main_space(state, target);
                } else {
                    // hidden: placed by workspaces::switch(..) later, don't steal focus
                    state.mut_workspace().main_area = target;
                }
            }
        } else {
            remove_side_window(state, window);
        }
        if is_visible {
            crate::windows::layout::layout_side_space(state);
            if state.workspace().main_area.is_empty() {
                crate::ewmh::clear_active(state);
            }
        }
    }
    state.current_monitor = real_current_monitor; // TODO: gross temp set
    state.current_workspace = real_current_workspace; // TODO: gross temp set
    crate::windows::layout::place_close_boxes(state);
    crate::windows::layout::place_monitor_boxes(state);
}
