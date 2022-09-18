mod processor;
mod io;

fn main() {
    let proc = processor::Processor::new();
    proc.start();
}
