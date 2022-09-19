use crate::account::Account;
use csv::{Reader as CsvReader, Writer as CsvWriter};
use std::fs::File;
use std::io;
use std::io::Stdout;

use crate::error::{Error, Result};
use crate::processor::Record;

pub struct Reader<T = File> {
    inner: CsvReader<T>,
}

impl Reader {
    pub fn from_path(file: &str) -> Result<Self> {
        Ok(Self {
            inner: CsvReader::from_path(file).map_err(|_| Error::InvalidData)?,
        })
    }

    pub(crate) fn read(&mut self) -> Result<Vec<Record>> {
        let mut records = Vec::new();
        for result in self.inner.deserialize() {
            let record: Record = result.map_err(|_| Error::InvalidData)?;
            records.push(record);
        }
        Ok(records)
    }
}

pub struct Writer {
    inner: CsvWriter<Stdout>,
}

impl Writer {
    pub(crate) fn new() -> Self {
        Self {
            inner: CsvWriter::from_writer(io::stdout()),
        }
    }

    pub(crate) fn write(&mut self, data: Vec<&Account>) -> Result<()> {
        for d in data {
            self.inner.serialize(d).map_err(|_| Error::InvalidData)?;
        }
        Ok(())
    }
}
