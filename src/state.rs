use libc::c_int;
use std::mem::zeroed;
use x11::xlib;

pub struct State {
    pub settings: crate::settings::Settings,
    pub display: *mut xlib::Display,
    pub sizes: Sizes,
    pub event: xlib::XEvent,
    pub main_window: Option<xlib::Window>,
    pub side_windows: Vec<Option<xlib::Window>>,
}
impl State {
    pub fn init() -> Self {
        let settings = crate::settings::get_settings();
        let arg0 = 0x0_i8;
        let display = unsafe { xlib::XOpenDisplay(&arg0) };
        if display.is_null() {
            eprintln!("Display \"{}\" is null.", arg0);
            std::process::exit(1);
        }
        let sizes = Sizes::init(&settings, arg0, display);
        let event: xlib::XEvent = unsafe { zeroed() };
        Self {
            settings,
            display,
            sizes,
            event,
            main_window: None,
            side_windows: vec![],
        }
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
        arg0: i8,
        display: *mut xlib::Display,
    ) -> Self {
        let screen: (c_int, c_int) = (
            unsafe { xlib::XDisplayWidth(display, arg0.into()) },
            unsafe { xlib::XDisplayHeight(display, arg0.into()) },
        );
        let main = crate::calc::get_full_size(
            screen.0 as f32,
            screen.1 as f32,
            &settings.layout.main_size,
        );
        let side = (screen.0 - main.0, main.1);
        let ret = Self { screen, main, side };
        ret.print();
        ret
    }
    fn print(&self) {
        println!(
            "Calculate Sizes:\n\tScreen: {}x{}\n\tMain: {}x{}\n\tSide: {}x{}",
            self.screen.0, self.screen.1, self.main.0, self.main.1, self.side.0, self.side.1
        );
    }
}
