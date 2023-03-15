use chrono::{DateTime, Utc};
use reqwest::blocking::Client;
use reqwest::header;
use serde::Deserialize;
use serde_with::serde_as;
use std::{collections::HashMap, error::Error};

use crate::transaction_cache::TransactionCache;

#[serde_as]
#[derive(Clone, Debug, Deserialize)]
pub struct AkahuCategoryGroup {
    pub _id: String,
    pub name: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AkahuCategory {
    _id: String,
    name: String,
    groups: HashMap<String, AkahuCategoryGroup>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AkahuMerchant {
    pub _id: String,
    pub name: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AkahuConversion {
    pub amount: f64,
    pub currency: String,
    pub fee: f64,
    pub rate: f64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AkahuMetadata {
    pub conversion: Option<AkahuConversion>,
    pub logo: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AkahuTransaction {
    pub _id: String,
    pub _account: String,
    pub _user: String,
    pub _connection: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub date: DateTime<Utc>,
    pub hash: String,
    pub description: String,
    pub amount: f64,
    r#type: String,
    pub merchant: Option<AkahuMerchant>,
    pub category: Option<AkahuCategory>,
    pub meta: AkahuMetadata,
}

#[derive(Debug, Deserialize)]
pub struct AkahuCursor {
    next: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AkahuTransactionResponse {
    pub cursor: Option<AkahuCursor>,
    pub items: Vec<AkahuTransaction>,
    pub success: bool,
}

pub struct BankAPI {
    app_token: String,
    base_url: String,
    client: Client,
    transaction_cache: TransactionCache,
    user_token: String,
}

impl BankAPI {
    pub fn new(app_token: &str, user_token: &str) -> Result<BankAPI, Box<dyn Error>> {
        let mut headers = header::HeaderMap::new();
        let t = format!("Bearer {}", user_token);
        headers.insert(header::AUTHORIZATION, header::HeaderValue::from_str(&t)?);
        headers.insert("X-Akahu-ID", header::HeaderValue::from_str(app_token)?);

        let client = Client::builder().default_headers(headers).build()?;

        Ok(BankAPI {
            app_token: app_token.to_string(),
            base_url: "https://api.akahu.io/v1".to_string(),
            client,
            transaction_cache: TransactionCache::new()?,
            user_token: user_token.to_string(),
        })
    }

    pub fn latest_transactions(
        &mut self,
        account_id: &str,
    ) -> Result<Vec<AkahuTransaction>, Box<dyn Error>> {
        let mut transactions: Vec<AkahuTransaction> = vec![];
        let mut cursor = String::new();
        let cached_transaction = self.transaction_cache.get_latest_transaction(account_id)?;

        loop {
            let mut url = format!("{}/accounts/{}/transactions", self.base_url, account_id);
            if cursor != "" {
                url.push_str(&format!("?cursor={}", cursor));
            }

            let res = self.client.get(url.to_owned()).send()?;
            let atr = res.json::<AkahuTransactionResponse>()?;

            let mut new_transactions: Vec<AkahuTransaction> = atr
                .items
                .clone()
                .into_iter()
                .take_while(|x| x._id != cached_transaction.akahu_transaction_id)
                .collect();
            let total_new = new_transactions.len();
            transactions.append(&mut new_transactions);

            if total_new != atr.items.len() {
                // We must have encountered a known transaction ID, stop looking for more
                println!(
                    "Found Akahu ID {} at index {}, stopping API search.",
                    cached_transaction.akahu_transaction_id, total_new
                );
                break;
            }

            cursor = match atr.cursor {
                Some(c) => match c.next {
                    Some(n) => n,
                    None => {
                        // No more transactions to check: exit
                        println!(
                            "Didn't find any transaction with cached ID: {}",
                            cached_transaction.akahu_transaction_id
                        );
                        break;
                    }
                },
                None => "".to_string(),
            };
        }

        println!("Total new transactions found: {}", transactions.len());
        println!(
            "(while searching for cached Akahu ID: {})",
            cached_transaction.akahu_transaction_id
        );

        Ok(transactions)
    }
}
