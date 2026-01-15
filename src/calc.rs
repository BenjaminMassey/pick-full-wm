use libc::{c_int, c_uint};
use x11::xlib;

pub fn get_full_size(
    display_width: f32,
    display_height: f32,
    config_string: &str,
) -> (c_int, c_int) {
    config_parse(display_width, display_height, config_string, "x")
}

pub fn get_position(
    display_width: f32,
    display_height: f32,
    config_string: &str,
) -> (c_int, c_int) {
    config_parse(display_width, display_height, config_string, ",")
}

fn config_parse(
    display_width: f32,
    display_height: f32,
    config_string: &str,
    deliminator: &str,
) -> (c_int, c_int) {
    assert!(config_string.contains(deliminator));
    let pieces: Vec<String> = config_string
        .split(deliminator)
        .map(|s| s.to_owned())
        .collect();
    assert_eq!(pieces.len(), 2);
    // TODO: assert that pieces are made of numbers
    // TODO: backup scenarios, rather than asserts
    (
        percent_or_value(&pieces[0], display_width),
        percent_or_value(&pieces[1], display_height),
    )
}

fn percent_or_value(s: &str, size: f32) -> c_int {
    if last_char(s) == '%' {
        str_percent(&all_but_last(s), size)
    } else {
        s.parse::<i32>().unwrap()
    }
}

fn last_char(s: &str) -> char {
    s.chars().collect::<Vec<char>>()[s.len() - 1]
}

fn all_but_last(s: &str) -> String {
    s[0..s.len() - 1].to_owned()
}

fn str_percent(s: &str, denominator: f32) -> i32 {
    ((s.parse::<i32>().unwrap() as f32 / 100f32) * denominator) as i32
} // TODO: assert that percent between 0 and 100

pub fn update_current_monitor(state: &mut crate::state::State) {
    let root = unsafe { xlib::XDefaultRootWindow(state.display) };
    let mut root_return: xlib::Window = 0;
    let mut child_return: xlib::Window = 0;
    let mut root_x: c_int = 0;
    let mut root_y: c_int = 0;
    let mut win_x: c_int = 0;
    let mut win_y: c_int = 0;
    let mut mask: c_uint = 0;

    unsafe {
        xlib::XQueryPointer(
            state.display,
            root,
            &mut root_return,
            &mut child_return,
            &mut root_x,
            &mut root_y,
            &mut win_x,
            &mut win_y,
            &mut mask,
        )
    };

    for (i, monitor) in state.monitors.iter().enumerate() {
        if root_x >= monitor.position.0
            && root_x < monitor.position.0 + monitor.sizes.screen.0 as i32
            && root_y >= monitor.position.1
            && root_y < monitor.position.1 + monitor.sizes.screen.1 as i32
        {
            if state.current_monitor != i {
                state.current_monitor = i;
                crate::windows::focus_main(state);
            }
            break;
        }
    }
}
