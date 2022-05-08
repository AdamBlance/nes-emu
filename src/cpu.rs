use std::io;
use crate::util::*;
use crate::hw::*;
use crate::opc::*;
use crate::mem::*;

use crate::outfile::*;

// Need a summary on opcodes, addressing modes
fn get_instr_addr(addressing_mode: Mode, byte1: u8, byte2: u8, nes: &mut Nes) -> u16 {
    match addressing_mode {
        // Use two bytes to address all 64K of memory
        Mode::Absolute  => concat_u8(byte2, byte1),
        Mode::AbsoluteX => concat_u8(byte2, byte1).wrapping_add(nes.cpu.x as u16),
        Mode::AbsoluteY => concat_u8(byte2, byte1).wrapping_add(nes.cpu.y as u16),
        Mode::AbsoluteI => {
            // MSB not incremented when indirect address sits on page boundary
            let lsb = read_mem(concat_u8(byte2, byte1), nes);
            let msb = read_mem(concat_u8(byte2, byte1.wrapping_add(1)), nes);
            concat_u8(msb, lsb)
        },

        // Use one byte to address the first 256B of memory
        Mode::ZeroPage  => byte1 as u16,
        Mode::ZeroPageX => byte1.wrapping_add(nes.cpu.x) as u16,
        Mode::ZeroPageY => byte1.wrapping_add(nes.cpu.y) as u16,
        Mode::IndirectX => read_mem_u16_zp(byte1.wrapping_add(nes.cpu.x) as u16, nes),
        Mode::IndirectY => read_mem_u16_zp(byte1 as u16, nes).wrapping_add(nes.cpu.y as u16),

        // Use byte as a signed integer and add to program counter as an offset
        Mode::Relative  => {
            // u8 -> i8 uses bit 7 as sign, i8 -> i16 sign extends
            let signed_offset = (byte1 as i8) as i16;
            nes.cpu.pc.wrapping_add_signed(signed_offset).wrapping_add(2)
        },

        // Instructions with these addressing modes don't involve memory addresses
        // Could have used Option here but just didn't think it was necessary 
        Mode::Accumulator | Mode::Implied | Mode::Immediate => 0,
    }
}

fn log(opcode: u8, byte2: u8, byte3: u8, instr_addr: u16, instr_val: u8, nes: &mut Nes) -> String{
    let instruction = INSTRUCTIONS[opcode as usize];

    // pc, opcode
    let mut log_line = format!("{pc:04X}  {opc:02X} ", pc=nes.cpu.pc, opc=opcode);

    // byte2, byte3
    match instruction.mode.num_bytes() {
        1 => log_line.push_str("      "),
        2 => log_line.push_str(&format!("{byte2:02X}    ")),
        3 => log_line.push_str(&format!("{byte2:02X} {byte3:02X} ")),
        _ => {},
    }

    // opc name
    log_line.push_str(&format!("{name:>4} ", name=instruction.name));

    // mode formatting
    match instruction.mode {
        Mode::Implied => log_line.push_str("                            "),
        Mode::Accumulator => log_line.push_str("A                           "),
        Mode::Immediate => log_line.push_str(&format!("#${byte2:02X}                        ")),
        Mode::Absolute => {
            if opcode != 0x4C && opcode != 0x20 {
                log_line.push_str(&format!("${instr_addr:04X} = {instr_val:02X}                  "));
            } else {
                log_line.push_str(&format!("${instr_addr:04X}                       "));
            }
        },

        Mode::Relative => log_line.push_str(&format!("${instr_addr:04X}                       ")),
        Mode::AbsoluteX => log_line.push_str(&format!("${byte3:02X}{byte2:02X},X @ {instr_addr:04X} = {instr_val:02X}         ")),
        Mode::AbsoluteY => log_line.push_str(&format!("${byte3:02X}{byte2:02X},Y @ {instr_addr:04X} = {instr_val:02X}         ")),
        Mode::ZeroPage => log_line.push_str(&format!("${byte2:02X} = {instr_val:02X}                    ")),
        Mode::ZeroPageX => log_line.push_str(&format!("${byte2:02X},X @ {offset:02X} = {instr_val:02X}             ", offset=byte2.wrapping_add(nes.cpu.x))),
        Mode::ZeroPageY => log_line.push_str(&format!("${byte2:02X},Y @ {offset:02X} = {instr_val:02X}             ", offset=byte2.wrapping_add(nes.cpu.y))),
        Mode::IndirectX => log_line.push_str(&format!("(${byte2:02X},X) @ {ind_addr:02X} = {instr_addr:04X} = {instr_val:02X}    ", ind_addr=byte2.wrapping_add(nes.cpu.x))),
        Mode::IndirectY => log_line.push_str(&format!("(${byte2:02X}),Y = {ind_addr:04X} @ {instr_addr:04X} = {instr_val:02X}  ",ind_addr=read_mem_u16_zp(byte2 as u16, nes))),
        Mode::AbsoluteI => log_line.push_str(&format!("(${byte3:02X}{byte2:02X}) = {instr_addr:04X}              ")),
    }

    log_line.push_str(&format!("A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X}", nes.cpu.a, nes.cpu.x, nes.cpu.y, p_to_byte(nes), nes.cpu.s));

    log_line
}


pub fn step_cpu(nes: &mut Nes) {

    if nes.cpu.nmi_interrupt && !nes.cpu.nmi_internal_flag {
        nes.cpu.nmi_internal_flag = true;
        stack_push_u16(nes.cpu.pc, nes);
        stack_push(p_to_byte(nes), nes);
        nes.cpu.pc = read_mem_u16(0xFFFA, nes);
    }

    // Use opcode to index into lookup table
    let opcode = read_mem(nes.cpu.pc, nes);
    let instruction = INSTRUCTIONS[opcode as usize];

    // The two bytes following the opcode
    // Instructions can be between 1 and 3 bytes long, so these bytes may or may not be relevant
    let byte1 = read_mem(nes.cpu.pc.wrapping_add(1), nes);
    let byte2 = read_mem(nes.cpu.pc.wrapping_add(2), nes);

    // Need to explain this, basically instructions that use memory need to decode the target 
    // address from the opcode and following bytes
    // If instruction is only one byte, this is just 0 and won't be used
    let relevant_address = get_instr_addr(instruction.mode, byte1, byte2, nes);

    // Get the value the instruction needs to work on
    // If this is a value from memory, read it from the address we just determined above
    // If this is an immediate value, read it directly from the byte following the opcode
    let instr_val = match instruction.mode {
        Mode::Immediate => byte2,
        Mode::Accumulator | Mode::Implied => 0,
        _ => read_mem(relevant_address, nes),
    };

    let instruction_log = log(opcode, byte1, byte2, relevant_address, instr_val, nes);

    if instruction_log == LOGS[nes.cpu.instruction_count as usize] {
        println!("Matches {}", instruction_log);
    } else {
        panic!("Doesn't match! {}", instruction_log);
    }
    

    let prev_pc = nes.cpu.pc;
    




    // Execute instruction
    (instruction.associated_function)(instr_val, relevant_address, nes);
    



    

    nes.cpu.cycles += instruction.cycles as u64;
    nes.cpu.instruction_count += 1;
    

    // If no jump ocurred, advance the pc according to the length of the instruction 
    // This depends on its addressing mode
    if nes.cpu.pc == prev_pc {
        let offset = instruction.mode.num_bytes();
        nes.cpu.pc = nes.cpu.pc.wrapping_add(offset);
    }
}
