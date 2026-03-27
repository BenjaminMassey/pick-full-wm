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
    if state.settings.layout.close_box && state.monitors.len() > 1 {
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
    if std::process::Command::new("which")
        .arg("dbus-launch")
        .status()
        .is_ok()
        && std::env::var("DBUS_SESSION_BUS_ADDRESS").is_err()
    {
        match std::process::Command::new("dbus-launch")
            .arg("--exit-with-session")
            .spawn()
        {
            Ok(_) => log::info!("Successfully launched dbus-launch"),
            Err(e) => log::error!("Warning: Failed to launch dbus-launch: {}", e),
        }
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

    state.conn.sync().expect("Failed to sync");
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
    let _ = simplelog::WriteLogger::init(
        simplelog::LevelFilter::max(),
        simplelog::Config::default(),
        std::fs::File::create(&log_path).unwrap(),
    );
    log::info!("Logging initialized at path \"{}\".", &log_path);
}
