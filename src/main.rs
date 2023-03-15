mod bank_api;
mod budget_api;
mod settings;
mod transaction_cache;

use bank_api::BankAPI;
use budget_api::BudgetAPI;
use settings::Settings;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let settings = Settings::new().unwrap_or_else(|e| {
        panic!("Error getting configuration: {}", e);
    });
    let mut budget = BudgetAPI::new(
        &settings.ynab.access_token,
        &settings.rich.budget_id,
        &settings.payee.regex,
    )
    .unwrap_or_else(|e| {
        panic!("Error initialising budget API: {}", e);
    });
    let mut bank = BankAPI::new(&settings.bank.app_token, &settings.bank.user_token)
        .unwrap_or_else(|e| {
            panic!("Error initialising bank API: {}", e);
        });

    let accounts_to_check = [
        settings.rich.accounts["visa_business"].to_owned(),
        settings.rich.accounts["visa_personal"].to_owned(),
    ];
    for account in accounts_to_check.iter() {
        println!("Checking account {}...", account.akahu_id);
        let bank_transactions = bank
            .latest_transactions(&account.akahu_id)
            .unwrap_or_else(|e| {
                panic!("Error reading Akahu API: {}", e);
            });
        for t in bank_transactions.iter() {
            if let Err(e) =
                budget.create_transaction_from_akahu_transaction(&account.ynab_id, t, false)
            {
                panic!("Error creating transaction: {:?}", e);
            }
        }
    }

    Ok(())
}
