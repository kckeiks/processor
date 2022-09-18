use serde::{Deserialize, Serialize};

use crate::account::Accounts;
use crate::error::{Error, Result};
use crate::io::{CsvReader, CsvWriter};

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Record {
    #[serde(rename = "type")]
    pub(crate) ty: String,
    pub(crate) client: u16,
    pub(crate) tx: u32,
    pub(crate) amount: f64,
}

/// Processor processes the transactions.
pub struct Processor {
    reader: CsvReader,
    writer: CsvWriter,
    accounts: Accounts,
}

impl Processor {
    pub(crate) fn new() -> Self {
        Self {
            reader: CsvReader::new(),
            writer: CsvWriter::new(),
            accounts: Accounts::new(),
        }
    }

    /// Start reading transactions using the Reader and writing results using the Writer.
    pub(crate) fn start(mut self) {
        match self.reader.read() {
            Ok(records) => self.writer.write(records).expect("failed to write"),
            Err(e) => panic!("failed to read: {}", e),
        }
    }

    /// Process a single record.
    pub(crate) fn process(&mut self, record: Record) -> Result<()> {
        let mut account = self.accounts.get(record.client)?;
        match record.ty.to_lowercase().as_str() {
            "deposit" => account.deposit(record.amount)?,
            "withdrawal" => account.withdraw(record.amount)?,
            "dispute" => account.dispute(record.amount)?,
            "resolve" => account.resolve(record.amount)?,
            "chargeback" => account.chargeback(record.amount)?,
            _ => return Err(Error::InvalidData),
        }
        self.accounts.update(account)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::error::Error;
    use crate::processor::{Processor, Record};
    use csv::Reader;

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

    #[test]
    fn deposit_success() {
        let records = records!(
            "deposit,1,61,100",
            "deposit,2,62,100",
            "deposit,3,61,120",
            "deposit,1,62,50"
        );
        let mut processor = Processor::new();
        for record in records {
            processor.process(record).unwrap();
        }
        assert_eq!(processor.accounts.get(1).unwrap().available(), 150 as f64);
        assert_eq!(processor.accounts.get(2).unwrap().available(), 100 as f64);
        assert_eq!(processor.accounts.get(3).unwrap().available(), 120 as f64);
    }

    #[test]
    fn withdrawal_success() {
        let records = records!(
            "deposit,1,61,200",
            "deposit,2,62,200",
            "withdrawal,1,62,150",
            "withdrawal,2,62,20"
        );
        let mut processor = Processor::new();
        for record in records {
            processor.process(record).unwrap();
        }
        assert_eq!(processor.accounts.get(1).unwrap().available(), 50 as f64);
        assert_eq!(processor.accounts.get(2).unwrap().available(), 180 as f64);
    }

    #[test]
    fn withdrawal_fail() {
        let records = records!(
            "deposit,1,61,100",
            "deposit,2,62,200",
            "withdrawal,1,62,150"
        );
        let mut processor = Processor::new();
        let mut records_iter = records.into_iter();
        processor.process(records_iter.next().unwrap()).unwrap();
        processor.process(records_iter.next().unwrap()).unwrap();
        assert_eq!(
            processor.process(records_iter.next().unwrap()),
            Err(Error::InsufficientFunds)
        );
        assert_eq!(processor.accounts.get(1).unwrap().available(), 100 as f64);
        assert_eq!(processor.accounts.get(2).unwrap().available(), 200 as f64);

        // We try to withdraw from recently created client account.
        let records = records!("withdrawal,1,62,150", "deposit,1,61,100");
        let mut processor = Processor::new();
        let mut records_iter = records.into_iter();
        assert_eq!(
            processor.process(records_iter.next().unwrap()),
            Err(Error::InsufficientFunds)
        );
        assert_eq!(processor.accounts.get(1).unwrap().available(), 0 as f64);
    }
}
