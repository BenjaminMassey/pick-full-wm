pub fn help_window() {
    if let Some(dir) = get_pfwm_dir() {
        let path = &format!("{}/help_window", dir);
        crate::windows::misc::run_command(&path);
    }
}

pub fn close_box() {
    if let Some(dir) = get_pfwm_dir() {
        let path = &format!("{}/close_box", dir);
        crate::windows::misc::run_command(&path);
    }
}

pub fn monitor_box() {
    if let Some(dir) = get_pfwm_dir() {
        let path = &format!("{}/monitor_box", dir);
        crate::windows::misc::run_command(&path);
    }
}

pub fn key_hint(key: &str) {
    if let Some(dir) = get_pfwm_dir() {
        let path = &format!("{}/key_hint \"{}\"", dir, key);
        crate::windows::misc::run_command(&path);
    }
}

fn get_pfwm_dir() -> Option<String> {
    if let Ok(path) = std::env::current_exe() {
        if let Some(dir) = path.parent() {
            return Some(dir.to_str()?.to_owned());
        }
    }
    eprintln!("failed to get pfwm dir");
    None
}
