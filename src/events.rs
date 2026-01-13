use x11::xlib;

pub fn map_request(state: &mut crate::state::State) {
    let event: xlib::XMapRequestEvent = From::from(state.event);
    if !crate::safety::window_exists(state, event.window) {
        unsafe { xlib::XAllowEvents(state.display, xlib::AsyncBoth, xlib::CurrentTime) };
        return;
    }
    unsafe { xlib::XMapWindow(state.display, event.window) };
    if let Some(key) = crate::windows::get_key_hint_window(state, event.window) {
        if let Some(entry) = state.key_hint_windows.get_mut(&key) {
            *entry = event.window;
        } else {
            state.key_hint_windows.insert(key, event.window);
        }
        crate::windows::layout_side_space(state);
        return;
    }
    if crate::windows::is_help_window(state, event.window) {
        crate::ewmh::set_active(state, event.window);
        unsafe { xlib::XFlush(state.display) };
        state.help_window = Some(event.window);
        return;
    }
    if crate::windows::is_excepted_window(state, event.window) {
        return;
    }
    if state.main_window.is_none() {
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
        if let Some(existing) = state.main_window {
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
        if let Some(existing) = state.main_window
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
        if index >= state.side_windows.len() {
            continue;
        }
        let swap_key = crate::keymap::parse_string(key);
        if let Some(swap_key) = swap_key {
            if keysym == swap_key as u64 && (event.state & xlib::Mod4Mask) != 0 {
                let target = state.side_windows[index];
                if let Some(target) = target {
                    let existing = state.main_window.clone();
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
            if let Some(main) = state.main_window {
                unsafe { xlib::XDestroyWindow(state.display, main) };
                unsafe { xlib::XAllowEvents(state.display, xlib::AsyncPointer, xlib::CurrentTime) };
            }
        }
    }

    let full_key = crate::keymap::parse_string(&state.settings.bindings.fullscreen);
    if let Some(full_key) = full_key {
        if keysym == full_key as u64 && (event.state & xlib::Mod4Mask) != 0 {
            if let Some(main) = state.main_window {
                state.fullscreen = !state.fullscreen;
                if state.fullscreen {
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
}

pub fn destroy(state: &mut crate::state::State) {
    let event: xlib::XDestroyWindowEvent = From::from(state.event);
    if let Some(help) = state.help_window
        && event.window == help
    {
        if let Some(main_window) = state.main_window {
            crate::ewmh::set_active(state, main_window);
            unsafe { xlib::XFlush(state.display) };
        }
        state.help_window = None;
        return;
    }
    if let Some(main_window) = state.main_window {
        if event.window == main_window {
            if !state.side_windows.is_empty() {
                if let Some(target) = state.side_windows[0] {
                    crate::windows::remove_side_window(state, target);
                    crate::windows::fill_main_space(state, target);
                }
            } else {
                state.main_window = None;
            }
        } else {
            crate::windows::remove_side_window(state, event.window);
        }
    }
    crate::windows::layout_side_space(state);
    if state.main_window.is_none() {
        crate::ewmh::clear_active(state);
    }
}
