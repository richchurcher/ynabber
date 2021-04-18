use config::{Config, ConfigError, File};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct YNAB {
    pub access_token: String,
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub ynab: YNAB,
}

const SETTINGS_FILE_PATH: &str = "./Settings.toml";

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let mut s = Config::new();
        s.merge(File::with_name(SETTINGS_FILE_PATH))?;
        s.try_into()
    }
}
