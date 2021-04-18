mod day_month_year_format;

use chrono::NaiveDate;
use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq)]
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
pub struct Transaction {
    pub amount: String,
    pub card: String,
    pub conversion_charge: String,
    pub details: String,
    pub foreign_currency_amount: String,
    #[serde(with = "day_month_year_format")]
    pub processed_date: NaiveDate,
    #[serde(with = "day_month_year_format")]
    pub transaction_date: NaiveDate,
    #[serde(with = "serde_with::rust::display_fromstr")]
    r#type: TransactionType,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn transaction_type_credit() {
        assert_eq!(
            TransactionType::from_str("C").unwrap(),
            TransactionType::Credit
        );
    }

    #[test]
    fn transaction_type_debit() {
        assert_eq!(
            TransactionType::from_str("D").unwrap(),
            TransactionType::Debit
        );
    }

    #[test]
    fn transaction_type_unknown_results_in_error() {
        assert!(TransactionType::from_str("X").is_err());
    }
}
