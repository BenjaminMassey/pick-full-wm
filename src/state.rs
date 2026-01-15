use libc::c_int;
use std::{collections::HashMap, mem::zeroed};
use x11::{xlib, xrandr};

pub struct State {
    pub settings: crate::settings::Settings,
    pub display: *mut xlib::Display,
    pub event: xlib::XEvent,
    pub monitors: Vec<Monitor>,
    pub current_monitor: usize,
    pub current_workspace: usize,
}
impl State {
    pub fn init() -> Self {
        let settings = crate::settings::get_settings();
        let arg0 = 0x0_i8;
        let display = crate::setup::display(arg0);
        if display.is_null() {
            eprintln!("Display \"{}\" is null.", arg0);
            std::process::exit(1);
        }
        let event: xlib::XEvent = unsafe { zeroed() };
        let mut monitor_infos = get_monitor_infos(display);
        monitor_infos.sort_by_key(|m| m.position.0);
        let mut monitors: Vec<Monitor> = vec![];
        for (index, info) in monitor_infos.iter().enumerate() {
            info.print();
            let monitor = Monitor::new(&settings, &info, index);
            monitor.print();
            monitors.push(monitor);
        }
        Self {
            settings,
            display,
            event,
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
}
impl Monitor {
    pub fn new(
        settings: &crate::settings::Settings,
        monitor_info: &MonitorInfo,
        index: usize,
    ) -> Self {
        let sizes = Sizes::init(&settings, monitor_info, index);
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
    pub screen: (c_int, c_int),
    pub main: (c_int, c_int),
    pub side: (c_int, c_int),
}
impl Sizes {
    pub fn init(
        settings: &crate::settings::Settings,
        monitor_info: &MonitorInfo,
        index: usize,
    ) -> Self {
        let screen: (c_int, c_int) = (monitor_info.size.0 as c_int, monitor_info.size.1 as c_int);
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
    pub main_window: Option<xlib::Window>,
    pub side_windows: Vec<Option<xlib::Window>>,
    pub help_window: Option<xlib::Window>,
    pub key_hint_windows: HashMap<String, xlib::Window>,
    pub fullscreen: bool,
}
impl Workspace {
    fn new() -> Self {
        Self {
            main_window: None,
            side_windows: vec![],
            help_window: None,
            key_hint_windows: HashMap::new(),
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

pub fn get_monitor_infos(display: *mut xlib::Display) -> Vec<MonitorInfo> {
    let mut monitor_infos: Vec<MonitorInfo> = vec![];
    let root = unsafe { xlib::XDefaultRootWindow(display) };
    let resources = unsafe { xrandr::XRRGetScreenResources(display, root) };
    if resources.is_null() {
        panic!("unable to get screen resources via xrandr"); // TODO: logging/grace
    }
    let res = unsafe { &*resources };
    for i in 0..res.noutput {
        let output = unsafe { *res.outputs.offset(i as isize) };
        let output_info = unsafe { xrandr::XRRGetOutputInfo(display, resources, output) };
        if output_info.is_null() {
            panic!("unable to get output info via xrandr for #{}", i); // TODO: logging/grace
        }
        let info = unsafe { &*output_info };
        if info.connection as i32 == xrandr::RR_Connected && info.crtc != 0 {
            let crtc_info = unsafe { xrandr::XRRGetCrtcInfo(display, resources, info.crtc) };
            if !crtc_info.is_null() {
                let crtc = unsafe { &*crtc_info };
                monitor_infos.push(MonitorInfo {
                    position: (crtc.x, crtc.y),
                    size: (crtc.width, crtc.height),
                });
                unsafe { xrandr::XRRFreeCrtcInfo(crtc_info) };
            }
        }
        unsafe { xrandr::XRRFreeOutputInfo(output_info) };
    }
    unsafe { xrandr::XRRFreeScreenResources(resources) };
    monitor_infos
}
