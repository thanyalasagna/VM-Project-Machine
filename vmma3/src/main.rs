use std::env::args;
use std::fs::File;
use std::io::{self, Read};

const RAM_SIZE: usize = 4096;

fn main() {
    let args: Vec<String> = args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <input_file>", args[0]);
        return;
    }

    let input_file = &args[1];
    let mut file = File::open(input_file).expect("Failed to read the input file");

    // Read and check magic number
    let mut magic_buf = [0u8; 4];
    file.read_exact(&mut magic_buf).expect("Failed to read magic bytes");

    if magic_buf != [0xDE, 0xAD, 0xBE, 0xEF] {
        eprintln!("Invalid magic number: expected 0xDEADBEEF, found {:02X?}", magic_buf);
        return;
    }

    // Allocate RAM and zero it
    let mut ram = [0u8; RAM_SIZE];

    // Read the remaining bytes into a buffer
    let mut instructions = Vec::new();
    file.read_to_end(&mut instructions).expect("Failed to read instructions");

    if instructions.len() > RAM_SIZE {
        eprintln!("Program is too large to fit in RAM.");
        return;
    }

    // Load instructions into top of RAM (starts at index 0)
    for (i, byte) in instructions.iter().enumerate() {
        ram[i] = *byte;
    }

    // Initialize PC and SP
    let mut pc: usize = 0;
    let mut sp: usize = RAM_SIZE;

    println!("Loaded {} bytes into RAM.", instructions.len());
    println!("PC starts at {}, SP starts at {}.", pc, sp);

    // You can now start your fetch-decode-execute loop here
}