use chrono::{DateTime, Utc};
use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, Write};
use std::path::Path;

#[derive(Debug)]
pub struct TransactionCacheItem {
    pub akahu_account_id: String,
    pub akahu_transaction_id: String,
    pub transaction_date: DateTime<Utc>,
    pub ynab_account_id: String,
    pub ynab_transaction_id: String,
}

pub struct TransactionCache {
    path: &'static Path,
}

impl TransactionCache {
    pub fn new(cache_path: &'static str) -> Result<TransactionCache, Box<dyn Error>> {
        Ok(TransactionCache {
            path: Path::new(cache_path),
        })
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

    // TODO: needs to allow for multiple accounts per file
    pub fn set_latest_transaction(&self, t: &TransactionCacheItem) -> Result<(), Box<dyn Error>> {
        let mut file = File::create(&self.path)?;
        let contents = format!(
            "{}\n{}\n{}\n{}\n{}",
            t.akahu_account_id,
            t.ynab_account_id,
            t.akahu_transaction_id,
            t.ynab_transaction_id,
            t.transaction_date
        );

        Ok(file
            .write_all(contents.as_bytes())
            .expect("Couldn't write to cache file."))
    }
}
