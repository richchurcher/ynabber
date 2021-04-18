mod budget_api;
mod settings;
mod transaction;

use budget_api::BudgetAPI;
use settings::Settings;
use std::error::Error;
use std::process;
use transaction::Transaction;

fn example() -> Result<(), Box<dyn Error>> {
    let mut rdr = csv::Reader::from_path("./in.csv")?;

    for result in rdr.deserialize() {
        let transaction: Transaction = result?;
        println!("{:#?}", transaction);
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    let settings = match Settings::new() {
        Ok(s) => s,
        Err(_) => panic!("Error loading settings. Check your `./Settings.toml`."),
    };
    println!("Settings? {:?}", settings.ynab.access_token);

    // if let Err(err) = example() {
    //     println!("error running example: {}", err);
    //     process::exit(1);
    // }

    let mut budget = BudgetAPI::new("").unwrap();
    if let Err(err) = budget
        .request("https://jsonplaceholder.typicode.com/todos/1")
        .await
    {
        println!("error making request: {}", err);
        process::exit(1);
    }
}
