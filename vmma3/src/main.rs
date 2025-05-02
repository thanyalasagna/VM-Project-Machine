mod instruction;

use std::env::args;
use std::fs::File;
use std::io::Read;
use instruction::{RAM_SIZE, execute_instruction};

fn main() {
    let args: Vec<String> = args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <input_file>", args[0]);
        return;
    }

    let input_file = &args[1];
    let mut file = File::open(input_file).expect("Failed to read the input file");

    let mut magic_buf = [0u8; 4];
    file.read_exact(&mut magic_buf).expect("Failed to read magic bytes");

    if magic_buf != [0xDE, 0xAD, 0xBE, 0xEF] {
        eprintln!("Invalid magic number.");
        return;
    }

    let mut ram = [0u8; RAM_SIZE];
    let mut instructions = Vec::new();
    file.read_to_end(&mut instructions).expect("Failed to read instructions");

    if instructions.len() > RAM_SIZE {
        eprintln!("Program is too large to fit in RAM.");
        return;
    }

    for (i, byte) in instructions.iter().enumerate() {
        ram[i] = *byte;
    }

    let mut pc: usize = 0;
    let mut sp: usize = RAM_SIZE;

    println!("Loaded {} bytes into RAM.", instructions.len());
    println!("PC starts at {}, SP starts at {}.", pc, sp);

    while instruction::execute_instruction(&mut pc, &mut sp, &mut ram) {}
}
