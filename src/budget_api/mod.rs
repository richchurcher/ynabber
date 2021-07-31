use crate::statement_line::StatementLine;
use regex::Regex;
use rusty_ulid::generate_ulid_string;
use std::collections::HashMap;
use std::error::Error;
use ynab_api::apis::{client::APIClient, configuration::ApiKey, configuration::Configuration};
use ynab_api::models::{save_transaction::Cleared, SaveTransaction, SaveTransactionsWrapper};

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct Todo {
    user_id: i32,
    id: i32,
    title: String,
    completed: bool,
}

pub struct BudgetAPI {
    budget_id: String,
    client: APIClient,
    payee_regex: HashMap<String, String>,
}

impl BudgetAPI {
    pub fn new(
        access_token: &str,
        budget_id: &str,
        payee_regex: &HashMap<String, String>,
    ) -> Result<BudgetAPI, Box<dyn Error>> {
        let mut ynab_config = Configuration::default();
        ynab_config.api_key = Some(ApiKey {
            prefix: Some("Bearer".to_string()),
            key: access_token.to_owned(),
        });
        Ok(BudgetAPI {
            budget_id: budget_id.to_owned(),
            client: APIClient::new(ynab_config),
            payee_regex: payee_regex.to_owned(),
        })
    }

    // pub async fn request(&mut self, url: &str) -> Result<(), Box<dyn Error>> {
    //     let accounts = self.client.accounts_api().get_accounts("123", None);
    // let response = self
    //     .client
    //     .get(format!("{}{}", self.base_url, url))
    //     .header(AUTHORIZATION, format!("Bearer {}", self.access_token))
    //     .send()
    //     .await?
    //     .json()
    //     .await?;
    // println!("{:?}", response);
    //     Ok(())
    // }

    // pub async fn get_budgets(&mut self) -> Result<(), Box<dyn Error>> {
    //     let budgets = self.client.budgets_api().get_budgets();
    // let response = self
    //     .client
    //     .get(format!("{}{}", self.base_url, url))
    //     .header(AUTHORIZATION, format!("Bearer {}", self.access_token))
    //     .send()
    //     .await?
    //     .json()
    //     .await?;
    //     println!("{:#?}", budgets);
    //     Ok(())
    // }

    // pub async fn get_budget(&mut self, budget_id: &str) -> Result<(), Box<dyn Error>> {
    //     let budget = self.client.budgets_api().get_budget_by_id(budget_id, None);
    //     println!("{:#?}", budget);
    //     Ok(())
    // }
    pub async fn get_payees(&mut self) -> Result<(), Box<dyn Error>> {
        let response = self.client.payees_api().get_payees(&self.budget_id, None);
        // .get(format!("{}{}", self.base_url, url))
        // .header(AUTHORIZATION, format!("Bearer {}", self.access_token))
        // .send()
        // .await?
        // .json()
        // .await?;
        // println!("{:?}", response.unwrap().data.payees[0]);
        for payee in response.unwrap().data.payees {
            println!("{:?}", payee);
        }
        Ok(())
    }

    pub async fn create_transaction(
        &mut self,
        transaction_wrapper: SaveTransactionsWrapper,
        dry_run: bool,
    ) -> Result<(), Box<dyn Error>> {
        if dry_run {
            println!("## DRY RUN ##");
            println!("{:#?}", transaction_wrapper);
            return Ok(());
        }

        let result = self
            .client
            .transactions_api()
            .create_transaction(&self.budget_id, transaction_wrapper);
        println!("{:#?}", result);
        Ok(())
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

    pub async fn create_transaction_from_statement_line(
        &mut self,
        account_id: &str,
        statement_line: &StatementLine,
        dry_run: bool,
    ) -> Result<(), Box<dyn Error>> {
        let mut amount = match statement_line.amount.to_owned().parse::<f64>() {
            Ok(a) => a,
            Err(e) => panic!("Error converting '{}' to i64: {}", statement_line.amount, e),
        };
        let payee_id = self.find_payee_id(&statement_line.details);
        // Only use a name if no id matched
        let payee_name = match payee_id {
            Some(_) => None,
            None => Some(statement_line.details.to_owned()),
        };
        if statement_line.is_debit() {
            amount = amount * -1.0;
        }
        let memo = format!("yn#{}", generate_ulid_string());

        if let Err(err) = self
            .create_transaction(
                SaveTransactionsWrapper {
                    transaction: Some(SaveTransaction {
                        account_id: account_id.to_owned(),
                        amount: (amount * 1000.0) as i64,
                        approved: Some(true),
                        category_id: None,
                        cleared: Some(Cleared::Uncleared),
                        date: statement_line.transaction_date.to_string(),
                        flag_color: None,
                        import_id: None,
                        memo: Some(memo),
                        payee_id: payee_id,
                        payee_name: payee_name,
                    }),
                    transactions: None,
                },
                dry_run,
            )
            .await
        {
            return Err(err.into());
        }
        Ok(())
    }

    // pub fn read_from_csv(&mut self, path: &str) -> Result<(), Box<dyn Error>> {
    //     let mut rdr = csv::Reader::from_path(path)?;
    //
    //     for result in rdr.deserialize() {
    //         let transaction: Transaction = result?;
    //         println!("{:#?}", transaction);
    //     }
    //
    //     Ok(())
    // }

    pub fn read_all_from_csv(&mut self, path: &str) -> Result<Vec<StatementLine>, Box<dyn Error>> {
        let mut rdr = csv::Reader::from_path(path)?;
        let mut lines: Vec<StatementLine> = Vec::new();

        for result in rdr.deserialize() {
            let t: StatementLine = result?;
            lines.push(t);
        }

        Ok(lines)
    }
}
