use chrono::{DateTime, Utc};
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, Write};
use std::path::PathBuf;

use home::home_dir;

#[derive(Clone, Debug)]
pub struct TransactionCacheItem {
    pub akahu_account_id: String,
    pub akahu_transaction_id: String,
    pub transaction_date: DateTime<Utc>,
    pub ynab_account_id: String,
    pub ynab_transaction_id: String,
}

pub struct TransactionCache {
    path: PathBuf,
}

impl TransactionCache {
    pub fn new() -> Result<TransactionCache, Box<dyn Error>> {
        let home = home_dir().unwrap_or(PathBuf::from("."));
        let mut cache_path = match env::var("XDG_CACHE_HOME") {
            Ok(p) => PathBuf::from(p),
            Err(_) => {
                let mut p = PathBuf::from(home);
                p.push(".cache");
                p
            }
        };
        cache_path.push("ynabber");
        cache_path.push(".transaction_cache");
        Ok(TransactionCache { path: cache_path })
    }

    pub fn get_latest_transaction(
        &self,
        account_id: &str,
    ) -> Result<TransactionCacheItem, Box<dyn Error>> {
        let file = File::open(&self.path)?;
        let lines = io::BufReader::new(file)
            .lines()
            .map(|x| x.unwrap())
            .collect::<Vec<String>>();
        let mut i = 0;
        while i < lines.len() {
            // Sample file:
            //
            // acc_cl2y4h0sy000009lc1g8eg7zk
            // c33bd5ef-2538-4ff2-b1b9-004b6ae7ccc9
            // trans_cl5uopvg808b509jr4k62ggth
            // 37f12dc1-6e83-4e3b-8811-262ab1396f03
            // 2022-07-18T12:00:00Z
            //
            // which equates to:
            //
            // akahu_account_id
            // ynab_account_id
            // akahu_transaction_id
            // ynab_transaction_id
            // transaction_date
            let account_cache_raw = &lines[i..i + 5];
            let transaction_date =
                DateTime::parse_from_rfc3339(&account_cache_raw[4])?.with_timezone(&Utc);

            if account_cache_raw[0] == account_id {
                return Ok(TransactionCacheItem {
                    akahu_account_id: account_cache_raw[0].to_owned(),
                    ynab_account_id: account_cache_raw[1].to_owned(),
                    akahu_transaction_id: account_cache_raw[2].to_owned(),
                    transaction_date,
                    ynab_transaction_id: account_cache_raw[3].to_owned(),
                });
            }

            i = i + 5;
        }

        return Err(format!("No such account ID found in cache: {}", account_id).into());
    }

    pub fn set_latest_transaction(&self, t: &TransactionCacheItem) -> Result<(), Box<dyn Error>> {
        // TODO: maybe generalise this to avoid repetition in get_latest_transaction
        // (or hell, stop using a flat file)
        let read_file = File::open(&self.path)?;
        let lines = io::BufReader::new(read_file)
            .lines()
            .map(|x| x.unwrap())
            .collect::<Vec<String>>();
        let mut cache_items: Vec<TransactionCacheItem> = vec![];
        let mut i = 0;
        while i < lines.len() {
            let account_cache_raw = &lines[i..i + 5];

            if account_cache_raw[0] == t.akahu_account_id {
                cache_items.push(t.clone());
            } else {
                let transaction_date =
                    DateTime::parse_from_rfc3339(&account_cache_raw[4])?.with_timezone(&Utc);
                cache_items.push(TransactionCacheItem {
                    akahu_account_id: account_cache_raw[0].to_owned(),
                    ynab_account_id: account_cache_raw[1].to_owned(),
                    akahu_transaction_id: account_cache_raw[2].to_owned(),
                    transaction_date,
                    ynab_transaction_id: account_cache_raw[3].to_owned(),
                });
            }

            i = i + 5;
        }

        let mut write_file = File::create(&self.path)?;
        let mut contents = String::new();
        for item in cache_items {
            let rfc3339 = item.transaction_date.to_rfc3339();
            contents.push_str(&format!(
                "{}\n{}\n{}\n{}\n{}\n",
                item.akahu_account_id,
                item.ynab_account_id,
                item.akahu_transaction_id,
                item.ynab_transaction_id,
                rfc3339,
            ));
        }

        Ok(write_file
            .write_all(contents.as_bytes())
            .expect("Couldn't write to cache file."))
    }
}
