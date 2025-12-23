use libc::c_int;

pub fn get_full_size(
    display_width: f32,
    display_height: f32,
    config_string: &str,
) -> (c_int, c_int) {
    println!("calculating sizing with string \"{}\"", config_string);
    // TODO: backup scenarios, rather than asserts
    assert!(config_string.contains("x"));
    let pieces: Vec<String> = config_string.split("x").map(|s| s.to_owned()).collect();
    assert_eq!(pieces.len(), 2);
    // TODO: assert that pieces are made of numbers
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
