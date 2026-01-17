use std::ffi::CString;
use x11::xlib;

pub fn set_active(state: &mut crate::state::State, window: xlib::Window) {
    unsafe {
        let net_active_window = xlib::XInternAtom(
            state.display,
            CString::new("_NET_ACTIVE_WINDOW").unwrap().as_ptr(),
            xlib::False,
        );
        xlib::XChangeProperty(
            state.display,
            xlib::XDefaultRootWindow(state.display),
            net_active_window,
            xlib::XA_WINDOW,
            32,
            xlib::PropModeReplace,
            &window as *const xlib::Window as *const u8,
            1,
        );
        if crate::safety::window_exists(state, window) {
            if state.workspace().floatings.is_empty() {
                xlib::XSetInputFocus(
                    state.display,
                    window,
                    xlib::RevertToPointerRoot,
                    xlib::CurrentTime,
                );
            }
            xlib::XRaiseWindow(state.display, window);
        }
        xlib::XFlush(state.display);
    }
}

pub fn clear_active(state: &mut crate::state::State) {
    set_active(state, 0);
}

pub fn update_workspace(state: &crate::state::State) {
    unsafe {
        let net_current_desktop = xlib::XInternAtom(
            state.display,
            CString::new("_NET_CURRENT_DESKTOP").unwrap().as_ptr(),
            xlib::False,
        );
        let current_desktop = state.current_workspace as u64;
        xlib::XChangeProperty(
            state.display,
            xlib::XDefaultRootWindow(state.display),
            net_current_desktop,
            xlib::XA_CARDINAL,
            32,
            xlib::PropModeReplace,
            &current_desktop as *const u64 as *const u8,
            1,
        );
        xlib::XFlush(state.display);
    }
}

pub fn update_client_list(state: &crate::state::State) {
    let mut all_windows: Vec<xlib::Window> = Vec::new();

    for monitor in &state.monitors {
        for (workspace_index, workspace) in monitor.workspaces.iter().enumerate() {
            if let Some(main) = workspace.main_window {
                all_windows.push(main);
                set_window_desktop(state, main, workspace_index);
            }
            for side_window in &workspace.side_windows {
                if let Some(side) = side_window {
                    all_windows.push(*side);
                    set_window_desktop(state, *side, workspace_index);
                }
            }
            /* TODO: I think we don't want these in the list?
            for floating in &workspace.floatings {
                all_windows.push(*floating);
                set_window_desktop(state, *floating, workspace_index);
            }
            for (_, key_window) in &workspace.key_hint_windows {
                all_windows.push(*key_window);
                set_window_desktop(state, *key_window, workspace_index);
            }
            */
            if let Some(help) = workspace.help_window {
                all_windows.push(help);
                set_window_desktop(state, help, workspace_index);
            }
        }
    }

    unsafe {
        let net_client_list = xlib::XInternAtom(
            state.display,
            std::ffi::CString::new("_NET_CLIENT_LIST").unwrap().as_ptr(),
            xlib::False,
        );
        let root = xlib::XDefaultRootWindow(state.display);
        if all_windows.is_empty() {
            xlib::XChangeProperty(
                state.display,
                root,
                net_client_list,
                xlib::XA_WINDOW,
                32,
                xlib::PropModeReplace,
                std::ptr::null(),
                0,
            );
        } else {
            xlib::XChangeProperty(
                state.display,
                root,
                net_client_list,
                xlib::XA_WINDOW,
                32,
                xlib::PropModeReplace,
                all_windows.as_ptr() as *const u8,
                all_windows.len() as i32,
            );
        }

        let net_client_list_stacking = xlib::XInternAtom(
            state.display,
            std::ffi::CString::new("_NET_CLIENT_LIST_STACKING")
                .unwrap()
                .as_ptr(),
            xlib::False,
        );
        if all_windows.is_empty() {
            xlib::XChangeProperty(
                state.display,
                root,
                net_client_list_stacking,
                xlib::XA_WINDOW,
                32,
                xlib::PropModeReplace,
                std::ptr::null(),
                0,
            );
        } else {
            xlib::XChangeProperty(
                state.display,
                root,
                net_client_list_stacking,
                xlib::XA_WINDOW,
                32,
                xlib::PropModeReplace,
                all_windows.as_ptr() as *const u8,
                all_windows.len() as i32,
            );
        }

        xlib::XFlush(state.display);
    }
}

pub fn set_window_desktop(state: &crate::state::State, window: xlib::Window, desktop: usize) {
    unsafe {
        let net_wm_desktop = xlib::XInternAtom(
            state.display,
            std::ffi::CString::new("_NET_WM_DESKTOP").unwrap().as_ptr(),
            xlib::False,
        );
        let desktop_num = desktop as u64;
        xlib::XChangeProperty(
            state.display,
            window,
            net_wm_desktop,
            xlib::XA_CARDINAL,
            32,
            xlib::PropModeReplace,
            &desktop_num as *const u64 as *const u8,
            1,
        );
        xlib::XFlush(state.display);
    }
}
