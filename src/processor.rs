use std::io;
use std::io::Stdin;
use csv::Reader;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Record {
    #[serde(rename = "type")]
    ty: String,
    client: u32,
    tx: u16,
    amount: u32,
}

pub struct Processor {
    reader: CsvReader
}

impl Processor {
    pub fn new() -> Self {
        Self {
            reader: CsvReader::new()
        }
    }

    pub fn start(mut self) {
        for record in self.reader.read() {
            println!("{:?}", record);
        }
    }
}

struct CsvReader {
    inner: Reader<Stdin>
}

impl CsvReader {
    fn new() -> Self {
        Self {
            inner: Reader::from_reader(io::stdin())
        }
    }

    fn read(&mut self) -> Vec<Record> {
        self.inner
            .deserialize()
            .filter_map(|res: csv::Result<Record>| {
                match res {
                    Ok(r) => Some(r),
                    Err(e) => {
                        println!("{}", e);
                        None
                    }
                }
            })
            .collect::<Vec<_>>()
    }
}