use std::error::Error;
use std::io;
use std::process;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Record {
    #[serde(rename = "type")]
    ty: String,
    client: u32,
    tx: u16,
    amount: u32,
}

fn main() {
    let mut rdr = csv::Reader::from_reader(io::stdin());
    for result in rdr.deserialize() {
        let record: Record = match result {
            Ok(r) => r,
            Err(err) => {
                println!("error: {}", err);
                process::exit(1);
            }
        };
        println!("{:?}", record);
    }
}
