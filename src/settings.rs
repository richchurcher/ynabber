use config::{Config, ConfigError, File};
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct YNABSettings {
    pub access_token: String,
}

#[derive(Debug, Deserialize)]
pub struct RichSettings {
    pub budget_id: String,
    pub accounts: HashMap<String, String>,
    pub cards: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
pub struct PayeeSettings {
    pub regex: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub ynab: YNABSettings,
    pub rich: RichSettings,
    pub payee: PayeeSettings,
}

const SETTINGS_FILE_PATH: &str = "./Settings.toml";

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let mut s = Config::new();
        s.merge(File::with_name(SETTINGS_FILE_PATH))?;
        s.try_into()
    }
}
