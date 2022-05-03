use std::io;

use crate::util::*;
use crate::hw::*;
use crate::opc::*;
use crate::mem::*;

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

pub fn step_cpu(nes: &mut Nes) {

    // if nes.cpu.nmi_interrupt {
    //     stack_push_u16(nes.cpu.pc, nes);
    //     stack_push(p_to_byte(nes), nes);
    //     nes.cpu.pc = read_mem_u16(0xFFFA, nes);
    //     nes.cpu.nmi_interrupt = false;
    // }

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

    let prev_pc = nes.cpu.pc;

    // Execute instruction
    (instruction.associated_function)(instr_val, relevant_address, nes);
    
    nes.cpu.cycles += instruction.cycles as u64;

    // If no jump ocurred, advance the pc according to the length of the instruction 
    // This depends on its addressing mode
    if nes.cpu.pc != prev_pc {
        let offset = instruction.mode.num_bytes();
        nes.cpu.pc = nes.cpu.pc.wrapping_add(offset);
    }
}
