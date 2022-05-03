use std::io;

use crate::util::*;
use crate::hw::*;
use crate::opc::*;
use crate::mem::*;

fn byte_to_p(byte: u8, nes: &mut Nes) {
    nes.cpu.p_n = get_bit(byte, 7);
    nes.cpu.p_v = get_bit(byte, 6);
    nes.cpu.p_d = get_bit(byte, 3);
    nes.cpu.p_i = get_bit(byte, 2);
    nes.cpu.p_z = get_bit(byte, 1);
    nes.cpu.p_c = get_bit(byte, 0);
}
fn p_to_byte(nes: &Nes) -> u8 {
    (if nes.cpu.p_n {0b1000_0000} else {0}) | 
    (if nes.cpu.p_v {0b0100_0000} else {0}) | 
                 0b0010_0000            |
    (if nes.cpu.p_d {0b0000_1000} else {0}) | 
    (if nes.cpu.p_i {0b0000_0100} else {0}) | 
    (if nes.cpu.p_z {0b0000_0010} else {0}) | 
    (if nes.cpu.p_c {0b0000_0001} else {0})
}
fn stack_push(val: u8, nes: &mut Nes) {
    nes.wram[0x0100 + (nes.cpu.s as usize)] = val;
    nes.cpu.s = nes.cpu.s.wrapping_sub(1);
}
fn stack_pop(nes: &mut Nes) -> u8 {
    nes.cpu.s = nes.cpu.s.wrapping_add(1);
    nes.wram[0x0100 + (nes.cpu.s as usize)]
}
fn stack_push_u16(val: u16, nes: &mut Nes) {
    stack_push((val >> 8)     as u8, nes);
    stack_push((val & 0x00FF) as u8, nes);
}
fn stack_pop_u16(nes: &mut Nes) -> u16 {
    let lsb = stack_pop(nes);
    let msb = stack_pop(nes);
    concat_u8(msb, lsb)
}
fn update_p_nz(val: u8, nes: &mut Nes) {
    nes.cpu.p_n = val > 0x7F;
    nes.cpu.p_z = val == 0;
}
fn shift_left(val: u8, rotate: bool, nes: &mut Nes) -> u8 {
    let prev_carry = nes.cpu.p_c;
    nes.cpu.p_c = get_bit(val, 7);
    (val << 1) | ((prev_carry && rotate) as u8)
}
fn shift_right(val: u8, rotate: bool, nes: &mut Nes) -> u8 {
    let prev_carry = nes.cpu.p_c;
    nes.cpu.p_c = get_bit(val, 0);
    (val >> 1) | (((prev_carry && rotate) as u8) << 7)
}
fn add_with_carry(val: u8, nes: &mut Nes) {
    let (result, carry) = nes.cpu.a.carrying_add(val, nes.cpu.p_c);
    nes.cpu.p_v = was_signed_overflow(nes.cpu.a, val, result);
    nes.cpu.p_c = carry;
    nes.cpu.a = result;  
}

fn get_instr_addr(addressing_mode: Mode, byte2: u8, byte3: u8, nes: &mut Nes) -> u16 {
    match addressing_mode {
        Mode::Abs  => concat_u8(byte3, byte2),
        Mode::AbsX => concat_u8(byte3, byte2).wrapping_add(nes.cpu.x as u16),
        Mode::AbsY => concat_u8(byte3, byte2).wrapping_add(nes.cpu.y as u16),
        Mode::Zpg  => byte2 as u16,
        Mode::ZpgX => byte2.wrapping_add(nes.cpu.x) as u16,
        Mode::ZpgY => byte2.wrapping_add(nes.cpu.y) as u16,
        Mode::IndX => read_mem_u16_zp(byte2.wrapping_add(nes.cpu.x) as u16, nes),
        Mode::IndY => read_mem_u16_zp(byte2 as u16, nes).wrapping_add(nes.cpu.y as u16),
        Mode::AbsI => {
            // MSB not incremented when indirect address sits on page boundary
            let lsb = read_mem(concat_u8(byte3, byte2), nes);
            let msb = read_mem(concat_u8(byte3, byte2.wrapping_add(1)), nes);
            concat_u8(msb, lsb)
        },
        Mode::Rel  => {
            // u8 -> i8 uses bit 7 as sign, i8 -> i16 sign extends
            let signed_offset = (byte2 as i8) as i16;
            nes.cpu.pc.wrapping_add_signed(signed_offset).wrapping_add(2)
        },
        _ => 0,
    }
}



fn log(opcode: u8, byte2: u8, byte3: u8, instr_addr: u16, instr_val: u8, instr_info: Info, nes: &mut Nes) {
    let log = String::new();

    // pc, opcode
    let mut log_line = format!("{pc:04X}  {opc:02X} ", pc=nes.cpu.pc, opc=opcode);

    // byte2, byte3
    match instr_len(instr_info.mode) {
        1 => log_line.push_str("      "),
        2 => log_line.push_str(&format!("{byte2:02X}    ")),
        3 => log_line.push_str(&format!("{byte2:02X} {byte3:02X} ")),
        _ => {},
    }

    // opc name
    log_line.push_str(&format!("{name:>4} ", name=instr_info.name));

    // mode formatting
    match instr_info.mode {
        Mode::Imp => log_line.push_str("                            "),
        Mode::Acc => log_line.push_str("A                           "),
        Mode::Imm => log_line.push_str(&format!("#${byte2:02X}                        ")),
        Mode::Abs => {
            if opcode != 0x4C && opcode != 0x20 {
                log_line.push_str(&format!("${instr_addr:04X} = {instr_val:02X}                  "));
            } else {
                log_line.push_str(&format!("${instr_addr:04X}                       "));
            }
        },

        Mode::Rel => log_line.push_str(&format!("${instr_addr:04X}                       ")),
        Mode::AbsX => log_line.push_str(&format!("${byte3:02X}{byte2:02X},X @ {instr_addr:04X} = {instr_val:02X}         ")),
        Mode::AbsY => log_line.push_str(&format!("${byte3:02X}{byte2:02X},Y @ {instr_addr:04X} = {instr_val:02X}         ")),
        Mode::Zpg => log_line.push_str(&format!("${byte2:02X} = {instr_val:02X}                    ")),
        Mode::ZpgX => log_line.push_str(&format!("${byte2:02X},X @ {offset:02X} = {instr_val:02X}             ", offset=byte2.wrapping_add(nes.cpu.x))),
        Mode::ZpgY => log_line.push_str(&format!("${byte2:02X},Y @ {offset:02X} = {instr_val:02X}             ", offset=byte2.wrapping_add(nes.cpu.y))),
        Mode::IndX => log_line.push_str(&format!("(${byte2:02X},X) @ {ind_addr:02X} = {instr_addr:04X} = {instr_val:02X}    ", ind_addr=byte2.wrapping_add(nes.cpu.x))),
        Mode::IndY => log_line.push_str(&format!("(${byte2:02X}),Y = {ind_addr:04X} @ {instr_addr:04X} = {instr_val:02X}  ",ind_addr=read_mem_u16_zp(byte2 as u16, nes))),
        Mode::AbsI => log_line.push_str(&format!("(${byte3:02X}{byte2:02X}) = {instr_addr:04X}              ")),
    }

    log_line.push_str(&format!("A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X}", nes.cpu.a, nes.cpu.x, nes.cpu.y, p_to_byte(nes), nes.cpu.s));
    println!("{}", log_line);
    println!("ppu: ({},{})", nes.ppu.scanline, nes.ppu.pixel);

    if nes.cpu.cycles % nes.skip == 0 {
        let mut buffer = String::new();
        let mut stdin = io::stdin(); // We get `Stdin` here.
        stdin.read_line(&mut buffer).expect("it broke");
        nes.skip = match buffer.trim().parse() {
            Ok(x) => x,
            Err(e) => {println!("{}", e); 1}
        };
        println!("{:?}", nes.ppu.palette_mem);
    }

}

pub fn step_cpu(nes: &mut Nes) {

    if nes.cpu.nmi_interrupt {
        stack_push_u16(nes.cpu.pc, nes);
        stack_push(p_to_byte(nes), nes);
        nes.cpu.pc = read_mem_u16(0xFFFA, nes);
        nes.cpu.nmi_interrupt = false;
    }

    let opcode = read_mem(nes.cpu.pc, nes);
    let instr_info = INSTRUCTION_INFO[opcode as usize];

    // Gets the relevant address referenced by the instruction, if any
    let byte2 = read_mem(nes.cpu.pc.wrapping_add(1), nes);
    let byte3 = read_mem(nes.cpu.pc.wrapping_add(2), nes);
    let instr_addr = get_instr_addr(instr_info.mode, byte2, byte3, nes);

    // Gets the immediate value, or value located at instr_addr
    let instr_val = match instr_info.mode {
        Mode::Imm => byte2,
        _ => read_mem(instr_addr, nes),
    };

    nes.cpu.counter += 1;

    let prev_pc = nes.cpu.pc;

    log(opcode, byte2, byte3, instr_addr, instr_val, instr_info, nes);

    exec_instruction(opcode, instr_addr, instr_val, nes);
    
    nes.cpu.cycles += instr_info.cycles as u64;

    // If branch was taken (if pc has changed)...
    if nes.cpu.pc == prev_pc {
        nes.cpu.pc = nes.cpu.pc.wrapping_add(instr_len(instr_info.mode));
    }
}

fn exec_instruction(opcode: u8, instr_addr: u16, instr_val: u8, nes: &mut Nes) {
    match opcode {
        // LDA
        0xAD | 0xBD | 0xA9 | 0xB9 | 0xA1 | 0xB1 | 0xA5 | 0xB5 => {
            nes.cpu.a = instr_val;
            update_p_nz(nes.cpu.a, nes);
        },
        // LDX
        0xAE | 0xBE | 0xA2 | 0xA6 | 0xB6 => {
            nes.cpu.x = instr_val;
            update_p_nz(nes.cpu.x, nes);
        },
        // LDY
        0xAC | 0xBC | 0xA0 | 0xA4 | 0xB4 => {
            nes.cpu.y = instr_val;
            update_p_nz(nes.cpu.y, nes);
        },
        // STA
        0x8D | 0x9D | 0x99 | 0x81 | 0x91 | 0x85 | 0x95 => {
            write_mem(instr_addr, nes.cpu.a, nes);
        },        
        // STX
        0x8E | 0x86 | 0x96 => {
            write_mem(instr_addr, nes.cpu.x, nes);
        },        
        // STY
        0x8C | 0x84 | 0x94 => {
            write_mem(instr_addr, nes.cpu.y, nes);
        },
        // TAX
        0xAA => {
            nes.cpu.x = nes.cpu.a;
            update_p_nz(nes.cpu.x, nes);
        },
        // TAY
        0xA8 => {
            nes.cpu.y = nes.cpu.a;
            update_p_nz(nes.cpu.y, nes);
        },
        // TSX
        0xBA => {
            nes.cpu.x = nes.cpu.s;
            update_p_nz(nes.cpu.x, nes);
        },
        // TXA
        0x8A => {
            nes.cpu.a = nes.cpu.x;
            update_p_nz(nes.cpu.a, nes);
        },
        // TXS
        0x9A => {
            nes.cpu.s = nes.cpu.x;
        },
        // TYA
        0x98 => {
            nes.cpu.a = nes.cpu.y;
            update_p_nz(nes.cpu.a, nes);
        },
        // PHA
        0x48 => {
            stack_push(nes.cpu.a, nes);
        },
        // PHP
        0x08 => {
            stack_push(p_to_byte(nes) | 0b0001_0000, nes);
        },
        // PLA
        0x68 => {
            nes.cpu.a = stack_pop(nes);
            update_p_nz(nes.cpu.a, nes);
        },
        // PLP
        0x28 => {
            let p_byte = stack_pop(nes);
            byte_to_p(p_byte, nes);
        },
        // ASL (ACC)
        0x0A => {
            nes.cpu.a = shift_left(nes.cpu.a, false, nes);
            update_p_nz(nes.cpu.a, nes);
        },
        // ASL (RMW)
        0x0E | 0x1E | 0x06 | 0x16 => {
            let new_val = shift_left(instr_val, false, nes);
            write_mem(instr_addr, new_val, nes);
            update_p_nz(new_val, nes);
        },
        // LSR (ACC)
        0x4A => {
            nes.cpu.a = shift_right(nes.cpu.a, false, nes);
            update_p_nz(nes.cpu.a, nes);
        },
        // LSR (RMW)
        0x4E | 0x5E | 0x46 | 0x56 => {
            let new_val = shift_right(instr_val, false, nes);
            write_mem(instr_addr, new_val, nes);
            update_p_nz(new_val, nes);
        },
        // ROL (ACC)
        0x2A => {
            nes.cpu.a = shift_left(nes.cpu.a, true, nes);
            update_p_nz(nes.cpu.a, nes);
        },
        // ROL (RMW)
        0x2E | 0x3E | 0x26 | 0x36 => {
            let new_val = shift_left(instr_val, true, nes);
            write_mem(instr_addr, new_val, nes);
            update_p_nz(new_val, nes);
        },
        // ROR (ACC)
        0x6A => {
            nes.cpu.a = shift_right(nes.cpu.a, true, nes);
            update_p_nz(nes.cpu.a, nes);
        },
        // ROR (RMW)
        0x6E | 0x7E | 0x66 | 0x76 => {
            let new_val = shift_right(instr_val, true, nes);
            write_mem(instr_addr, new_val, nes);
            update_p_nz(new_val, nes);
        },
        // AND
        0x2D | 0x3D | 0x39 | 0x29 | 0x21 | 0x31 | 0x25 | 0x35 =>  {
            nes.cpu.a &= instr_val;
            update_p_nz(nes.cpu.a, nes);
        },
        // BIT
        0x2C | 0x24 => {
            nes.cpu.p_n = get_bit(instr_val, 7);
            nes.cpu.p_v = get_bit(instr_val, 6);
            nes.cpu.p_z = (nes.cpu.a & instr_val) == 0;
        },
        // EOR
        0x4D | 0x5D | 0x59 | 0x49 | 0x41 | 0x51 | 0x45 | 0x55 => { 
            nes.cpu.a ^= instr_val;
            update_p_nz(nes.cpu.a, nes);
        },
        // ORA
        0x0D | 0x1D | 0x19 | 0x09 | 0x01 | 0x11 | 0x05 | 0x15 => {
            nes.cpu.a |= instr_val;
            update_p_nz(nes.cpu.a, nes);
        },
        // ADC
        0x6D | 0x7D | 0x79 | 0x69 | 0x61 | 0x71 | 0x65 | 0x75 => {
            add_with_carry(instr_val, nes);
        },
        // CMP
        0xCD | 0xDD | 0xD9 | 0xC9 | 0xC1 | 0xD1 | 0xC5 | 0xD5 => {
            nes.cpu.p_z = nes.cpu.a == instr_val;
            nes.cpu.p_n = is_neg(nes.cpu.a.wrapping_sub(instr_val));
            nes.cpu.p_c = instr_val <= nes.cpu.a;
        },
        // CPX
        0xEC | 0xE0 | 0xE4 => {
            nes.cpu.p_z = nes.cpu.x == instr_val;
            nes.cpu.p_n = is_neg(nes.cpu.x.wrapping_sub(instr_val));
            nes.cpu.p_c = instr_val <= nes.cpu.x;
        },
        // CPY
        0xCC | 0xC0 | 0xC4 => {
            nes.cpu.p_z = nes.cpu.y == instr_val;
            nes.cpu.p_n = is_neg(nes.cpu.y.wrapping_sub(instr_val));
            nes.cpu.p_c = instr_val <= nes.cpu.y;
        },
        // SBC
        0xED | 0xFD | 0xF9 | 0xE9 | 0xE1 | 0xF1 | 0xE5 | 0xF5 | 0xEB => {
            add_with_carry(instr_val ^ 0xFF, nes);
        },
        // DEC
        0xCE | 0xDE | 0xC6 | 0xD6 => {
            let new_val = instr_val.wrapping_sub(1);
            write_mem(instr_addr, new_val, nes);
            update_p_nz(new_val, nes);
        },
        // DEX
        0xCA => {
            nes.cpu.x = nes.cpu.x.wrapping_sub(1);
            update_p_nz(nes.cpu.x, nes);
        },
        // DEY
        0x88 => {
            nes.cpu.y = nes.cpu.y.wrapping_sub(1);
            update_p_nz(nes.cpu.y, nes);
        },
        // INC
        0xEE | 0xFE | 0xE6 | 0xF6 => {
            let new_val = instr_val.wrapping_add(1);
            write_mem(instr_addr, new_val, nes);
            update_p_nz(new_val, nes);
        },
        // INX
        0xE8 => {
            nes.cpu.x = nes.cpu.x.wrapping_add(1);
            update_p_nz(nes.cpu.x, nes);
        },
        // INY
        0xC8 => {
            nes.cpu.y = nes.cpu.y.wrapping_add(1);
            update_p_nz(nes.cpu.y, nes);
        },
        // BRK
        0x00 => {
            stack_push_u16(nes.cpu.pc, nes);
            stack_push(p_to_byte(nes) | 0b0001_0000, nes);
            nes.cpu.pc = read_mem_u16(0xFFFE, nes);
        },
        // JMP 
        0x4C | 0x6C => {
            nes.cpu.pc = instr_addr;
        }
        // JSR
        0x20 => {
            stack_push_u16(nes.cpu.pc.wrapping_add(2), nes);
            nes.cpu.pc = instr_addr;
        },
        // RTI
        0x40 => {
            let p_reg = stack_pop(nes);
            byte_to_p(p_reg, nes);
            nes.cpu.pc = stack_pop_u16(nes);
        },
        // RTS 
        0x60 => {
            nes.cpu.pc = stack_pop_u16(nes).wrapping_add(1);
        },
        // BCC
        0x90 => if !nes.cpu.p_c {nes.cpu.pc = instr_addr},
        // BCS
        0xB0 => if nes.cpu.p_c  {nes.cpu.pc = instr_addr},
        // BVC
        0x50 => if !nes.cpu.p_v {nes.cpu.pc = instr_addr},
        // BVS
        0x70 => if nes.cpu.p_v  {nes.cpu.pc = instr_addr},
        // BEQ
        0xF0 => if nes.cpu.p_z  {nes.cpu.pc = instr_addr},
        // BNE
        0xD0 => if !nes.cpu.p_z {nes.cpu.pc = instr_addr},
        // BMI
        0x30 => if nes.cpu.p_n  {nes.cpu.pc = instr_addr},
        // BPL
        0x10 => if !nes.cpu.p_n {nes.cpu.pc = instr_addr},
        // CLC 
        0x18 => nes.cpu.p_c = false,
        // CLD
        0xD8 => nes.cpu.p_d = false,
        // CLI 
        0x58 => nes.cpu.p_i = false,
        // CLV 
        0xB8 => nes.cpu.p_v = false,
        // SEC
        0x38 => nes.cpu.p_c = true,
        // SED
        0xF8 => nes.cpu.p_d = true,
        // SEI
        0x78 => nes.cpu.p_i = true,
        // NOP
        0xEA => {}
        
        // UNOFFICIAL OPCODES

        // LAS
        0xBB => {
            let new_val = nes.cpu.s & instr_val;
            nes.cpu.a = new_val;
            nes.cpu.x = new_val;
            nes.cpu.s = new_val;
            update_p_nz(nes.cpu.a, nes);
        }
        // LAX
        0xAB | 0xAF | 0xBF | 0xA7 | 0xB7 | 0xA3 | 0xB3 => {
            nes.cpu.a = instr_val;
            nes.cpu.x = instr_val;
            update_p_nz(nes.cpu.a, nes);
        }
        // SAX
        0x83 | 0x87 | 0x8F | 0x97 => {
            write_mem(instr_addr, nes.cpu.a & nes.cpu.x, nes); 
        }
        // SHA (AbsY)
        0x9F => {}
        // SHA (IndY)
        0x93 => {}
        // SHX
        0x9E => {}
        // SHY
        0x9C => {}
        // SHS
        0x9B => {}
        // ANC
        0x0B | 0x2B => {}
        // ARR
        0x6B => {}
        // ASR
        0x4B => {
            nes.cpu.p_c = (instr_val & 0x01) == 1;
            (nes.cpu.a & instr_val) >> 1;
        }
        // DCP
        0xC3 | 0xC7 | 0xCF | 0xD3 | 0xD7 | 0xDB | 0xDF => {
            nes.wram[instr_addr as usize] = nes.wram[instr_addr as usize].wrapping_sub(1);
            nes.cpu.p_z = nes.cpu.a == nes.wram[instr_addr as usize];
            nes.cpu.p_n = is_neg(nes.cpu.a.wrapping_sub(nes.wram[instr_addr as usize]));
            nes.cpu.p_c = nes.wram[instr_addr as usize] <= nes.cpu.a;
        }
        // ISC
        0xE3 | 0xE7 | 0xEF | 0xF3 | 0xF7 | 0xFB | 0xFF => {
            nes.wram[instr_addr as usize] = nes.wram[instr_addr as usize].wrapping_add(1);            
            let (result, borrow) = nes.cpu.a.borrowing_sub(nes.wram[instr_addr as usize], !nes.cpu.p_c);

            nes.cpu.p_v = was_signed_overflow(nes.cpu.a, (-(nes.wram[instr_addr as usize] as i8) as u8), result);
            nes.cpu.a = result;
            nes.cpu.p_c = !borrow;
        }
        // RLA
        0x23 | 0x27 | 0x2F | 0x33 | 0x37 | 0x3B | 0x3F => {
            let result = shift_left(instr_val, true, nes);
            write_mem(instr_addr, result, nes);
            nes.cpu.a &= result;
            update_p_nz(nes.cpu.a, nes);
        }
        // RRA 
        0x63 | 0x67 | 0x6F | 0x73 | 0x77 | 0x7B | 0x7F => {
            let result = shift_right(instr_val, true, nes);
            write_mem(instr_addr, result, nes);
            add_with_carry(result, nes);
        }
        // SBX
        0xCB => {}
        // SLO
        0x03 | 0x07 | 0x0F | 0x13 | 0x17 | 0x1B | 0x1F => {
            nes.cpu.p_c = nes.wram[instr_addr as usize] > 127;
            nes.wram[instr_addr as usize] <<= 1;
            nes.cpu.a |= nes.wram[instr_addr as usize];
        }
        // SRE
        0x43 | 0x47 | 0x4F | 0x53 | 0x57 | 0x5B | 0x5F => {
            nes.cpu.p_c = (nes.wram[instr_addr as usize] & 0x01) == 1;
            nes.wram[instr_addr as usize] >>= 1;

            nes.cpu.a ^= nes.wram[instr_addr as usize];
        }
        // XAA
        0x8B => {}
        // JAM
        0x02 | 0x12 | 0x22 | 0x32 | 0x42 | 0x52 | 0x62 | 0x72 | 0x92 | 0xB2 | 0xD2 | 0xF2 => {}
        // NOP
        0x1A | 0x3A | 0x5A | 0x7A | 0xDA | 0xFA | 0x80 | 0x82 | 0x89 | 0xC2 | 0xE2 | 0x0C |
        0x1C | 0x3C | 0x5C | 0x7C | 0xDC | 0xFC | 0x04 | 0x44 | 0x64 | 0x14 | 0x34 | 0x54 | 0x74 |
        0xD4 | 0xF4 => {}
    }
}