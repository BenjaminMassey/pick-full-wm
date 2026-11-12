use x11rb::CURRENT_TIME;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::{
    Allow, ButtonPressEvent, ConnectionExt, KeyButMask, KeyReleaseEvent, Mapping,
    MappingNotifyEvent,
};

pub fn button(state: &mut crate::state::State, event: ButtonPressEvent) {
    let is_picking =
        event.state.contains(KeyButMask::MOD4) || !state.settings.bindings.super_for_clicks;
    if !crate::safety::window_exists(state, event.child)
        || state.docks.contains(&event.child)
        || crate::windows::checks::is_excepted_window(state, event.child)
        || crate::windows::checks::is_popup(state, event.child)
    {
        if state.workspace().floatings.contains(&event.child)
            && crate::windows::checks::takes_focus(state, event.child)
        {
            crate::ewmh::set_active(state, event.child);
        }
        if let Err(e) = state.conn.allow_events(Allow::REPLAY_POINTER, CURRENT_TIME) {
            log::error!("events::button(..) allow events error: {:?}", e);
        }
        if let Err(e) = state.conn.flush() {
            log::error!("events::button(..) flush error: {:?}", e);
        }
        return;
    }
    if event.detail == 1 {
        // left click
        if crate::windows::checks::is_close_box(state, event.child) {
            log::info!("Clicked on close box.");
            for index in 0..state.monitors.len() {
                if let Some(id) = state.monitors[index].close_box
                    && id == event.child
                {
                    if let Some(main) =
                        state.monitors[index].workspaces[state.current_workspace].focused_main()
                    {
                        crate::windows::core::close_window(state, main);
                        if let Err(e) = state.conn.allow_events(Allow::ASYNC_POINTER, CURRENT_TIME)
                        {
                            log::error!("events::button(..) allow events error: {:?}", e);
                        }
                        if let Err(e) = state.conn.flush() {
                            log::error!("events::button(..) flush error: {:?}", e);
                        }
                        break;
                    }
                }
            }
        } else if crate::windows::checks::is_monitor_box(state, event.child) {
            log::info!("Clicked on monitor box.");
            for index in 0..state.monitors.len() {
                if let Some(id) = state.monitors[index].monitor_box
                    && id == event.child
                {
                    let target_monitor = (state.current_monitor + 1) % state.monitors.len();
                    let move_target = state.workspace().main_area.clone();
                    if !move_target.is_empty() {
                        state.mut_workspace().main_area = vec![];
                        if !state.workspace().side_areas.is_empty() {
                            let target = crate::windows::core::take_side_area(state, 0);
                            crate::windows::core::fill_main_space(state, target);
                            crate::windows::layout::layout_side_space(state);
                        }
                        let current_monitor = state.current_monitor;
                        state.current_monitor = target_monitor;
                        let main = state.workspace().main_area.clone();
                        if !main.is_empty() {
                            state.mut_workspace().main_area = vec![];
                            crate::windows::core::send_side_space(state, main, None);
                        }
                        crate::windows::core::fill_main_space(state, move_target);
                        state.current_monitor = current_monitor;
                    }
                }
            }
        } else if !state.workspace().main_area.is_empty() {
            let is_main = state.workspace().main_area.contains(&event.child);
            if !is_picking || is_main {
                // clicking main takes the keyboard back from an open dialog (floats stay on top),
                // or moves it between the windows of a split main area
                if is_main
                    && (crate::windows::core::focus_floating(state).is_some()
                        || state.workspace().focused_main() != Some(event.child))
                {
                    state.mut_workspace().main_focus = Some(event.child);
                    crate::ewmh::set_active(state, event.child);
                    crate::windows::layout::reapply_float_windows(state);
                }
                if let Err(e) = state.conn.allow_events(Allow::REPLAY_POINTER, CURRENT_TIME) {
                    log::error!("events::button(..) allow events errors: {:?}", e)
                };
                if let Err(e) = state.conn.flush() {
                    log::error!("events::button(..) flush error: {:?}", e);
                }
                return;
            }
            log::info!("Clicked side window {}.", event.child);
            if let Some(index) = state.workspace().side_index(event.child) {
                crate::windows::core::swap_side_area(state, index, Some(event.child));
            }
            if let Err(e) = state.conn.allow_events(Allow::ASYNC_POINTER, CURRENT_TIME) {
                log::error!("events::button(..) allow events error: {:?}", e);
            }
            if let Err(e) = state.conn.flush() {
                log::error!("events::button(..) flush error: {:?}", e);
            }
        }
    } else if event.detail == 3 {
        // right click
        let is_main = state.workspace().main_area.contains(&event.child);
        if !is_picking || is_main {
            if let Err(e) = state.conn.allow_events(Allow::REPLAY_POINTER, CURRENT_TIME) {
                log::error!("events::button(..) allow events errors: {:?}", e)
            };
            if let Err(e) = state.conn.flush() {
                log::error!("events::button(..) flush error: {:?}", e);
            }
            return;
        }
        log::info!("Right-clicked side window {}", event.child);
        crate::windows::core::close_window(state, event.child);
        crate::windows::layout::layout_side_space(state);
        if let Err(e) = state.conn.allow_events(Allow::ASYNC_POINTER, CURRENT_TIME) {
            log::error!("events::button(..) allow events error: {:?}", e);
        }
        if let Err(e) = state.conn.flush() {
            log::error!("events::button(..) flush error: {:?}", e);
        }
    }
    if let Err(e) = state.conn.allow_events(Allow::REPLAY_POINTER, CURRENT_TIME) {
        log::error!("events::button(..) allow events error: {:?}", e);
    }
    if let Err(e) = state.conn.flush() {
        log::error!("events::button(..) flush error: {:?}", e);
    }
}

pub fn key(state: &mut crate::state::State, event: KeyReleaseEvent) {
    let keysym = keycode_to_keysym(state, event.detail);
    let mod4_pressed = event.state.contains(KeyButMask::MOD4);
    let shift_pressed = event.state.contains(KeyButMask::SHIFT);

    for (key, command) in &state.settings.bindings.functions {
        let function_key = crate::keymap::parse_string(&key);
        if let Some(function_key) = function_key {
            if keysym == Some(function_key) && mod4_pressed {
                log::info!("Pressed \"{}\" key for custom function.", &key);
                crate::windows::misc::run_command(&command);
            }
        }
    }

    for (index, key) in state.settings.bindings.swaps.clone().iter().enumerate() {
        if index >= state.workspace().side_areas.len() {
            continue;
        }
        let swap_key = crate::keymap::parse_string(key);
        if let Some(swap_key) = swap_key {
            if keysym == Some(swap_key) && mod4_pressed {
                log::info!("Pressed \"{}\" key for swapping side #{}", key, index);
                crate::windows::core::swap_side_area(state, index, None);
            }
        }
    }

    let split_key = crate::keymap::parse_string(&state.settings.bindings.split);
    if let Some(split_key) = split_key {
        if keysym == Some(split_key) && mod4_pressed {
            if shift_pressed {
                if state.workspace().main_area.len() > 1 {
                    log::info!("Pressed split key plus shift to pop the last main window.");
                    if let Some(popped) = state.mut_workspace().main_area.pop() {
                        crate::windows::core::send_side_space(state, vec![popped], None);
                    }
                } else {
                    log::info!("Pressed split key plus shift: main area isn't split.");
                }
            } else {
                state.split_queued = !state.split_queued;
                log::info!(
                    "Pressed split key: split is now {}.",
                    if state.split_queued {
                        "queued"
                    } else {
                        "unqueued"
                    }
                );
            }
        }
    }

    let cycle_key = crate::keymap::parse_string(&state.settings.bindings.cycle_split);
    if let Some(cycle_key) = cycle_key {
        if keysym == Some(cycle_key) && mod4_pressed {
            log::info!("Pressed cycle split key.");
            crate::windows::core::cycle_split(state);
        }
    }

    let close_key = crate::keymap::parse_string(&state.settings.bindings.close_main);
    if let Some(close_key) = close_key {
        if keysym == Some(close_key) && mod4_pressed {
            if let Some(main) = state.workspace().focused_main() {
                if shift_pressed {
                    log::info!("Pressed close key plus shift to force kill main.");
                    crate::windows::core::kill_window(state, main);
                } else {
                    log::info!("Pressed close key.");
                    crate::windows::core::close_window(state, main);
                }
                if let Err(e) = state.conn.allow_events(Allow::ASYNC_POINTER, CURRENT_TIME) {
                    log::error!("events::key(..) allow events error: {:?}", e);
                }
                if let Err(e) = state.conn.flush() {
                    log::error!("events::key(..) flush error: {:?}", e);
                }
            }
        }
    }

    let full_key = crate::keymap::parse_string(&state.settings.bindings.fullscreen);
    if let Some(full_key) = full_key {
        if keysym == Some(full_key) && mod4_pressed {
            log::info!("Pressed fullscreen key.");
            // keyed on the first window: pops / joins only change the end of a split main
            if let Some(main) = state.workspace().main_area.first().copied() {
                if state.workspace().is_fullscreen() {
                    state.mut_workspace().fullscreen = None;
                } else {
                    state.mut_workspace().fullscreen = Some(main);
                }
                // full layout (not just main) so key hints get re-raised when leaving fullscreen
                crate::windows::layout::layout_side_space(state);
            }
        }
    }

    let help_key = crate::keymap::parse_string(&state.settings.bindings.help);
    if let Some(help_key) = help_key {
        if keysym == Some(help_key) && mod4_pressed {
            crate::binaries::help_window();
        }
    }

    for (index, key) in state
        .settings
        .bindings
        .workspaces
        .clone()
        .iter()
        .enumerate()
    {
        let workspace_key = crate::keymap::parse_string(key);
        if let Some(workspace_key) = workspace_key {
            if keysym == Some(workspace_key) && mod4_pressed && state.current_workspace != index {
                if shift_pressed {
                    log::info!(
                        "Pressed \"{}\" with shift to move current main to workspace #{}",
                        key,
                        index
                    );
                    let move_target = state.workspace().main_area.clone();
                    if !move_target.is_empty() {
                        if !state.workspace().side_areas.is_empty() {
                            let new_main = crate::windows::core::take_side_area(state, 0);
                            crate::windows::core::fill_main_space(state, new_main);
                        } else {
                            state.mut_workspace().main_area = vec![];
                        }
                        let real_current_workspace = state.current_workspace; // TODO: gross temp set
                        state.current_workspace = index; // TODO: gross temp set
                        let move_aside = state.workspace().main_area.clone();
                        if !move_aside.is_empty() {
                            state.mut_workspace().main_area = vec![];
                            crate::windows::core::send_side_space(state, move_aside, None);
                        }
                        crate::windows::core::fill_main_space(state, move_target);
                        state.current_workspace = real_current_workspace; // TODO: gross temp set
                        crate::windows::core::focus_main(state);
                        crate::windows::audits::full(state);
                    }
                } else {
                    log::info!("Pressed \"{}\" to swap to workspace #{}", key, index);
                    state.current_workspace = index;
                }
                crate::windows::workspaces::switch(state);
            }
        }
    }

    let monitor_key = crate::keymap::parse_string(&state.settings.bindings.monitor);
    if let Some(monitor_key) = monitor_key {
        if keysym == Some(monitor_key) && mod4_pressed {
            if shift_pressed {
                log::info!("Pressed monitor key plus shift to move current main to next monitor.");
            } else {
                log::info!("Pressed monitor key to swap to next monitor.");
            }
            let index = (state.current_monitor + 1) % state.monitors.len();
            let target = &state.monitors[index];

            // Calculate center position, clamping to i16 range to prevent overflow
            let x = (target.position.0 + (target.sizes.screen.0 as f32 * 0.5) as i32)
                .clamp(-32768, 32767) as i16;
            let y = (target.position.1 + (target.sizes.screen.1 as f32 * 0.5) as i32)
                .clamp(-32768, 32767) as i16;

            if let Err(e) = state.conn.warp_pointer(0u32, state.root, 0, 0, 0, 0, x, y) {
                log::error!("events::key(..) warp pointer error: {:?}", e);
            }
            if let Err(e) = state.conn.flush() {
                log::error!("events::key(..) flush error: {:?}", e);
            }
            let move_target = state.workspace().main_area.clone();
            if !move_target.is_empty() && shift_pressed {
                state.mut_workspace().main_area = vec![];
                if !state.workspace().side_areas.is_empty() {
                    let target = crate::windows::core::take_side_area(state, 0);
                    crate::windows::core::fill_main_space(state, target);
                    crate::windows::layout::layout_side_space(state);
                }
            }
            state.current_monitor = index;
            if !move_target.is_empty() && shift_pressed {
                let main = state.workspace().main_area.clone();
                if !main.is_empty() {
                    state.mut_workspace().main_area = vec![];
                    crate::windows::core::send_side_space(state, main, None);
                }
                crate::windows::core::fill_main_space(state, move_target);
            }
            crate::windows::core::focus_main(state);
        }
    }
}

fn keycode_to_keysym(state: &crate::state::State, keycode: u8) -> Option<u32> {
    let min_keycode = state.conn.setup().min_keycode;
    let mapping = state.keyboard_mapping.as_ref()?;

    let keysyms_per_keycode = mapping.keysyms_per_keycode as usize;
    let index = (keycode.checked_sub(min_keycode)?) as usize;

    if index * keysyms_per_keycode < mapping.keysyms.len() {
        let keysym = mapping.keysyms[index * keysyms_per_keycode];
        if keysym != 0 {
            return Some(keysym);
        }
    }
    None
}

pub fn mapping(state: &mut crate::state::State, event: MappingNotifyEvent) {
    // pointer / modifier remaps don't move which keycodes our bindings live on
    if event.request != Mapping::KEYBOARD {
        return;
    }
    log::info!("Keyboard mapping changed: re-grabbing keys.");
    crate::setup::input::grab_keys(state);
}
