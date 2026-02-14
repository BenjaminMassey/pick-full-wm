use std::collections::HashMap;
use x11rb::connection::Connection;
use x11rb::protocol::randr::{self, ConnectionExt as RandrConnectionExt};
use x11rb::protocol::xproto::Window;
use x11rb::rust_connection::RustConnection;

pub struct State {
    pub settings: crate::settings::Settings,
    pub conn: RustConnection,
    pub root: Window,
    pub atoms: crate::atoms::Atoms,
    pub monitors: Vec<Monitor>,
    pub current_monitor: usize,
    pub current_workspace: usize,
}
impl State {
    pub fn init() -> Self {
        let settings = crate::settings::get_settings();
        let (conn, screen_num) = crate::setup::internal::connect();
        let screen = &conn.setup().roots[screen_num];
        let root = screen.root;
        let atoms = crate::atoms::Atoms::new(&conn)
            .expect("Failed to intern atoms")
            .reply()
            .expect("Failed to get atom reply");
        let mut monitor_infos = get_monitor_infos(&conn, root);
        monitor_infos.sort_by_key(|m| m.position.0);
        let mut monitors: Vec<Monitor> = vec![];
        for (index, info) in monitor_infos.iter().enumerate() {
            info.print();
            let monitor = Monitor::new(&settings, info, index);
            monitor.print();
            monitors.push(monitor);
        }
        Self {
            settings,
            conn,
            root,
            atoms,
            monitors,
            current_monitor: 0,
            current_workspace: 0,
        }
    }
    pub fn monitor(&self) -> &Monitor {
        &self.monitors[self.current_monitor]
    }
    pub fn mut_monitor(&mut self) -> &mut Monitor {
        &mut self.monitors[self.current_monitor]
    }
    pub fn workspace(&self) -> &Workspace {
        &self.monitor().workspaces[self.current_workspace]
    }
    pub fn mut_workspace(&mut self) -> &mut Workspace {
        let current_workspace = self.current_workspace;
        &mut self.mut_monitor().workspaces[current_workspace]
    }
}
pub struct Monitor {
    pub sizes: Sizes,
    pub position: (i32, i32),
    pub workspaces: Vec<Workspace>,
    pub close_box: Option<Window>,
}
impl Monitor {
    pub fn new(
        settings: &crate::settings::Settings,
        monitor_info: &MonitorInfo,
        index: usize,
    ) -> Self {
        let sizes = Sizes::init(settings, monitor_info, index);
        let parsed_position = crate::calc::get_position(
            sizes.screen.0 as f32,
            sizes.screen.1 as f32,
            &settings.layout.top_left[index],
        );
        let position = (
            parsed_position.0 + monitor_info.position.0,
            parsed_position.1 + monitor_info.position.1,
        );
        let mut workspaces: Vec<Workspace> = vec![];
        for _ in 0..settings.bindings.workspaces.len() {
            workspaces.push(Workspace::new());
        }
        Self {
            sizes,
            position,
            workspaces,
            close_box: None,
        }
    }
    pub fn print(&self) {
        println!(
            "Calculated Monitor: {}x{} at ({}, {})",
            self.sizes.main.0 + self.sizes.side.0,
            self.sizes.main.1,
            self.position.0,
            self.position.1,
        );
    }
}

pub struct Sizes {
    pub screen: (i32, i32),
    pub main: (i32, i32),
    pub side: (i32, i32),
}
impl Sizes {
    pub fn init(
        settings: &crate::settings::Settings,
        monitor_info: &MonitorInfo,
        index: usize,
    ) -> Self {
        let screen: (i32, i32) = (monitor_info.size.0 as i32, monitor_info.size.1 as i32);
        let main = crate::calc::get_full_size(
            screen.0 as f32,
            screen.1 as f32,
            &settings.layout.main_size[index],
        );
        let side = (screen.0 - main.0, main.1);
        Self { screen, main, side }
    }
}

pub struct Workspace {
    pub main_window: Option<Window>,
    pub side_windows: Vec<Option<Window>>,
    pub help_window: Option<Window>,
    pub key_hint_windows: HashMap<String, Window>,
    pub floatings: Vec<Window>,
    pub fullscreen: bool,
}
impl Workspace {
    fn new() -> Self {
        Self {
            main_window: None,
            side_windows: vec![],
            help_window: None,
            key_hint_windows: HashMap::new(),
            floatings: vec![],
            fullscreen: false,
        }
    }
}

pub struct MonitorInfo {
    pub position: (i32, i32),
    pub size: (u32, u32),
}
impl MonitorInfo {
    pub fn print(&self) {
        println!(
            "Xrandr Monitor: {}x{} at ({}, {})",
            self.size.0, self.size.1, self.position.0, self.position.1,
        );
    }
}

pub fn get_monitor_infos(conn: &RustConnection, root: Window) -> Vec<MonitorInfo> {
    let mut monitor_infos: Vec<MonitorInfo> = vec![];

    // TODO: add more graceful logging around monitor errors
    let resources = conn
        .randr_get_screen_resources(root)
        .expect("Failed to get screen resources")
        .reply()
        .expect("Failed to get screen resources reply");

    for &output in &resources.outputs {
        let output_info = conn
            .randr_get_output_info(output, resources.config_timestamp)
            .expect("Failed to get output info")
            .reply()
            .expect("Failed to get output info reply");

        if output_info.connection == randr::Connection::CONNECTED && output_info.crtc != 0 {
            let crtc_info = conn
                .randr_get_crtc_info(output_info.crtc, resources.config_timestamp)
                .expect("Failed to get crtc info")
                .reply()
                .expect("Failed to get crtc info reply");

            monitor_infos.push(MonitorInfo {
                position: (crtc_info.x as i32, crtc_info.y as i32),
                size: (crtc_info.width as u32, crtc_info.height as u32),
            });
        }
    }
    monitor_infos
}
