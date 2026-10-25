use x11rb::CURRENT_TIME;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::{ChangeWindowAttributesAux, ConnectionExt, EventMask, ModMask};
use x11rb::rust_connection::RustConnection;
use x11rb::wrapper::ConnectionExt as WrapperConnectionExt;

pub fn startups(state: &mut crate::state::State) {
    if state.settings.layout.close_box {
        for _ in 0..state.monitors.len() {
            crate::binaries::close_box();
        }
    }
    if state.settings.layout.monitor_box && state.monitors.len() > 1 {
        for _ in 0..state.monitors.len() {
            crate::binaries::monitor_box();
        }
    }
}

pub fn custom_startups(state: &mut crate::state::State) {
    for startup in &state.settings.applications.startups {
        crate::windows::misc::run_command(startup);
    }
}

pub fn dbus_init() {
    // systemd / dbus activated services (portals, etc) need this session's display
    match std::process::Command::new("dbus-update-activation-environment")
        .args([
            "--systemd",
            "DISPLAY",
            "XAUTHORITY",
            "XDG_CURRENT_DESKTOP=pick-full-wm",
        ])
        .status()
    {
        Ok(_) => log::info!("Updated dbus activation environment."),
        Err(e) => log::error!("Failed to update dbus activation environment: {}", e),
    }
    // a leftover wayland session's display would be tried first by gtk services
    match std::process::Command::new("systemctl")
        .args(["--user", "unset-environment", "WAYLAND_DISPLAY"])
        .status()
    {
        Ok(_) => log::info!("Unset WAYLAND_DISPLAY from systemd environment."),
        Err(e) => log::error!("Failed to unset WAYLAND_DISPLAY: {}", e),
    }
    // portals still running from another session keep its environment: restart if running
    match std::process::Command::new("systemctl")
        .args([
            "--user",
            "--no-block",
            "try-restart",
            "xdg-desktop-portal.service",
            "xdg-desktop-portal-gtk.service",
        ])
        .status()
    {
        Ok(_) => log::info!("Restarted any running xdg-desktop-portal services."),
        Err(e) => log::error!("Failed to restart xdg-desktop-portal services: {}", e),
    }
}

pub fn windows(state: &mut crate::state::State) {
    state
        .conn
        .change_window_attributes(
            state.root,
            &ChangeWindowAttributesAux::new()
                .event_mask(EventMask::SUBSTRUCTURE_REDIRECT | EventMask::SUBSTRUCTURE_NOTIFY),
        )
        .expect("Failed to select input on root");

    root_cursor(state);

    state.conn.sync().expect("Failed to sync");
}

// the root has no cursor of its own: without one the pointer is invisible until a client sets one
fn root_cursor(state: &mut crate::state::State) {
    let database = match x11rb::resource_manager::new_from_default(&state.conn) {
        Ok(database) => database,
        Err(e) => {
            log::error!("setup::root_cursor(..) resource database error: {:?}", e);
            return;
        }
    };
    let handle = match x11rb::cursor::Handle::new(&state.conn, state.screen_num, &database) {
        Ok(cookie) => match cookie.reply() {
            Ok(handle) => handle,
            Err(e) => {
                log::error!("setup::root_cursor(..) cursor handle reply error: {:?}", e);
                return;
            }
        },
        Err(e) => {
            log::error!("setup::root_cursor(..) cursor handle error: {:?}", e);
            return;
        }
    };
    let cursor = match handle.load_cursor(&state.conn, "left_ptr") {
        Ok(cursor) => cursor,
        Err(e) => {
            log::error!("setup::root_cursor(..) load cursor error: {:?}", e);
            return;
        }
    };
    if let Err(e) = state
        .conn
        .change_window_attributes(state.root, &ChangeWindowAttributesAux::new().cursor(cursor))
    {
        log::error!("setup::root_cursor(..) set cursor error: {:?}", e);
    }
    log::info!("Set root window cursor.");
}

pub fn connect() -> (RustConnection, usize) {
    let (conn, screen_num) = x11rb::connect(None).expect("Failed to connect to X server");

    let screen = &conn.setup().roots[screen_num];
    let root = screen.root;

    conn.ungrab_keyboard(CURRENT_TIME)
        .expect("Failed to ungrab keyboard");
    conn.ungrab_pointer(CURRENT_TIME)
        .expect("Failed to ungrab pointer");
    conn.ungrab_server().expect("Failed to ungrab server");
    conn.ungrab_key(0u8, root, ModMask::ANY)
        .expect("Failed to ungrab keys");

    let black_pixel = screen.black_pixel;
    conn.change_window_attributes(
        root,
        &ChangeWindowAttributesAux::new().background_pixel(black_pixel),
    )
    .expect("Failed to set background");

    conn.clear_area(false, root, 0i16, 0i16, 0u16, 0u16)
        .expect("Failed to clear window");

    conn.sync().expect("Failed to sync");

    (conn, screen_num)
}

pub fn logging(state: &crate::state::State) {
    let dir_str = shellexpand::tilde(&state.settings.files.log_directory).to_string();
    let dir_path = std::path::Path::new(&dir_str);
    if !dir_path.exists() {
        let _ = std::fs::create_dir_all(dir_path);
    }
    let time: chrono::DateTime<chrono::offset::Local> = chrono::offset::Local::now();
    let time_str = time.format("%Y_%m_%d-%H_%M_%S").to_string();
    let log_path = format!("{}/{}.log", dir_path.to_str().unwrap(), time_str);
    // local time only resolves while single-threaded, so this must run early
    let mut config = simplelog::ConfigBuilder::new();
    let is_local = config.set_time_offset_to_local().is_ok();
    let _ = simplelog::WriteLogger::init(
        simplelog::LevelFilter::max(),
        config.build(),
        std::fs::File::create(&log_path).unwrap(),
    );
    log::info!("Logging initialized at path \"{}\".", &log_path);
    if !is_local {
        log::warn!("Failed to get local time offset: log times are UTC.");
    }
    for warning in &state.startup_warnings {
        log::warn!("{}", warning);
    }
    // panics otherwise only reach stderr (the tty), not the log
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        log::error!("Panic: {}", info);
        default_hook(info);
    }));
}
