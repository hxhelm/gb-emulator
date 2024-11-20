use std::env;
use std::fs;

use cpu::CPU;

mod cpu;

fn main() {
    let args: Vec<String> = env::args().collect();
    let bin_path = args.get(1).expect("First argument <BINARY_PATH> required.");
    let contents = fs::read(bin_path).expect("Failed to read binary path.");
    let mut cpu = CPU::default();

    cpu.boot(contents.as_slice());
}
