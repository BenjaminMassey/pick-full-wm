use x11rb::protocol::xproto::ConnectionExt;

// None on a malformed config string, so callers can fall back to a default
pub fn get_full_size(
    display_width: f32,
    display_height: f32,
    config_string: &str,
) -> Option<(i32, i32)> {
    config_parse(display_width, display_height, config_string, "x")
}

pub fn get_position(
    display_width: f32,
    display_height: f32,
    config_string: &str,
) -> Option<(i32, i32)> {
    config_parse(display_width, display_height, config_string, ",")
}

fn config_parse(
    display_width: f32,
    display_height: f32,
    config_string: &str,
    deliminator: &str,
) -> Option<(i32, i32)> {
    let (first, second) = config_string.split_once(deliminator)?;
    Some((
        percent_or_value(first, display_width)?,
        percent_or_value(second, display_height)?,
    ))
}

fn percent_or_value(s: &str, size: f32) -> Option<i32> {
    let s = s.trim();
    match s.strip_suffix('%') {
        // TODO: check that percent is between 0 and 100
        Some(percent) => Some(((percent.trim().parse::<f32>().ok()? / 100f32) * size) as i32),
        None => s.parse::<i32>().ok(),
    }
}

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
                log::info!("Updated monitor index to {}.", i);
                state.current_monitor = i;
                crate::windows::core::focus_main(state);
            }
            break;
        }
    }
}
