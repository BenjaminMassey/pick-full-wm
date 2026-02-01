use x11rb::connection::Connection;
use x11rb::protocol::xproto::{AtomEnum, ConnectionExt, CreateWindowAux, PropMode, WindowClass};
use x11rb::wrapper::ConnectionExt as WrapperConnectionExt;

pub fn init(state: &mut crate::state::State) {
    // Create a dummy window for EWMH compliance check
    let check_window = state
        .conn
        .generate_id()
        .expect("Failed to generate window id");
    state
        .conn
        .create_window(
            0,
            check_window,
            state.root,
            0,
            0,
            1,
            1,
            0,
            WindowClass::INPUT_OUTPUT,
            0,
            &CreateWindowAux::new(),
        )
        .expect("Failed to create check window");

    // Set _NET_SUPPORTED on root window
    let supported_atoms = [
        state.atoms._NET_ACTIVE_WINDOW,
        state.atoms._NET_WM_NAME,
        state.atoms._NET_CLIENT_LIST,
        state.atoms._NET_SUPPORTING_WM_CHECK,
        state.atoms._NET_NUMBER_OF_DESKTOPS,
        state.atoms._NET_CURRENT_DESKTOP,
        state.atoms._NET_DESKTOP_NAMES,
    ];

    state
        .conn
        .change_property32(
            PropMode::REPLACE,
            state.root,
            state.atoms._NET_SUPPORTED,
            AtomEnum::ATOM,
            &supported_atoms,
        )
        .expect("Failed to set _NET_SUPPORTED");

    // Set _NET_SUPPORTING_WM_CHECK on root and check window
    state
        .conn
        .change_property32(
            PropMode::REPLACE,
            state.root,
            state.atoms._NET_SUPPORTING_WM_CHECK,
            AtomEnum::WINDOW,
            &[check_window],
        )
        .expect("Failed to set _NET_SUPPORTING_WM_CHECK on root");

    state
        .conn
        .change_property32(
            PropMode::REPLACE,
            check_window,
            state.atoms._NET_SUPPORTING_WM_CHECK,
            AtomEnum::WINDOW,
            &[check_window],
        )
        .expect("Failed to set _NET_SUPPORTING_WM_CHECK on check window");

    // Set WM name on check window
    let wm_name = b"Pick-Full-WM";
    state
        .conn
        .change_property8(
            PropMode::REPLACE,
            check_window,
            state.atoms._NET_WM_NAME,
            state.atoms.UTF8_STRING,
            wm_name,
        )
        .expect("Failed to set _NET_WM_NAME");

    // Set number of desktops
    let num_desktops = state.monitor().workspaces.len() as u32;
    state
        .conn
        .change_property32(
            PropMode::REPLACE,
            state.root,
            state.atoms._NET_NUMBER_OF_DESKTOPS,
            AtomEnum::CARDINAL,
            &[num_desktops],
        )
        .expect("Failed to set _NET_NUMBER_OF_DESKTOPS");

    // Set current desktop
    state
        .conn
        .change_property32(
            PropMode::REPLACE,
            state.root,
            state.atoms._NET_CURRENT_DESKTOP,
            AtomEnum::CARDINAL,
            &[0u32],
        )
        .expect("Failed to set _NET_CURRENT_DESKTOP");

    // Set desktop names
    let names: Vec<u8> = state
        .settings
        .bindings
        .workspaces
        .iter()
        .flat_map(|s| {
            let mut bytes = s.as_bytes().to_vec();
            bytes.push(0);
            bytes
        })
        .collect();

    state
        .conn
        .change_property8(
            PropMode::REPLACE,
            state.root,
            state.atoms._NET_DESKTOP_NAMES,
            state.atoms.UTF8_STRING,
            &names,
        )
        .expect("Failed to set _NET_DESKTOP_NAMES");

    state.conn.sync().expect("Failed to sync");
}
