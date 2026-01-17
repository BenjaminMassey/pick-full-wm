#[derive(serde::Deserialize, Clone)]
pub struct Applications {
    pub startups: Vec<String>,
    pub launcher: String,
    pub excluded: Vec<String>,
    pub terminal: String,
}

#[derive(serde::Deserialize, Clone)]
pub struct Layout {
    pub main_size: Vec<String>,
    pub top_left: Vec<String>,
    pub conditional_full: bool,
    pub new_to_main: bool,
}

#[derive(serde::Deserialize, Clone)]
pub struct Bindings {
    pub launcher: String,
    pub swaps: Vec<String>,
    pub close_main: String,
    pub fullscreen: String,
    pub help: String,
    pub key_hints: bool,
    pub terminal: String,
    pub workspaces: Vec<String>,
    pub monitor: String,
}

#[derive(serde::Deserialize, Clone)]
pub struct Settings {
    pub applications: Applications,
    pub layout: Layout,
    pub bindings: Bindings,
}
impl Settings {
    fn default() -> Self {
        Self {
            applications: Applications {
                startups: vec!["polybar".to_owned()],
                launcher: "rofi -show drun".to_owned(),
                excluded: vec!["polybar".to_owned(), "rofi".to_owned()],
                terminal: "alacritty".to_owned(),
            },
            layout: Layout {
                main_size: vec!["80%x96%".to_owned(), "80%x100%".to_owned()],
                top_left: vec!["0,4%".to_owned(), "0,0".to_owned()],
                conditional_full: true,
                new_to_main: true,
            },
            bindings: Bindings {
                launcher: "space".to_owned(),
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
                close_main: "q".to_owned(),
                fullscreen: "f".to_owned(),
                help: "h".to_owned(),
                key_hints: true,
                terminal: "t".to_owned(),
                workspaces: vec![
                    "1".to_owned(),
                    "2".to_owned(),
                    "3".to_owned(),
                    "4".to_owned(),
                ],
                monitor: "tab".to_owned(),
            },
        }
    }
}

pub fn get_settings() -> Settings {
    let file = shellexpand::tilde("~/.config/pick-full-wm/settings.toml").to_string();
    let path = std::path::Path::new(&file);
    if path.exists()
        && let Ok(text) = std::fs::read_to_string(path)
        && let Ok(settings) = toml::from_str(&text)
    {
        return settings;
    }
    eprintln!(
        "Failed to load settings file from \"{}\": using defaults.",
        &file,
    );
    Settings::default()
}
