use serde::{Deserialize, Serialize};

fn default_skip_overwrite_warning() -> bool {
    false
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AppConfig {
    #[serde(default = "default_skip_overwrite_warning")]
    pub skip_overwrite_disclaimer: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            skip_overwrite_disclaimer: default_skip_overwrite_warning(),
        }
    }
}

impl AppConfig {
    pub fn load() -> Self {
        confy::load("rclone-shuttle", "config").expect("Failed to load config")
    }

    pub fn save(&self) {
        match confy::store("rclone-shuttle", "config", self) {
            Ok(_) => {}
            Err(err) => println!("Warning: failed to save config. {}", err),
        }
    }
}
