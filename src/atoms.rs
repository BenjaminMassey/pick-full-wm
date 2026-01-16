use x11rb::atom_manager;

atom_manager! {
    pub Atoms: AtomsCookie {
        _NET_SUPPORTED,
        _NET_SUPPORTING_WM_CHECK,
        _NET_WM_NAME,
        _NET_ACTIVE_WINDOW,
        _NET_CLIENT_LIST,
        _NET_NUMBER_OF_DESKTOPS,
        _NET_CURRENT_DESKTOP,
        _NET_DESKTOP_NAMES,
        UTF8_STRING,
        WM_NAME,
    }
}
