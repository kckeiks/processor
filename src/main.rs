mod account;
mod error;
mod io;
mod processor;

use crate::io::Reader;
use std::env;

fn main() {
    env_logger::init();

    let args: Vec<String> = env::args().collect();
    let file = args.get(1).expect("expected filename");

    let reader = Reader::from_path(file.as_str()).expect("failed to create reader");
    let proc = processor::Processor::new_with(reader);
    if let Err(e) = proc.start() {
        eprintln!("Error: {}", e);
    }
}
