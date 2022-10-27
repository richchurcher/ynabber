use config::{Config, ConfigError, File};
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct YNABSettings {
    pub access_token: String,
}

#[derive(Debug, Deserialize)]
pub struct BankSettings {
    pub app_token: String,
    pub user_token: String,
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
    pub bank: BankSettings,
    pub rich: RichSettings,
    pub payee: PayeeSettings,
}

const SETTINGS_FILE_PATH: &str = "./Settings.toml";

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let builder = Config::builder().add_source(File::with_name(SETTINGS_FILE_PATH));
        match builder.build() {
            Ok(c) => c.try_deserialize(),
            Err(e) => Err(e),
        }
    }
}
