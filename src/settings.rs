use std::collections::BTreeMap;

// used for any monitor without a valid layout.main_size / layout.top_left entry
pub const DEFAULT_MAIN_SIZE: &str = "80%x100%";
pub const DEFAULT_TOP_LEFT: &str = "0,0";

#[derive(serde::Deserialize, Clone)]
#[serde(default)]
pub struct Applications {
    pub startups: Vec<String>,
    pub excluded: Vec<String>,
}

#[derive(serde::Deserialize, Clone)]
#[serde(default)]
pub struct Layout {
    pub main_size: Vec<String>,
    pub top_left: Vec<String>,
    pub side_orientation: String,
    pub conditional_full: bool,
    pub new_to_main: bool,
    pub swap_not_stack: bool,
    pub close_box: bool,
    pub monitor_box: bool,
}

#[derive(serde::Deserialize, Clone)]
#[serde(default)]
pub struct Bindings {
    pub functions: BTreeMap<String, String>,
    pub swaps: Vec<String>,
    pub super_for_clicks: bool,
    pub close_main: String,
    pub fullscreen: String,
    pub help: String,
    pub key_hints: bool,
    pub workspaces: Vec<String>,
    pub monitor: String,
}

#[derive(serde::Deserialize, Clone)]
#[serde(default)]
pub struct Files {
    pub log_directory: String,
}

#[derive(serde::Deserialize, Clone)]
#[serde(default)]
pub struct Settings {
    pub applications: Applications,
    pub layout: Layout,
    pub bindings: Bindings,
    pub files: Files,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            applications: Applications {
                startups: vec!["polybar".to_owned()],
                excluded: vec!["polybar".to_owned(), "rofi".to_owned()],
            },
            layout: Layout {
                main_size: vec!["80%x96%".to_owned(), "80%x100%".to_owned()],
                top_left: vec!["0,4%".to_owned(), "0,0".to_owned()],
                side_orientation: "right".to_owned(),
                conditional_full: true,
                new_to_main: true,
                swap_not_stack: true,
                close_box: true,
                monitor_box: true,
            },
            bindings: Bindings {
                functions: BTreeMap::from([
                    ("d".to_owned(), "rofi -show drun".to_owned()),
                    ("t".to_owned(), "alacritty".to_owned()),
                    ("b".to_owned(), "firefox-esr".to_owned()),
                    ("briu".to_owned(), "brightnessctl set +10%".to_owned()),
                    ("brid".to_owned(), "brightnessctl set 10%-".to_owned()),
                    (
                        "volu".to_owned(),
                        "wpctl set-volume @DEFAULT_AUDIO_SINK@ 5%+".to_owned(),
                    ),
                    (
                        "vold".to_owned(),
                        "wpctl set-volume @DEFAULT_AUDIO_SINK@ 5%-".to_owned(),
                    ),
                    (
                        "mute".to_owned(),
                        "wpctl set-mute @DEFAULT_SINK@ toggle".to_owned(),
                    ),
                ]),
                swaps: vec![
                    "j".to_owned(),
                    "k".to_owned(),
                    "l".to_owned(),
                    ";".to_owned(),
                    "u".to_owned(),
                    "i".to_owned(),
                    "o".to_owned(),
                    "p".to_owned(),
                ],
                super_for_clicks: false,
                close_main: "q".to_owned(),
                fullscreen: "f".to_owned(),
                help: "h".to_owned(),
                key_hints: true,
                workspaces: vec![
                    "1".to_owned(),
                    "2".to_owned(),
                    "3".to_owned(),
                    "4".to_owned(),
                ],
                monitor: "tab".to_owned(),
            },
            files: Files {
                log_directory: "~/.local/share/pick-full-wm/logs".to_owned(),
            },
        }
    }
}

// missing fields / sections fall back to these (serde(default)) instead of discarding the file
impl Default for Applications {
    fn default() -> Self {
        Settings::default().applications
    }
}
impl Default for Layout {
    fn default() -> Self {
        Settings::default().layout
    }
}
impl Default for Bindings {
    fn default() -> Self {
        Settings::default().bindings
    }
}
impl Default for Files {
    fn default() -> Self {
        Settings::default().files
    }
}

// also returns warnings, so the wm can log them once logging is set up
pub fn load_settings() -> (Settings, Vec<String>) {
    let file = shellexpand::tilde("~/.config/pick-full-wm/settings.toml").to_string();
    let path = std::path::Path::new(&file);
    let warning = if path.exists() {
        match std::fs::read_to_string(path) {
            Ok(text) => match toml::from_str(&text) {
                Ok(settings) => {
                    println!("settings file loaded.");
                    return (settings, vec![]);
                }
                Err(e) => format!("Settings file \"{}\" has an error: {}", &file, e),
            },
            Err(e) => format!("Failed to read settings file \"{}\": {}", &file, e),
        }
    } else {
        format!("No settings file at \"{}\".", &file)
    };
    let warning = format!("{} Using default settings.", warning);
    println!("{}", warning);
    (Settings::default(), vec![warning])
}
