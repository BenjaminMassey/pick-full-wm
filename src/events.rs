use x11::xlib;

pub fn map_request(state: &mut crate::state::State) {
    let event: xlib::XMapRequestEvent = From::from(state.event);
    if !crate::safety::window_exists(state, event.window) {
        unsafe { xlib::XAllowEvents(state.display, xlib::AsyncBoth, xlib::CurrentTime) };
        return;
    }
    unsafe { xlib::XMapWindow(state.display, event.window) };
    if let Some(key) = crate::windows::get_key_hint_window(state, event.window) {
        if let Some(entry) = state.mut_workspace().key_hint_windows.get_mut(&key) {
            *entry = event.window;
        } else {
            state
                .mut_workspace()
                .key_hint_windows
                .insert(key, event.window);
        }
        crate::windows::layout_side_space(state);
        return;
    }
    if crate::windows::is_help_window(state, event.window) {
        crate::ewmh::set_active(state, event.window);
        unsafe { xlib::XFlush(state.display) };
        state.mut_workspace().help_window = Some(event.window);
        return;
    }
    if crate::windows::is_excepted_window(state, event.window) {
        return;
    }
    if state.workspace().main_window.is_none() {
        crate::windows::fill_main_space(state, event.window);
    } else {
        crate::windows::send_side_space(state, event.window);
    }
}

pub fn button(state: &mut crate::state::State) {
    let event: xlib::XButtonEvent = From::from(state.event);
    if !crate::safety::window_exists(state, event.window)
        || crate::windows::is_excepted_window(state, event.subwindow)
    {
        unsafe { xlib::XAllowEvents(state.display, xlib::ReplayPointer, xlib::CurrentTime) };
        return;
    }
    if event.button == 1 {
        // left click
        if let Some(existing) = state.workspace().main_window {
            if existing == event.subwindow {
                unsafe {
                    xlib::XAllowEvents(state.display, xlib::ReplayPointer, xlib::CurrentTime)
                };
                return;
            }
            crate::windows::remove_side_window(state, event.subwindow);
            crate::windows::fill_main_space(state, event.subwindow);
            crate::windows::send_side_space(state, existing);
            unsafe { xlib::XAllowEvents(state.display, xlib::AsyncPointer, xlib::CurrentTime) };
        }
    } else if event.button == 3 {
        // right click
        if let Some(existing) = state.workspace().main_window
            && existing == event.subwindow
        {
            unsafe { xlib::XAllowEvents(state.display, xlib::ReplayPointer, xlib::CurrentTime) };
            return;
        }
        unsafe { xlib::XDestroyWindow(state.display, event.subwindow) };
        crate::windows::layout_side_space(state);
        unsafe { xlib::XAllowEvents(state.display, xlib::AsyncPointer, xlib::CurrentTime) };
    }
}

pub fn key(state: &mut crate::state::State) {
    let event: xlib::XKeyReleasedEvent = From::from(state.event);
    let keysym = unsafe { xlib::XKeycodeToKeysym(state.display, event.keycode as u8, 0) };

    let launcher_key = crate::keymap::parse_string(&state.settings.bindings.launcher);
    if let Some(launcher_key) = launcher_key {
        if keysym == launcher_key as u64 && (event.state & xlib::Mod4Mask) != 0 {
            crate::windows::run_command(&state.settings.applications.launcher);
        }
    }

    for (index, key) in state.settings.bindings.swaps.clone().iter().enumerate() {
        if index >= state.workspace().side_windows.len() {
            continue;
        }
        let swap_key = crate::keymap::parse_string(key);
        if let Some(swap_key) = swap_key {
            if keysym == swap_key as u64 && (event.state & xlib::Mod4Mask) != 0 {
                let target = state.workspace().side_windows[index];
                if let Some(target) = target {
                    let existing = state.workspace().main_window.clone();
                    crate::windows::remove_side_window(state, target);
                    crate::windows::fill_main_space(state, target);
                    if let Some(existing) = existing {
                        crate::windows::send_side_space(state, existing);
                    }
                }
            }
        }
    }

    let close_key = crate::keymap::parse_string(&state.settings.bindings.close_main);
    if let Some(close_key) = close_key {
        if keysym == close_key as u64 && (event.state & xlib::Mod4Mask) != 0 {
            if let Some(main) = state.workspace().main_window {
                unsafe { xlib::XDestroyWindow(state.display, main) };
                unsafe { xlib::XAllowEvents(state.display, xlib::AsyncPointer, xlib::CurrentTime) };
            }
        }
    }

    let full_key = crate::keymap::parse_string(&state.settings.bindings.fullscreen);
    if let Some(full_key) = full_key {
        if keysym == full_key as u64 && (event.state & xlib::Mod4Mask) != 0 {
            if let Some(main) = state.workspace().main_window {
                state.mut_workspace().fullscreen = !state.workspace().fullscreen;
                if state.workspace().fullscreen {
                    crate::windows::fullscreen(state, main);
                } else {
                    crate::windows::fill_main_space(state, main);
                }
            }
        }
    }

    let help_key = crate::keymap::parse_string(&state.settings.bindings.help);
    if let Some(help_key) = help_key {
        if keysym == help_key as u64 && (event.state & xlib::Mod4Mask) != 0 {
            crate::binaries::help_window();
        }
    }

    let term_key = crate::keymap::parse_string(&state.settings.bindings.terminal);
    if let Some(term_key) = term_key {
        if keysym == term_key as u64 && (event.state & xlib::Mod4Mask) != 0 {
            crate::windows::run_command(&state.settings.applications.terminal);
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
            if keysym == workspace_key as u64
                && (event.state & xlib::Mod4Mask) != 0
                && state.current_workspace != index
            {
                state.current_workspace = index;
                crate::windows::switch_workspace(state);
            }
        }
    }

    let monitor_key = crate::keymap::parse_string(&state.settings.bindings.monitor);
    if let Some(monitor_key) = monitor_key {
        if keysym == monitor_key as u64 && (event.state & xlib::Mod4Mask) != 0 {
            let index = (state.current_monitor + 1) % state.monitors.len();
            let target = &state.monitors[index];
            unsafe {
                xlib::XWarpPointer(
                    state.display,
                    0,
                    xlib::XDefaultRootWindow(state.display),
                    0,
                    0,
                    0,
                    0,
                    target.position.0 + (target.sizes.screen.0 as f32 * 0.5) as i32,
                    target.position.1 + (target.sizes.screen.1 as f32 * 0.5) as i32,
                );
                xlib::XFlush(state.display);
            }
            state.current_monitor = index;
            crate::windows::focus_main(state);
        }
    }
}

pub fn destroy(state: &mut crate::state::State) {
    let event: xlib::XDestroyWindowEvent = From::from(state.event);
    for i in 0..state.monitor().workspaces.len() {
        if let Some(help) = state.monitor().workspaces[i].help_window
            && event.window == help
        {
            if let Some(main_window) = state.monitor().workspaces[i].main_window
                && state.current_workspace == i
            {
                crate::ewmh::set_active(state, main_window);
                unsafe { xlib::XFlush(state.display) };
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
                        crate::windows::remove_side_window(state, target);
                        crate::windows::fill_main_space(state, target);
                    }
                } else {
                    state.mut_workspace().main_window = None;
                }
            } else {
                crate::windows::remove_side_window(state, event.window);
            }
        }
        state.current_workspace = real_workspace; // TODO: gross, for windows.rs calls
    }
    crate::windows::layout_side_space(state);
    if state.workspace().main_window.is_none() {
        crate::ewmh::clear_active(state);
    }
}

pub fn client_message(state: &mut crate::state::State) {
    let event: xlib::XClientMessageEvent = From::from(state.event);
    let net_current_desktop = unsafe {
        xlib::XInternAtom(
            state.display,
            std::ffi::CString::new("_NET_CURRENT_DESKTOP")
                .unwrap()
                .as_ptr(),
            xlib::False,
        )
    };
    if event.message_type == net_current_desktop {
        let requested_workspace = event.data.get_long(0) as usize;
        if state.current_workspace != requested_workspace {
            state.current_workspace = requested_workspace;
            crate::windows::switch_workspace(state);
        }
    }
}
