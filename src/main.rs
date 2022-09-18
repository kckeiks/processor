mod processor;

fn main() {
    let proc = processor::Processor::new();
    proc.start();
}
