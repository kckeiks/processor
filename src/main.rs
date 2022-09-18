mod processor;
mod io;
mod record;

fn main() {
    let proc = processor::Processor::new();
    proc.start();
}
