use crate::io::{CsvReader, CsvWriter};

pub struct Processor {
    reader: CsvReader,
    writer: CsvWriter
}

impl Processor {
    pub fn new() -> Self {
        Self {
            reader: CsvReader::new(),
            writer: CsvWriter::new()
        }
    }

    pub fn start(mut self) {
        match self.reader.read() {
            Ok(records) => self.writer.write(records).expect("failed to write"),
            Err(e) => println!("error: {}", e),
        }
    }
}