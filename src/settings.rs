use config::{Config, ConfigError, File};
use home::home_dir;
use serde::Deserialize;
use std::{collections::HashMap, env, path::PathBuf};

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
    pub accounts: HashMap<String, AccountSettings>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AccountSettings {
    pub akahu_id: String,
    pub ynab_id: String,
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

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let home = home_dir().unwrap_or(PathBuf::from("."));
        let mut config_path = match env::var("XDG_CONFIG_HOME") {
            Ok(p) => PathBuf::from(p),
            Err(_) => {
                let mut p = PathBuf::from(home);
                p.push(".config");
                p
            }
        };
        config_path.push("ynabber");
        config_path.push("Settings.toml");
        let builder = Config::builder().add_source(File::from(config_path));
        match builder.build() {
            Ok(c) => c.try_deserialize(),
            Err(e) => Err(e),
        }
    }
}
