#[derive(serde::Deserialize, Clone)]
pub struct Applications {
    pub startups: Vec<String>,
    pub launcher: String,
    pub excluded: Vec<String>,
}

#[derive(serde::Deserialize, Clone)]
pub struct Layout {
    pub main_size: String,
}

#[derive(serde::Deserialize, Clone)]
pub struct Bindings {
    pub launcher: String,
    pub swaps: Vec<String>,
    pub close_main: String,
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
                startups: vec!["polybar".to_owned(), "rofi -show run".to_owned()],
                launcher: "rofi -show run".to_owned(),
                excluded: vec!["polybar".to_owned(), "rofi".to_owned()],
            },
            layout: Layout {
                main_size: "80%x96%".to_owned(),
            },
            bindings: Bindings {
                launcher: "space".to_owned(),
                swaps: vec!["j".to_owned(), "k".to_owned(), "l".to_owned(), ";".to_owned()],
                close_main: "q".to_owned(),
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
