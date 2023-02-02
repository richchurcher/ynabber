mod bank_api;
mod budget_api;
mod settings;
mod transaction_cache;

use bank_api::BankAPI;
use budget_api::BudgetAPI;
use settings::Settings;
use std::process;

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
    let bank_transactions = bank
        .latest_transactions(&settings.rich.accounts["visa_personal_akahu_id"])
        // .latest_transactions(&settings.rich.accounts["visa_business_akahu_id"])
        .unwrap_or_else(|e| {
            panic!("Error reading Akahu API: {}", e);
        });
    println!("{:#?}", bank_transactions);

    // NOTE: business
    // let account_suffix = "8834";
    // NOTE: personal
    let account_suffix = "1380";
    if !&settings.rich.cards.contains_key(account_suffix) {
        panic!("Unrecognised account suffix: {}", account_suffix);
    }
    let account_key = &settings.rich.cards[account_suffix];
    let account_id = &settings.rich.accounts[account_key];

    for t in bank_transactions.iter() {
        if let Err(err) = budget
            // NOTE: boolean value denotes dry run (or not)
            .create_transaction_from_akahu_transaction(account_id, t, true)
        {
            println!("Error creating transaction: {}", err);
            process::exit(1);
        }
    }

    // let yts = budget
    //     .get_latest_transaction(&settings.rich.budget_id, account_id)
    //     .unwrap_or_else(|_e| {
    //         panic!("Nope.");
    //     });
    // println!("{:#?}", yts);

    Ok(())
}
