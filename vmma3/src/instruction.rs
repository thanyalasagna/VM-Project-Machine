// instruction.rs
pub const RAM_SIZE: usize = 4096;

pub fn read_u32(ram: &[u8], addr: usize) -> u32 {
    u32::from_le_bytes([ram[addr], ram[addr + 1], ram[addr + 2], ram[addr + 3]])
}

pub fn write_u32(ram: &mut [u8], addr: usize, value: u32) {
    ram[addr..addr + 4].copy_from_slice(&value.to_le_bytes());
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
    use std::io::Write;

    if *pc + 4 > RAM_SIZE {
        return false;
    }

    let instr = read_u32(ram, *pc);
    let opcode = (instr >> 28) & 0xF;

    *pc += 4;

    match opcode {
        0x2 => {
            let subcode = (instr >> 24) & 0xF;
            let rhs = pop(sp, ram);
            let lhs = pop(sp, ram);
            let result = match subcode {
                0x0 => lhs.wrapping_add(rhs),
                0x1 => lhs.wrapping_sub(rhs),
                0x2 => lhs.wrapping_mul(rhs),
                0x3 => if rhs != 0 { lhs / rhs } else { 0 },
                0x4 => if rhs != 0 { lhs % rhs } else { 0 },
                0x5 => lhs & rhs,
                0x6 => lhs | rhs,
                0x7 => lhs ^ rhs,
                0x8 => lhs << rhs,
                0x9 => lhs >> rhs,
                0xB => ((lhs as i32) >> rhs) as u32,
                _ => return true,
            };
            push(sp, ram, result);
        }

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

        0x4 => {
            let raw_offset = ((instr >> 2) & 0x00FF_FFFF) as i32;
            let offset = (raw_offset << 2) as isize;
            let mut addr = (*sp as isize + offset) as usize;

            while addr + 4 <= RAM_SIZE {
                let word = read_u32(ram, addr);
                let bytes = word.to_le_bytes();

                for &b in &bytes {
                    if b == 0x00 {
                        break;
                    } else if b != 0x01 {
                        print!("{}", b as char);
                    }
                }

                if bytes.contains(&0x00) {
                    break;
                }

                addr += 4;
            }

            println!();
        }

        0xD => {
            let fmt = instr & 0b11;
            let offset = (((instr >> 2) as i32) << 2) as isize;
            let addr = (*sp as isize + offset) as usize;
            let val = read_u32(ram, addr);
            match fmt {
                0b00 => println!("{}", val),
                0b01 => println!("0x{:X}", val),
                0b10 => println!("0b{:b}", val),
                0b11 => println!("0o{:o}", val),
                _ => (),
            }
        }

        0xE => {
            if *sp == RAM_SIZE { return true; }
            for addr in (*sp..RAM_SIZE).step_by(4) {
                let rel = addr - *sp;
                let val = read_u32(ram, addr);
                println!("{:04x}: {:08x}", rel, val);
            }
        }

        0xF => {
            let value = instr & 0x0FFF_FFFF;
            let signed = if (value & (1 << 27)) != 0 {
                (value | 0xF000_0000) as i32 as u32
            } else {
                value
            };
            push(sp, ram, signed);
        }

        0x0 => return false,
        _ => (),
    }

    true
}
