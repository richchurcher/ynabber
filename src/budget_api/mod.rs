use crate::bank_api::AkahuTransaction;
use regex::Regex;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use ynab_api::apis::{client::APIClient, configuration::ApiKey, configuration::Configuration};
use ynab_api::models::{
    save_transaction::Cleared, SaveTransaction, SaveTransactionsResponse, SaveTransactionsWrapper,
};

use crate::transaction_cache::{TransactionCache, TransactionCacheItem};

pub struct BudgetAPI {
    budget_id: String,
    client: APIClient,
    payee_regex: HashMap<String, String>,
    transaction_cache: TransactionCache,
}

#[derive(Debug)]
pub enum BudgetError {
    ApiError(String),
    DryRun,
}

impl fmt::Display for BudgetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BudgetError::DryRun => write!(f, "DRY RUN"),
            BudgetError::ApiError(msg) => write!(f, "BudgetAPI error: {}", msg),
        }
    }
}

impl Error for BudgetError {}

impl BudgetAPI {
    pub fn new(
        access_token: &str,
        budget_id: &str,
        payee_regex: &HashMap<String, String>,
    ) -> Result<BudgetAPI, Box<dyn Error>> {
        let ynab_config = Configuration {
            api_key: Some(ApiKey {
                prefix: Some("Bearer".to_string()),
                key: access_token.to_owned(),
            }),
            ..Default::default()
        };
        Ok(BudgetAPI {
            budget_id: budget_id.to_owned(),
            client: APIClient::new(ynab_config),
            payee_regex: payee_regex.to_owned(),
            transaction_cache: TransactionCache::new(
                // TODO: fix
                "/home/basie/.cache/ynabber/.transaction_cache",
            )?,
        })
    }

    pub fn get_latest_transaction(
        &mut self,
        budget_id: &str,
        account_id: &str,
    ) -> Result<(), Box<dyn Error>> {
        let transactions = self
            .client
            .transactions_api()
            .get_transactions_by_account(budget_id, account_id, None, None, None)
            .unwrap_or_else(|e| {
                panic!("Couldn't find latest processed transaction: {:#?}", e);
            });
        println!("{:#?}", transactions);
        Ok(())
    }

    pub fn get_payees(&mut self) -> Result<(), Box<dyn Error>> {
        let response = self.client.payees_api().get_payees(&self.budget_id, None);
        for payee in response.unwrap().data.payees {
            println!("{:?}", payee);
        }
        Ok(())
    }

    pub fn create_transaction(
        &mut self,
        transaction_wrapper: SaveTransactionsWrapper,
        dry_run: bool,
    ) -> Result<SaveTransactionsResponse, BudgetError> {
        if dry_run {
            println!("## DRY RUN ##");
            println!("{:#?}", transaction_wrapper);
            return Err(BudgetError::DryRun);
        }

        let result = match self
            .client
            .transactions_api()
            .create_transaction(&self.budget_id, transaction_wrapper)
        {
            Ok(r) => r,
            // TODO: ynab_api's errors could be Reqwest, Serde, or Io: find a better way to provide
            // more information?
            Err(_) => {
                return Err(BudgetError::ApiError(
                    "Unable to create transaction.".to_owned(),
                ))
            }
        };

        println!("{:#?}", result);
        Ok(result)
    }

    pub fn find_payee_id(&mut self, details: &str) -> Option<String> {
        for (k, v) in self.payee_regex.iter() {
            let re = Regex::new(v).unwrap();
            if re.is_match(details) {
                println!("MATCH: {}: {} ({})", k, v, details);
                return Some(k.to_string());
            }
        }
        None
    }

    pub fn create_transaction_from_akahu_transaction(
        &mut self,
        account_id: &str,
        akahu_transaction: &AkahuTransaction,
        dry_run: bool,
    ) -> Result<(), Box<dyn Error>> {
        let payee_id = self.find_payee_id(&akahu_transaction.description);
        // Only use a name if no id matched
        let payee_name = match payee_id {
            Some(_) => None,
            None => Some(akahu_transaction.description.to_owned()),
        };

        let ynab_result = match self.create_transaction(
            SaveTransactionsWrapper {
                transaction: Some(SaveTransaction {
                    account_id: account_id.to_owned(),
                    amount: (akahu_transaction.amount * 1000.0) as i64,
                    approved: Some(true),
                    category_id: None,
                    cleared: Some(Cleared::Uncleared),
                    date: akahu_transaction.date.to_string(),
                    flag_color: None,
                    import_id: None,
                    memo: Some("🤖".to_owned()),
                    payee_id,
                    payee_name,
                }),
                transactions: None,
            },
            dry_run,
        ) {
            Ok(r) => r,
            Err(e) => match e {
                BudgetError::DryRun => {
                    println!("Akahu transaction ID: {}", akahu_transaction._id);
                    return Ok(());
                }
                BudgetError::ApiError(_) => return Err(e.into()),
            },
        };

        let ynab_transaction_id = ynab_result
            .data
            .transaction
            .ok_or("No transaction detail available.")?
            .id;

        let latest_transaction = TransactionCacheItem {
            akahu_account_id: akahu_transaction._account.to_owned(),
            akahu_transaction_id: akahu_transaction._id.to_owned(),
            transaction_date: akahu_transaction.date,
            ynab_account_id: account_id.to_string(),
            ynab_transaction_id,
        };
        println!(
            "Success, so update transaction cache to {:#?}",
            latest_transaction
        );
        self.transaction_cache
            .set_latest_transaction(&latest_transaction)?;

        Ok(())
    }
}
