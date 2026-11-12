use x11rb::connection::Connection;
use x11rb::protocol::xproto::{ConfigureWindowAux, ConnectionExt, StackMode, Window};

// box windows are 60x60 (see bin/close_box.rs / bin/monitor_box.rs)
pub const BOX_SIZE: i32 = 60;

// (x, width) of the main window on the given monitor, honoring side_orientation and conditional_full
pub fn main_geometry(
    monitor: &crate::state::Monitor,
    settings: &crate::settings::Settings,
    side_count: usize,
) -> (i32, i32) {
    let side = &settings.layout.side_orientation;
    let fill = settings.layout.conditional_full;
    let half_side = (monitor.sizes.side.0 as f32 * 0.5f32).floor() as i32;
    let width = if side == "both" {
        if fill {
            if side_count == 0 {
                monitor.sizes.screen.0
            } else if side_count == 1 {
                monitor.sizes.main.0 + half_side
            } else {
                monitor.sizes.main.0
            }
        } else {
            monitor.sizes.main.0
        }
    } else if fill && side_count == 0 {
        monitor.sizes.screen.0
    } else {
        monitor.sizes.main.0
    };
    let x = if side == "right" || (fill && side_count == 0) {
        monitor.position.0
    } else if side == "both" {
        monitor.position.0 + half_side
    } else {
        monitor.position.0 + monitor.sizes.side.0
    };
    (x, width)
}

pub fn layout_side_space(state: &mut crate::state::State) {
    let mut hint_positions: Vec<(i32, i32)> = vec![]; // TODO: I hate this
    let mut split_positions: Vec<(i32, i32)> = vec![]; // top-right of each side area
    let side_orientation = &state.settings.layout.side_orientation;
    let true_side_count = state.workspace().side_areas.len();
    for (true_index, area) in state.workspace().side_areas.iter().enumerate() {
        let is_left_side = (true_index % 2) == 0;
        let side_count = if side_orientation == "both" {
            let half = state.workspace().side_areas.len() as f32 * 0.5f32;
            if is_left_side {
                half.ceil() as usize
            } else {
                half.floor() as usize
            }
        } else {
            true_side_count
        };
        let side_height = (state.monitor().sizes.side.1 as f32) / (side_count as f32);
        let index = if side_orientation == "both" {
            let half = true_index as f32 * 0.5f32;
            if is_left_side {
                half.ceil() as usize
            } else {
                half.floor() as usize
            }
        } else {
            true_index
        };
        let size = (
            if side_orientation == "both" {
                (state.monitor().sizes.side.0 as f32 * 0.5f32).floor() as u32
            } else {
                state.monitor().sizes.side.0 as u32
            },
            side_height as u32,
        );
        let position = (
            if (side_orientation == "both" && is_left_side) || side_orientation == "left" {
                state.monitor().position.0
            } else {
                state.monitor().position.0
                    + state.monitor().sizes.main.0
                    + if side_orientation == "both" {
                        (state.monitor().sizes.side.0 as f32 * 0.5f32).floor() as i32
                    } else {
                        0
                    }
            },
            state.monitor().position.1 + (size.1 as f32 * index as f32) as i32,
        );
        // a split area previews as split: its windows side-by-side within the slot
        // TODO: take into account x offset
        crate::windows::layout::split_area(state, area, position, size);
        log::info!(
            "layout_side_space {:?} {},{} {}x{}",
            area,
            position.0,
            position.1,
            size.0,
            size.1,
        );
        hint_positions.push(position);
        split_positions.push((position.0 + size.0 as i32 - BOX_SIZE, position.1));
    }
    if let Err(e) = state.conn.flush() {
        log::error!("windows::fill_main_space(..) flush error: {:?}", e);
    }
    crate::windows::audits::key_hints(state, &hint_positions);
    crate::windows::audits::split_boxes(state, &split_positions);
    let main = state.workspace().main_area.clone();
    if !main.is_empty() {
        crate::windows::core::fill_main_space(state, main);
    }
    crate::windows::layout::place_close_boxes(state);
    crate::windows::layout::place_monitor_boxes(state);
    crate::windows::layout::place_pop_boxes(state);
}

// lay an area's windows out side-by-side across the given space (the last takes any remainder)
pub fn split_area(
    state: &crate::state::State,
    area: &[Window],
    position: (i32, i32),
    size: (u32, u32),
) {
    if area.is_empty() {
        return;
    }
    let width = size.0 / area.len() as u32;
    for (i, window) in area.iter().enumerate() {
        let window_width = if i + 1 == area.len() {
            size.0 - width * i as u32
        } else {
            width
        };
        if let Err(e) = state.conn.configure_window(
            *window,
            &ConfigureWindowAux::new()
                .x(position.0 + (width * i as u32) as i32)
                .y(position.1)
                .width(window_width)
                .height(size.1),
        ) {
            log::error!("windows::split_area(..) move window error: {:?}", e);
        }
    }
}

pub fn fullscreen(state: &mut crate::state::State, area: &[Window]) {
    log::info!("fullscreen {:?}", area);
    split_area(
        state,
        area,
        (
            state.monitor().position.0
                + ((state.monitor().sizes.main.0 + state.monitor().sizes.side.0)
                    - state.monitor().sizes.screen.0),
            state.monitor().position.1
                + (state.monitor().sizes.main.1 - state.monitor().sizes.screen.1),
        ),
        (
            state.monitor().sizes.screen.0 as u32,
            state.monitor().sizes.screen.1 as u32,
        ),
    );
    // all of a split area (not just the focused window) covers boxes / docks / key hints
    for window in area {
        if let Err(e) = state.conn.configure_window(
            *window,
            &ConfigureWindowAux::new().stack_mode(StackMode::ABOVE),
        ) {
            log::error!("windows::fullscreen(..) raise window error: {:?}", e);
        }
    }

    if let Err(e) = state.conn.flush() {
        log::error!("windows::fullscreen(..) flush error: {:?}", e);
    }
    crate::windows::core::focus_main(state);
}

pub fn reapply_float_windows(state: &mut crate::state::State) {
    for window in state.workspace().floatings.clone() {
        if crate::safety::window_exists(state, window) {
            if let Err(e) = state.conn.configure_window(
                window,
                &ConfigureWindowAux::new().stack_mode(StackMode::ABOVE),
            ) {
                log::error!(
                    "windows::reapply_float_windows(..) raise window error: {:?}",
                    e
                );
            }

            if let Err(e) = state.conn.flush() {
                log::error!("windows::reapply_float_windows(..) flush error: {:?}", e);
            }
        } else {
            crate::windows::core::remove_floating(state, window);
            crate::windows::layout::layout_side_space(state);
        }
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
            log::error!("windows::center_window(..) move window error: {:?}", e);
        }
    }

    if let Err(e) = state.conn.flush() {
        log::error!("windows::center_window(..) flush error: {:?}", e);
    }
}

pub fn place_close_boxes(state: &mut crate::state::State) {
    if !state.settings.layout.close_box {
        return;
    }
    for i in 0..state.monitors.len() {
        if let Some(close_box_window) = state.monitors[i].close_box.clone() {
            // right-align the box to the main window's right edge // TODO: real width not hardcoded
            let side_count = state.monitors[i].workspaces[state.current_workspace]
                .side_areas
                .len();
            let (main_x, main_width) =
                main_geometry(&state.monitors[i], &state.settings, side_count);
            let x_pos = main_x + main_width - BOX_SIZE;
            if let Err(e) = state.conn.configure_window(
                close_box_window,
                &ConfigureWindowAux::new()
                    .x(x_pos)
                    .y(state.monitors[i].position.1)
                    .width(BOX_SIZE as u32)
                    .height(BOX_SIZE as u32),
            ) {
                log::error!("windows::place_close_boxes(..) move window error: {:?}", e);
            }
            // raise it
            if !state.monitors[i].workspaces[state.current_workspace].is_fullscreen()
                && let Err(e) = state.conn.configure_window(
                    close_box_window,
                    &ConfigureWindowAux::new().stack_mode(StackMode::ABOVE),
                )
            {
                log::error!("windows::place_close_boxes(..) raise window error: {:?}", e);
            }
        }
    }
    if let Err(e) = state.conn.flush() {
        log::error!("windows::place_close_boxes(..) flush error: {:?}", e);
    }
}

pub fn place_monitor_boxes(state: &mut crate::state::State) {
    if !state.settings.layout.monitor_box {
        return;
    }
    for i in 0..state.monitors.len() {
        if let Some(monitor_box_window) = state.monitors[i].monitor_box.clone() {
            // right-align the box to the main window's right edge // TODO: real width not hardcoded
            let side_count = state.monitors[i].workspaces[state.current_workspace]
                .side_areas
                .len();
            let (main_x, main_width) =
                main_geometry(&state.monitors[i], &state.settings, side_count);
            let x_pos = main_x + main_width - BOX_SIZE;
            let y_pos = if state.settings.layout.close_box {
                state.monitors[i].position.1 + BOX_SIZE
            } else {
                state.monitors[i].position.1
            };
            if let Err(e) = state.conn.configure_window(
                monitor_box_window,
                &ConfigureWindowAux::new()
                    .x(x_pos)
                    .y(y_pos)
                    .width(BOX_SIZE as u32)
                    .height(BOX_SIZE as u32),
            ) {
                log::error!(
                    "windows::place_monitor_boxes(..) move window error: {:?}",
                    e
                );
            }
            // raise it
            if !state.monitors[i].workspaces[state.current_workspace].is_fullscreen()
                && let Err(e) = state.conn.configure_window(
                    monitor_box_window,
                    &ConfigureWindowAux::new().stack_mode(StackMode::ABOVE),
                )
            {
                log::error!(
                    "windows::place_monitor_boxes(..) raise window error: {:?}",
                    e
                );
            }
        }
    }
    if let Err(e) = state.conn.flush() {
        log::error!("windows::place_monitor_boxes(..) flush error: {:?}", e);
    }
}

// below the close / monitor boxes, and only shown while the main area is split
pub fn place_pop_boxes(state: &mut crate::state::State) {
    if !state.settings.layout.pop_box {
        return;
    }
    for i in 0..state.monitors.len() {
        if let Some(pop_box_window) = state.monitors[i].pop_box.clone() {
            let workspace = &state.monitors[i].workspaces[state.current_workspace];
            if workspace.main_area.len() < 2 {
                if let Err(e) = state.conn.unmap_window(pop_box_window) {
                    log::error!("windows::place_pop_boxes(..) unmap window error: {:?}", e);
                }
                continue;
            }
            // right-align the box to the main window's right edge // TODO: real width not hardcoded
            let (main_x, main_width) = main_geometry(
                &state.monitors[i],
                &state.settings,
                workspace.side_areas.len(),
            );
            let x_pos = main_x + main_width - BOX_SIZE;
            let mut y_pos = state.monitors[i].position.1;
            if state.settings.layout.close_box {
                y_pos += BOX_SIZE;
            }
            if state.settings.layout.monitor_box && state.monitors.len() > 1 {
                y_pos += BOX_SIZE;
            }
            if let Err(e) = state.conn.configure_window(
                pop_box_window,
                &ConfigureWindowAux::new()
                    .x(x_pos)
                    .y(y_pos)
                    .width(BOX_SIZE as u32)
                    .height(BOX_SIZE as u32),
            ) {
                log::error!("windows::place_pop_boxes(..) move window error: {:?}", e);
            }
            if let Err(e) = state.conn.map_window(pop_box_window) {
                log::error!("windows::place_pop_boxes(..) map window error: {:?}", e);
            }
            // raise it
            if !workspace.is_fullscreen()
                && let Err(e) = state.conn.configure_window(
                    pop_box_window,
                    &ConfigureWindowAux::new().stack_mode(StackMode::ABOVE),
                )
            {
                log::error!("windows::place_pop_boxes(..) raise window error: {:?}", e);
            }
        }
    }
    if let Err(e) = state.conn.flush() {
        log::error!("windows::place_pop_boxes(..) flush error: {:?}", e);
    }
}

// docks sit above everything but a fullscreen main on their monitor
pub fn raise_docks(state: &mut crate::state::State) {
    let docks: Vec<Window> = state
        .docks
        .iter()
        .copied()
        .filter(|w| crate::safety::window_exists(state, *w))
        .collect();
    state.docks = docks.clone();
    for dock in docks {
        let dock_x = match state.conn.get_geometry(dock) {
            Ok(cookie) => match cookie.reply() {
                Ok(reply) => reply.x as i32,
                Err(_) => continue,
            },
            Err(_) => continue,
        };
        let is_covered = state.monitors.iter().any(|m| {
            dock_x >= m.position.0
                && dock_x < m.position.0 + m.sizes.screen.0
                && m.workspaces[state.current_workspace].is_fullscreen()
        });
        if !is_covered
            && let Err(e) = state.conn.configure_window(
                dock,
                &ConfigureWindowAux::new().stack_mode(StackMode::ABOVE),
            )
        {
            log::error!("windows::raise_docks(..) raise window error: {:?}", e);
        }
    }
    if let Err(e) = state.conn.flush() {
        log::error!("windows::raise_docks(..) flush error: {:?}", e);
    }
}
