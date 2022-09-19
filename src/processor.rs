use rust_decimal::Decimal;
use serde::{Deserialize, Deserializer, Serialize};
use std::str::FromStr;

use crate::account::Accounts;
use crate::error::{Error, Result};
use crate::io::{CsvReader, CsvWriter};

// This deserializer is needed to make sure precision is up to 4 decimal places.
fn deserialize_amount<'de, D>(amount: D) -> std::result::Result<Option<Decimal>, D::Error>
where
    D: Deserializer<'de>,
{
    let buf = String::deserialize(amount)?;
    if buf.is_empty() {
        return Ok(None);
    }

    let decimal = Decimal::from_str(buf.as_str())
        .map_err(serde::de::Error::custom)?
        .normalize();
    if decimal.scale() > 4 {
        return Err(serde::de::Error::custom(
            "only up to four decimal places for precision is allowed",
        ));
    }
    Ok(Some(decimal))
}

/// Record from csv.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub(crate) struct Record {
    #[serde(rename = "type")]
    ty: String,
    client: u16,
    tx: u32,
    #[serde(deserialize_with = "deserialize_amount")]
    amount: Option<Decimal>,
}

/// Processor processes the transactions.
pub struct Processor {
    reader: CsvReader,
    writer: CsvWriter,
    accounts: Accounts,
}

impl Processor {
    pub fn new() -> Self {
        Self {
            reader: CsvReader::new(),
            writer: CsvWriter::new(),
            accounts: Accounts::new(),
        }
    }

    /// Start reading transactions using the Reader and writing results using the Writer.
    pub fn start(mut self) {
        match self.reader.read() {
            Ok(records) => {
                for record in records.clone() {
                    if let Err(e) = self.process(record) {
                        log::error!("{:?}", e);
                    }
                }
                self.writer
                    .write(self.accounts.accounts())
                    .expect("failed to write")
            }
            Err(e) => panic!("failed to read: {}", e),
        }
    }

    /// Process a single record.
    fn process(&mut self, record: Record) -> Result<()> {
        match record.ty.to_lowercase().as_str() {
            "deposit" => {
                let amount = record.amount.ok_or(Error::InvalidData)?;
                self.accounts.deposit(record.client, amount, record.tx)?
            }
            "withdrawal" => {
                let amount = record.amount.ok_or(Error::InvalidData)?;
                self.accounts.withdraw(record.client, amount, record.tx)?
            }
            "dispute" => self.accounts.dispute(record.client, record.tx)?,
            "resolve" => self.accounts.resolve(record.client, record.tx)?,
            "chargeback" => self.accounts.chargeback(record.client, record.tx)?,
            _ => return Err(Error::InvalidData),
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::error::Error;
    use crate::processor::{Processor, Record};
    use csv::Reader;
    use rust_decimal::Decimal;

    // Creates Records from strings.
    macro_rules! records {
        ($($str:tt), *) => {{
            let mut data = String::from("type,client,tx,amount\n");
            $(
                data.push_str($str);
                data.push_str("\n");
            )*
            // create_records(data.as_str())
            let mut records = Vec::new();
            let mut rdr = Reader::from_reader(data.as_bytes());
            for result in rdr.deserialize() {
                let record: Record = result.unwrap();
                records.push(record);
            }
            records
        }}
    }

    // Creates Decimal, optionally with a given precision.
    macro_rules! dec {
        ($num:expr, $prec:expr) => {{
            Decimal::new($num, $prec)
        }};
        ($num:expr) => {{
            Decimal::new($num, 0)
        }};
    }

    #[test]
    fn deposit_success() {
        let records = records!(
            "deposit,1,61,100",
            "deposit,2,62,100",
            "deposit,3,64,120",
            "deposit,1,66,50"
        );
        let mut processor = Processor::new();
        for record in records {
            processor.process(record).unwrap();
        }
        assert_eq!(
            processor.accounts.account(1).unwrap().available(),
            dec!(150)
        );
        assert_eq!(
            processor.accounts.account(2).unwrap().available(),
            dec!(100)
        );
        assert_eq!(
            processor.accounts.account(3).unwrap().available(),
            dec!(120)
        );
    }

    #[test]
    fn withdrawal_success() {
        let records = records!(
            "deposit,1,61,200",
            "deposit,2,62,200",
            "withdrawal,1,65,150",
            "withdrawal,2,70,20"
        );
        let mut processor = Processor::new();
        for record in records {
            processor.process(record).unwrap();
        }
        assert_eq!(processor.accounts.account(1).unwrap().available(), dec!(50));
        assert_eq!(
            processor.accounts.account(2).unwrap().available(),
            dec!(180)
        );
    }

    #[test]
    fn withdrawal_fail() {
        let records = records!(
            "deposit,1,61,100",
            "deposit,2,90,200",
            "withdrawal,1,91,150"
        );
        let mut processor = Processor::new();
        let mut records_iter = records.into_iter();
        processor.process(records_iter.next().unwrap()).unwrap();
        processor.process(records_iter.next().unwrap()).unwrap();
        assert_eq!(
            processor.process(records_iter.next().unwrap()),
            Err(Error::InsufficientFunds)
        );
        assert_eq!(
            processor.accounts.account(1).unwrap().available(),
            dec!(100)
        );
        assert_eq!(
            processor.accounts.account(2).unwrap().available(),
            dec!(200)
        );

        // We try to withdraw from recently created client account.
        let records = records!("withdrawal,1,62,150", "deposit,1,61,100");
        let mut processor = Processor::new();
        let mut records_iter = records.into_iter();
        assert_eq!(
            processor.process(records_iter.next().unwrap()),
            Err(Error::InsufficientFunds)
        );
        assert_eq!(
            processor.accounts.account(1).unwrap().available(),
            Decimal::ZERO
        );
    }

    #[test]
    fn dispute() {
        // Dispute should decrement available.
        let records = records!(
            "deposit,1,61,100",
            "deposit,2,63,100",
            "deposit,1,64,120",
            "dispute,1,61,",
            "deposit,1,66,100"
        );
        let mut processor = Processor::new();
        for record in records {
            processor.process(record).unwrap();
        }
        assert_eq!(
            processor.accounts.account(1).unwrap().available(),
            dec!(220)
        );
        assert_eq!(processor.accounts.account(1).unwrap().total(), dec!(320));
        assert_eq!(
            processor.accounts.account(2).unwrap().available(),
            dec!(100)
        );
        assert_eq!(processor.accounts.account(2).unwrap().total(), dec!(100));

        // We get funds back after resolving.
        let records = records!("resolve,1,61,", "deposit,1,69,100");
        for record in records {
            processor.process(record).unwrap();
        }
        assert_eq!(
            processor.accounts.account(1).unwrap().available(),
            dec!(420)
        );
        assert_eq!(processor.accounts.account(1).unwrap().total(), dec!(420));

        // Try to resolve transaction that is not being desputed.
        let records = records!(
            "deposit,1,61,100",
            "deposit,1,64,120",
            "resolve,1,61,",
            "deposit,1,66,100"
        );
        let mut processor = Processor::new();
        for record in records {
            processor.process(record).unwrap();
        }
        assert_eq!(
            processor.accounts.account(1).unwrap().available(),
            dec!(320)
        );
        assert_eq!(processor.accounts.account(1).unwrap().total(), dec!(320));
    }

    #[test]
    fn chargeback() {
        // Dispute should decrement available.
        let records = records!(
            "deposit,1,61,100",
            "deposit,2,63,100",
            "deposit,1,64,120",
            "dispute,1,61,",
            "deposit,1,66,100"
        );
        let mut processor = Processor::new();
        for record in records {
            processor.process(record).unwrap();
        }
        assert_eq!(
            processor.accounts.account(1).unwrap().available(),
            dec!(220)
        );
        assert_eq!(processor.accounts.account(1).unwrap().total(), dec!(320));
        assert_eq!(
            processor.accounts.account(2).unwrap().available(),
            dec!(100)
        );
        assert_eq!(processor.accounts.account(2).unwrap().total(), dec!(100));

        // We get a chargeback and trying to deposit fails because account is frozen.
        let records = records!("chargeback,1,61,", "deposit,1,69,100");
        for record in records {
            processor.process(record).unwrap();
        }
        assert_eq!(
            processor.accounts.account(1).unwrap().available(),
            dec!(220)
        );
        assert_eq!(processor.accounts.account(1).unwrap().total(), dec!(220));

        // Try to chargeback a non-desputed transaction.
        let records = records!(
            "deposit,1,61,100",
            "deposit,1,64,100",
            "chargeback,1,61,",
            "deposit,1,65,100"
        );
        let mut processor = Processor::new();
        for record in records {
            processor.process(record).unwrap();
        }

        // Chargeback gets ignored and we can still process other records.
        assert_eq!(
            processor.accounts.account(1).unwrap().available(),
            dec!(300)
        );
        assert_eq!(processor.accounts.account(1).unwrap().total(), dec!(300));
    }

    #[test]
    fn nonexistent_transaction() {
        // Try to dispute a non existent transaction.
        let records = records!(
            "deposit,1,61,100",
            "dispute,1,33,",
            "chargeback,1,33,",
            "deposit,1,62,100"
        );
        let mut processor = Processor::new();
        for record in records {
            processor.process(record).unwrap();
        }
        assert_eq!(
            processor.accounts.account(1).unwrap().available(),
            dec!(200)
        );
        assert_eq!(processor.accounts.account(1).unwrap().total(), dec!(200));
    }

    #[test]
    fn precision() {
        let records = records!("deposit,1,61,4.321", "withdrawal,1,62,1.001");
        let mut processor = Processor::new();
        for record in records {
            processor.process(record).unwrap();
        }
        assert_eq!(
            processor.accounts.account(1).unwrap().available(),
            dec!(332, 2)
        );
        assert_eq!(processor.accounts.account(1).unwrap().total(), dec!(332, 2));

        // Invalid precision.
        let mut data = String::from("type,client,tx,amount\n");
        data.push_str("deposit,1,61,4.32111");
        data.push_str("\n");

        let mut rdr = Reader::from_reader(data.as_bytes());
        let err_msg = rdr.deserialize::<Record>().next().unwrap();
        assert!(err_msg
            .err()
            .unwrap()
            .to_string()
            .contains("only up to four decimal places for precision is allowed"));
    }

    #[test]
    fn reusing_tx() {
        // Transaction ID is globally unique so reusing it causes error.
        let records = records!("deposit,1,61,4.321", "deposit,1,61,1.001");
        let mut processor = Processor::new();

        let mut records_iter = records.into_iter();
        processor.process(records_iter.next().unwrap()).unwrap();

        assert_eq!(
            processor.process(records_iter.next().unwrap()),
            Err(Error::TxExists)
        )
    }
}
