use x11rb::protocol::xproto::ConnectionExt;

pub fn get_full_size(display_width: f32, display_height: f32, config_string: &str) -> (i32, i32) {
    config_parse(display_width, display_height, config_string, "x")
}

pub fn get_position(display_width: f32, display_height: f32, config_string: &str) -> (i32, i32) {
    config_parse(display_width, display_height, config_string, ",")
}

fn config_parse(
    display_width: f32,
    display_height: f32,
    config_string: &str,
    deliminator: &str,
) -> (i32, i32) {
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

fn percent_or_value(s: &str, size: f32) -> i32 {
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
    let reply = match state.conn.query_pointer(state.root) {
        Ok(cookie) => match cookie.reply() {
            Ok(reply) => reply,
            Err(_) => return,
        },
        Err(_) => return,
    };

    let root_x = reply.root_x as i32;
    let root_y = reply.root_y as i32;

    for (i, monitor) in state.monitors.iter().enumerate() {
        if root_x >= monitor.position.0
            && root_x < monitor.position.0 + monitor.sizes.screen.0
            && root_y >= monitor.position.1
            && root_y < monitor.position.1 + monitor.sizes.screen.1
        {
            if state.current_monitor != i {
                state.current_monitor = i;
                crate::windows::focus_main(state);
            }
            break;
        }
    }
}
