use csv::{Reader, Writer};
use std::io;
use std::error::Error;
use std::io::{Stdin, Stdout};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize,Deserialize)]
pub(crate) struct Record {
    #[serde(rename = "type")]
    ty: String,
    client: u32,
    tx: u16,
    amount: u32,
}

pub struct CsvReader {
    inner: Reader<Stdin>
}

impl CsvReader {
    pub(crate) fn new() -> Self {
        Self {
            inner: Reader::from_reader(io::stdin())
        }
    }

    pub(crate) fn read(&mut self) -> Result<Vec<Record>, Box<dyn Error>> {
        let mut records = Vec::new();
        for result in self.inner.deserialize() {
            let record: Record = result.map_err(|e| Box::new(e))?;
            records.push(record);
        }
        Ok(records)
    }
}

pub struct CsvWriter {
    inner: Writer<Stdout>
}

impl CsvWriter {
    pub(crate) fn new() -> Self {
        Self {
            inner: Writer::from_writer(io::stdout())
        }
    }

    pub(crate) fn write(&mut self, data: Vec<Record>) -> Result<(), Box<dyn Error>> {
        for d in data {
            self.inner.serialize(d).map_err(|e| Box::new(e))?;
        }
        Ok(())
    }
}
