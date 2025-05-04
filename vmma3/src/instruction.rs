// instruction.rs
//
// TODO:
// - debug

use std::io::stdin;
use std::io::{self, Write};

pub const RAM_SIZE: usize = 4096;

pub fn read_u32(ram: &[u8], addr: usize) -> u32 {
    u32::from_le_bytes([ram[addr], ram[addr + 1], ram[addr + 2], ram[addr + 3]])
}

pub fn write_u32(ram: &mut [u8], addr: usize, value: u32) {
    ram[addr..addr + 4].copy_from_slice(&value.to_le_bytes());
    //println!("{} {} {} {}", ram[addr], ram[addr + 1], ram[addr + 2], ram[addr + 3]);
}

pub fn pop(sp: &mut usize, ram: &[u8]) -> u32 {
    let val = read_u32(ram, *sp);
    *sp += 4;
    val
}

pub fn push(sp: &mut usize, ram: &mut [u8], val: u32) {
    *sp -= 4;
    write_u32(ram, *sp, val);
}

pub fn execute_instruction(pc: &mut usize, sp: &mut usize, ram: &mut [u8]) -> bool {
   // use std::io::Write;

    if *pc + 4 > RAM_SIZE {
        return false;
    }

    let instr = read_u32(ram, *pc);
    let opcode = (instr >> 28) & 0xF;

    *pc += 4;

    match opcode {
        // Miscellaneous Instructions
        0x0 => {
            let subcode = (instr >> 24) & 0xF;
            match subcode {
                // exit
                0x0 => {
                    let code = (instr & 0xFF) as i8;
                    std::process::exit(code.into());
                }
                // swap(sp + from, sp + to)
                0x1 => {
                    let from_encoded = ((instr >> 12) & 0xFFF) as i16;
                    let to_encoded   =  (instr         & 0xFFF) as i16;
                    let from_offset  = ((from_encoded << 4) >> 2) as isize;
                    let to_offset    = ((to_encoded   << 4) >> 2) as isize;
                    let base = *sp as isize;
                    let a = (base + from_offset) as usize;
                    let b = (base + to_offset)   as usize;
                    if a + 4 <= RAM_SIZE && b + 4 <= RAM_SIZE {
                        let va = read_u32(ram, a);
                        let vb = read_u32(ram, b);
                        write_u32(ram, a, vb);
                        write_u32(ram, b, va);
                    }
                }
                // input: read a line, parse dec/hex/bin, push value
                0x4 => {
                    print!("> ");
                    io::stdout().flush().unwrap();
                    let mut line = String::new();
                    io::stdin().read_line(&mut line).unwrap();
                    let s = line.trim();
                    let signed: i32 = if let Some(hex) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
                        u32::from_str_radix(hex, 16).unwrap_or(0) as i32
                    } else if let Some(bin) = s.strip_prefix("0b").or_else(|| s.strip_prefix("0B")) {
                        u32::from_str_radix(&s[2..], 2).unwrap_or(0) as i32
                    } else {
                        s.parse::<i32>().unwrap_or(0)
                    };


                    let val = signed as u32;
                    push(sp, ram, val);
                }

                // stinput
                0x5 => {
                    let max_char = (instr & 0x00FF_FFFF) as usize;
                    let mut input = String::new();
                    stdin().read_line(&mut input).expect("IO Error");
                    let trimmed = input.trim();
                    let limited = if trimmed.len() > max_char {
                        &trimmed[..max_char]
                    } else {
                        trimmed
                    };


                    let bytes = limited.as_bytes();
                    let mut index = bytes.len();
                    let mut first = true;

                    while index > 0 {
                        println!("Index: {}", index);
                        let start = index.saturating_sub(3);
                        let chunk = &bytes[start..index];

                        let cont = if first { 0 } else { 1 };
                        first = false;

                        let mut val = (cont as u32) << 24;
                        if chunk.len() > 2 {
                            val |= (chunk[2] as u32) << 16;
                        } else {
                            val |= 0x01 << 16;
                        }
                        if chunk.len() > 1 { 
                            val |= (chunk[1] as u32) << 8;
                        } else {
                            val |= 0x01 << 8;
                        }
                        if chunk.len() > 0 {
                            val |= chunk[0] as u32;
                        } else {
                            val |= 0x01;
                        }

                        push(sp, ram, val);
                        index = start;
                    }
                }
                // debug
                0xF => {
                    let debug_val = instr & 0x00FF_FFFF;
                    eprintln!("DEBUG Value={}, PC={}, SP={}", debug_val, *pc, *sp);
                }
                // any other subcode (including exit)
                _ => return false,
            }
        }
        // Pop Instructions
        0x1 => { // pop <imm>: immediate is a *byte* count (0 means “pop one word” = 4 bytes)
            let raw_offset = instr & 0x0FFF_FFFF;       // 26-bit immediate
            let offset_bytes = if raw_offset == 0 {
                4    // default pop one word
            } else {
                raw_offset as usize  // treat imm literally as bytes
            };
            let old_sp = *sp;
            // advance SP, but never past the last valid word
            *sp = (*sp + offset_bytes).min(RAM_SIZE);
            //println!("SP: {} -> {}", old_sp, *sp);
        }
        // Binary Arithmetic Instructions
        0x2 => {
            let subcode = (instr >> 24) & 0xF;
            let rhs_u = pop(sp, ram);
            let lhs_u = pop(sp, ram);

            let lhs = lhs_u as i32;
            let rhs = rhs_u as i32;


            let result = match subcode {
                0x0 => lhs.wrapping_add(rhs) as u32,
                0x1 => lhs.wrapping_sub(rhs) as u32,
                0x2 => lhs.wrapping_mul(rhs) as u32,
                0x3 => if rhs != 0 { (lhs / rhs) as u32 } else { 0 },
                0x4 => if rhs != 0 { (lhs % rhs) as u32 } else { 0 },
                0x5 => lhs_u & rhs_u,
                0x6 => lhs_u | rhs_u,
                0x7 => lhs_u ^ rhs_u,
                0x8 => lhs_u << rhs_u,
                0x9 => lhs_u >> rhs_u,
                0xB => ((lhs) >> (rhs_u)) as u32,
                _ => return true,
            };
            push(sp, ram, result);
        }
        // Unary Arithmetic Instructions
        0x3 => {
            let subcode = (instr >> 24) & 0xF;
            let val = pop(sp, ram);
            let result = match subcode {
                0x0 => -(val as i32) as u32,
                0x1 => !val,
                _ => return true,
            };
            push(sp, ram, result);
        }
        // String Print Instructions
        0x4 => {
            let raw_offset = ((instr >> 2) & 0x03FF_FFFF) as i32;
            let offset = (raw_offset << 6) >> 6;
            let mut addr = (*sp as i32 + (offset << 2)) as usize;

            while addr + 4 <= RAM_SIZE {
                let word = read_u32(ram, addr);
                let bytes = word.to_le_bytes();

                for &b in &bytes {
                    if b == 0x00 {
                        break;
                    } else if b != 0x01 {
                        print!("{}", b as char);
                        io::stdout().flush().unwrap();
                    }
                }

                if bytes.contains(&0x00) {
                    break;
                }

                addr += 4;
            }
        }
        // Call Instructions
        0x5 => {
            // Get 26-bit signed immediate
            let raw = ((instr >> 2) & 0x03FF_FFFF) as i32;
            let offset = (raw << 6) >> 6;
            //println!("Call Offset: {}", offset);
            // Push current PC to stack
            push(sp, ram, *pc as u32);
            // Jump to target PC
            let base_pc = (*pc as i32).wrapping_sub(4);
            let new_pc = base_pc.wrapping_add(offset << 2);

            if new_pc >= 0 && (new_pc as usize) < RAM_SIZE {
                *pc = new_pc as usize;
            } else {
                eprintln!("Invalid call offset: jumping to {:#X}", new_pc);
                return false;
            }
        }
        // Return Instructions
        0x6 => {
            let raw = instr & 0x0FFF_FFFF;
            let offset = raw as usize;

            *sp = (*sp).saturating_add(offset).min(RAM_SIZE.saturating_sub(4));

            let ret_addr = pop(sp, ram);

            *pc = ret_addr as usize;
        }
        // Unconditional Goto Instructions
        0x7 => {
            // 1) extract the 26-bit field from bits 27:2
            let raw = (instr >> 2) & 0x03FF_FFFF;           // mask = (1<<26)-1

            // 2) sign-extend from 26 → 32 bits
            let mut imm = raw as i32;
            if (imm & (1 << 25)) != 0 {                     // if top bit of 26 is set
                imm -= 1 << 26;
            }

            // 3) scale to bytes
            let offset = imm << 2;                          // multiply by 4

            // 4) PC was already bumped by 4, so undo that
            let base_pc = (*pc as i32).wrapping_sub(4);
            let new_pc  = base_pc.wrapping_add(offset);

            if new_pc >= 0 && (new_pc as usize) < RAM_SIZE {
                *pc = new_pc as usize;
            } else {
                eprintln!("Invalid goto offset: jumping to {:#X}", new_pc);
                return false;
            }
        }
        // Binary If Instructions
        0x8 => {
            // 1) Decode the 3-bit condition (bits 27:25)
            let cond = (instr >> 25) & 0x7;
            // 2) Decode the 23-bit signed offset (bits 24:2)
            let raw   = (instr >> 2) & 0x007F_FFFF;      // mask = (1<<23)-1
            let mut off = raw as i32;
            if (off & (1 << 22)) != 0 {                  // sign bit at 22
                off -= 1 << 23;                          // sign-extend
            }
            let offset = off.wrapping_mul(4);            // scale to bytes
            // 3) Peek stack: right = [SP], left = [SP+4]
            let right = read_u32(ram, *sp);
            let left  = if *sp + 4 < RAM_SIZE {
                read_u32(ram, *sp + 4)
            } else {
                0
            };
            // 4) Evaluate the condition
            let take = match cond {
                0 => left  == right,       // eq
                1 => left  != right,       // ne
                2 => (left as i32) <  (right as i32), // lt
                3 => (left as i32) >  (right as i32), // gt
                4 => (left as i32) <= (right as i32), // le
                5 => (left as i32) >= (right as i32), // ge
                _ => false,                       // 110/111 invalid
            };
            // 5) If true, jump PC by offset (undoing the earlier pc+=4)
            if take {
                let base_pc = (*pc as i32) - 4;
                let new_pc  = base_pc.wrapping_add(offset);
                if new_pc >= 0 && (new_pc as usize) < RAM_SIZE {
                    *pc = new_pc as usize;
                } else {
                    eprintln!("Invalid if<cond> jump to {:#X}", new_pc);
                    return false;
                }
            }
        }
        // Unary If Instructions
        0x9 => {
            let cond_code = (instr >> 25) & 0x3; // bits 25–24
            let raw = (instr & 0x00FF_FFFF) as i32;
            let offset = (raw << 8) >> 8;


            let val = read_u32(ram, *sp);

            let should_jump = match cond_code {
                0x0 => val == 0,                    // EZ
                0x1 => val != 0,                    // NZ
                0x2 => (val as i32) < 0,            // MI
                0x3 => (val as i32) >= 0,           // PL
                _ => false,
            };

            if should_jump {
                let current_pc = *pc as i32 - 4;
                let new_pc = current_pc.wrapping_add(offset);
                if new_pc >= 0 && (new_pc as usize) < RAM_SIZE {
                    *pc = new_pc as usize;
                } else {
                    eprintln!("Invalid jump to {:#X}", new_pc);
                    return false;
                }
            }
        }
        // Dup Instructions
        0xB => {
            let raw = (instr >> 2) & 0x03FF_FFFF;
            let mut imm = raw as i32;

            if (imm & (1 << 25)) != 0 {
                imm -= 1 << 26;
            }

            let offset = (imm << 2) as isize;
            let addr = (*sp as isize + offset) as usize;

            let val = read_u32(ram, addr);

            push(sp, ram, val);
        }
        // Print Instructions
        0xD => {
            let fmt = instr & 0b11;

            // 1) grab only bits [27:2] (26-bit signed immediate)
            let raw   = (instr >> 2) & 0x03FF_FFFF;    // mask = (1<<26)-1
            let mut imm = raw as i32;

            // 2) if bit-25 is set, subtract 1<<26 to sign-extend
            if (imm & (1 << 25)) != 0 {
                imm -= 1 << 26;
            }

            // 3) multiply by 4
            let offset = (imm << 2) as isize;

            // 4) compute address relative to SP
            let addr = (*sp as isize + offset) as usize;

            // now safe to read four bytes at addr
            let val = read_u32(ram, addr) as i32;

            match fmt {
                0b00 => println!("{}",   val),
                0b01 => println!("0x{:X}", val),
                0b10 => println!("0b{:b}", val),
                0b11 => println!("0o{:o}", val),
                _    => (),
            }
        }
        // Dump Instructions
        0xE => {
            if *sp == RAM_SIZE { return true; }
            for addr in (*sp..RAM_SIZE).step_by(4) {
                let rel = addr - *sp;
                let val = read_u32(ram, addr);
                println!("{:04x}: {:08x}", rel, val);
            }
        }
        // Push Instructions
        0xF => {
            let value = instr & 0x0FFF_FFFF;
            //let signed = if (value & (1 << 27)) != 0 {
            //    (value | 0xF000_0000) as i32 as u32
            //} else {
            //    value
            //};
            //println!("{:x}", value);
            push(sp, ram, value);
        }
        _ => (),
    }

    true
}
