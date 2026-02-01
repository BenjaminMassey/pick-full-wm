use x11rb::connection::Connection;
use x11rb::protocol::xproto::{ConfigureWindowAux, ConnectionExt, StackMode, Window};

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
    crate::windows::audits::key_hints(state, &positions);
    if let Some(main) = state.workspace().main_window {
        crate::windows::core::fill_main_space(state, main);
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
    crate::windows::core::focus_main(state);
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
            eprintln!("windows::center_window(..) move window error: {:?}", e);
        }
    }

    if let Err(e) = state.conn.flush() {
        eprintln!("windows::center_window(..) flush error: {:?}", e);
    }
}
