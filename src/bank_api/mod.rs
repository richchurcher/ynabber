use chrono::{DateTime, Utc};
use reqwest::blocking::Client;
use reqwest::header;
use serde::Deserialize;
use serde_with::{serde_as, DisplayFromStr};
use std::error::Error;

use crate::transaction_cache::TransactionCache;

#[derive(Clone, Debug, Deserialize, PartialEq)]
enum AkahuCategoryComponentType {
    Base,
    Group,
    PFM,
}

#[serde_as]
#[derive(Clone, Debug, Deserialize)]
pub struct AkahuCategoryComponent {
    pub name: String,
    #[serde_as(as = "DisplayFromStr")]
    r#type: AkahuCategoryComponentType,
}

impl std::str::FromStr for AkahuCategoryComponentType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "nzfcc:base" => Ok(AkahuCategoryComponentType::Base),
            "nzfcc:group" => Ok(AkahuCategoryComponentType::Group),
            "nzfcc:pfm" => Ok(AkahuCategoryComponentType::PFM),
            _ => Err(format!(
                "'{}' is not a valid value for AkahuCategoryComponentType",
                s
            )),
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct AkahuCategory {
    _id: String,
    components: Vec<AkahuCategoryComponent>,
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
            transaction_cache: TransactionCache::new(".transaction_cache")?,
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

            let res = self.client.get(url).send()?;
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

            println!("{:?}", atr.cursor);
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn akahu_category_component_type_base() {
        assert_eq!(
            AkahuCategoryComponentType::from_str("nzfcc:base").unwrap(),
            AkahuCategoryComponentType::Base
        );
    }

    #[test]
    fn akahu_category_component_type_group() {
        assert_eq!(
            AkahuCategoryComponentType::from_str("nzfcc:group").unwrap(),
            AkahuCategoryComponentType::Group
        );
    }

    #[test]
    fn akahu_category_component_type_pfm() {
        assert_eq!(
            AkahuCategoryComponentType::from_str("nzfcc:pfm").unwrap(),
            AkahuCategoryComponentType::PFM
        );
    }

    #[test]
    fn akahu_category_component_type_invalid() {
        assert!(AkahuCategoryComponentType::from_str("definitely not a type").is_err());
    }
}
