use std::collections::{HashMap, HashSet};
use x11rb::CURRENT_TIME;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::{
    AtomEnum, ConfigureWindowAux, ConnectionExt, InputFocus, PropMode, StackMode, Window,
};
use x11rb::wrapper::ConnectionExt as WrapperConnectionExt;

pub fn set_active(state: &mut crate::state::State, window: Window) {
    if let Err(e) = state.conn.change_property32(
        PropMode::REPLACE,
        state.root,
        state.atoms._NET_ACTIVE_WINDOW,
        AtomEnum::WINDOW,
        &[window],
    ) {
        log::error!(
            "ewmh::set_active(..) change property NET_ACTIVE_WINDOW error: {:?}",
            e
        );
    }

    if crate::safety::window_exists(state, window) {
        if let Err(e) = state
            .conn
            .set_input_focus(InputFocus::POINTER_ROOT, window, CURRENT_TIME)
        {
            log::error!("ewmh::set_active(..) set input focus error: {:?}", e);
        }

        if let Err(e) = state.conn.configure_window(
            window,
            &ConfigureWindowAux::new().stack_mode(StackMode::ABOVE),
        ) {
            log::error!("ewmh::set_active(..) configure window stack error: {:?}", e);
        }
    }

    if let Err(e) = state.conn.flush() {
        log::error!("ewmh::set_active(..) flush error: {:?}", e);
    }
}

pub fn clear_active(state: &mut crate::state::State) {
    set_active(state, 0);
}

pub fn update_workspace(state: &crate::state::State) {
    if let Err(e) = state.conn.change_property32(
        PropMode::REPLACE,
        state.root,
        state.atoms._NET_CURRENT_DESKTOP,
        AtomEnum::CARDINAL,
        &[state.current_workspace as u32],
    ) {
        log::error!(
            "ewmh::update_workspace(..) change property NET_CURRENT_DESKTOP error: {:?}",
            e
        );
    }

    if let Err(e) = state.conn.flush() {
        log::error!("ewmh::update_workspace(..) flush error: {:?}", e);
    }
}

pub fn update_client_list(state: &mut crate::state::State) {
    let mut clients: HashSet<Window> = state.all_windows.clone();
    for monitor in &state.monitors {
        for workspace in &monitor.workspaces {
            clients.extend(&workspace.floatings);
        }
    }

    // mapping order: keep the last published order, new windows go on the end
    let mut client_list: Vec<Window> = state
        .published_client_list
        .iter()
        .copied()
        .filter(|w| clients.contains(w))
        .collect();
    for window in &clients {
        if !client_list.contains(window) {
            client_list.push(*window);
        }
    }

    // stacking order: query_tree(..) lists the root's children bottom to top
    let client_stacking: Vec<Window> = match state.conn.query_tree(state.root) {
        Ok(cookie) => match cookie.reply() {
            Ok(reply) => reply
                .children
                .into_iter()
                .filter(|w| clients.contains(w))
                .collect(),
            Err(e) => {
                log::error!(
                    "ewmh::update_client_list(..) query tree reply error: {:?}",
                    e
                );
                state.published_client_stacking.clone()
            }
        },
        Err(e) => {
            log::error!("ewmh::update_client_list(..) query tree error: {:?}", e);
            state.published_client_stacking.clone()
        }
    };

    // every write wakes panels watching the root window, so skip unchanged lists
    if client_list != state.published_client_list {
        if let Err(e) = state.conn.change_property32(
            PropMode::REPLACE,
            state.root,
            state.atoms._NET_CLIENT_LIST,
            AtomEnum::WINDOW,
            &client_list,
        ) {
            log::error!(
                "ewmh::update_client_list(..) change property NET_CLIENT_LIST error: {:?}",
                e
            );
        }
        state.published_client_list = client_list;
    }

    if client_stacking != state.published_client_stacking {
        if let Err(e) = state.conn.change_property32(
            PropMode::REPLACE,
            state.root,
            state.atoms._NET_CLIENT_LIST_STACKING,
            AtomEnum::WINDOW,
            &client_stacking,
        ) {
            log::error!(
                "ewmh::update_client_list(..) change property NET_CLIENT_LIST_STACKING error: {:?}",
                e
            );
        }
        state.published_client_stacking = client_stacking;
    }

    if let Err(e) = state.conn.flush() {
        log::error!("ewmh::update_client_list(..) flush error: {:?}", e);
    }
}

pub fn desktop_assignments(state: &mut crate::state::State) {
    let mut desktops: HashMap<Window, usize> = HashMap::new();
    for monitor in &state.monitors {
        for (workspace_index, workspace) in monitor.workspaces.iter().enumerate() {
            if let Some(main) = workspace.main_window {
                desktops.insert(main, workspace_index);
            }
            for side_window in &workspace.side_windows {
                if let Some(side) = side_window {
                    desktops.insert(*side, workspace_index);
                }
            }
            if let Some(help) = workspace.help_window {
                desktops.insert(help, workspace_index);
            }
            for floating in &workspace.floatings {
                desktops.insert(*floating, workspace_index);
            }
        }
    }
    // only write changes: each write notifies the window's client
    for (window, desktop) in &desktops {
        if state.published_desktops.get(window) != Some(desktop) {
            set_window_desktop(state, *window, *desktop);
        }
    }
    state.published_desktops = desktops;
}

fn set_window_desktop(state: &crate::state::State, window: Window, desktop: usize) {
    if let Err(e) = state.conn.change_property32(
        PropMode::REPLACE,
        window,
        state.atoms._NET_WM_DESKTOP,
        AtomEnum::CARDINAL,
        &[desktop as u32],
    ) {
        log::error!(
            "ewmh::set_window_desktop(..) change property NET_WM_DESKTOP error: {:?}",
            e
        );
    }

    if let Err(e) = state.conn.flush() {
        log::error!("ewmh::set_window_desktop(..) flush error: {:?}", e);
    }
}
