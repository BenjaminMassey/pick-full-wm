use libc::c_int;

pub fn get_full_size(
    display_width: f32,
    display_height: f32,
    config_string: &str,
) -> (c_int, c_int) {
    assert!(config_string.contains("x"));
    let pieces: Vec<String> = config_string.split("x").map(|s| s.to_owned()).collect();
    assert_eq!(pieces.len(), 2);
    // TODO: assert that thing in there is made of numbers
    (
        if pieces[0].chars().collect::<Vec<char>>()[pieces[0].len() - 1] == '%' {
            let section = &pieces[0][0..pieces[0].len() - 1];
            ((section.parse::<i32>().unwrap() as f32 / 100f32) * display_width) as i32
        } else {
            // TODO: assert that percent between 0 and 100
            pieces[0].parse::<i32>().unwrap()
        },
        if pieces[1].chars().collect::<Vec<char>>()[pieces[1].len() - 1] == '%' {
            let section = &pieces[1][0..pieces[1].len() - 1];
            ((section.parse::<i32>().unwrap() as f32 / 100f32) * display_height) as i32
        } else {
            pieces[1].parse::<i32>().unwrap()
        },
    )
}
