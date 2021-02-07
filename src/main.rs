use std::error::Error;
use std::process;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
enum TransactionType {
    Credit,
    Debit,
}

impl std::str::FromStr for TransactionType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "C" => Ok(TransactionType::Credit),
            "D" => Ok(TransactionType::Debit),
            _ => Err(format!("'{}' is not a valid value for TransactionType", s)),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Transaction {
    amount: String,
    card: String,
    conversion_charge: String,
    details: String,
    foreign_currency_amount: String,
    processed_date: String,
    transaction_date: String,
    #[serde(with = "serde_with::rust::display_fromstr")]
    r#type: TransactionType,
}

fn example() -> Result<(), Box<dyn Error>> {
    let mut rdr = csv::Reader::from_path("./in.csv")?;

    for result in rdr.deserialize() {
        let transaction: Transaction = result?;
        println!("{:?}", transaction);
    }
    Ok(())
}

fn main() {
    if let Err(err) = example() {
        println!("error running example: {}", err);
        process::exit(1);
    }
}
