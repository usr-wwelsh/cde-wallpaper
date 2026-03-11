use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    pub wallpaper_dir: Option<String>,
    pub selected_file: Option<String>,
    pub selected_is_embedded: bool,
    pub hide_defaults: bool,
    pub fg_color: [u8; 3],
    pub bg_color: [u8; 3],
}

impl Default for Config {
    fn default() -> Self {
        Config {
            wallpaper_dir: None,
            selected_file: None,
            selected_is_embedded: false,
            hide_defaults: false,
            fg_color: [43, 80, 115],
            bg_color: [148, 148, 148],
        }
    }
}

fn config_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/root".to_string());
    PathBuf::from(format!("{}/.config/cde-wallpaper/config.toml", home))
}

impl Config {
    pub fn load() -> Config {
        let path = config_path();
        if let Ok(contents) = std::fs::read_to_string(&path) {
            toml::from_str(&contents).unwrap_or_default()
        } else {
            Config::default()
        }
    }

    pub fn save(&self) {
        let path = config_path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(s) = toml::to_string(self) {
            let _ = std::fs::write(&path, s);
        }
    }
}
