pub fn help_window() {
    if let Some(dir) = get_pfwm_dir() {
        let path = &format!("{}/help_window", dir);
        println!("!!! : {}", &path);
        crate::windows::run_command(&path);
    } else {
        eprintln!("failed to get pfwm dir");
    }
}

fn get_pfwm_dir() -> Option<String> {
    if let Ok(path) = std::env::current_exe() {
        if let Some(dir) = path.parent() {
            return Some(dir.to_str()?.to_owned());
        }
    }
    None
}
