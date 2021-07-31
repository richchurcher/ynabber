mod budget_api;
mod settings;
mod statement_line;

use budget_api::BudgetAPI;
use settings::Settings;
use std::process;

#[tokio::main]
async fn main() {
    let settings = match Settings::new() {
        Ok(s) => s,
        Err(e) => panic!("{}", e),
    };
    let mut budget = match BudgetAPI::new(
        &settings.ynab.access_token,
        &settings.rich.budget_id,
        &settings.payee.regex,
    ) {
        Ok(b) => b,
        Err(_) => panic!("Error initialising budget."),
    };

    let statement_lines = match budget.read_all_from_csv("./in.csv") {
        Ok(lines) => lines,
        Err(_) => panic!("Error reading from CSV."),
    };

    let account_suffix = statement_lines[0].card.split('-').next_back().unwrap();
    let account_id = match account_suffix {
        "9132" => &settings.rich.visa_personal_id,
        "8834" => &settings.rich.visa_business_id,
        _ => panic!("Unable to determine account number."),
    };

    for line in statement_lines.iter() {
        if let Err(err) = budget
            .create_transaction_from_statement_line(
                account_id, line, // dry run
                true,
            )
            .await
        {
            println!("Error creating transaction: {}", err);
            process::exit(1);
        }
    }

    // if let Err(err) = budget.get_payees().await {
    //     println!("Couldn't get payees: {}", err);
    //     process::exit(1);
    // }
}
