use crate::nes::cpu::addressing::{dummy_read_from_pc_address, fetch_immediate_from_pc, increment_pc};
use crate::nes::cpu::lookup_table::{ProcessingState, INSTRUCTIONS};
use crate::nes::cpu::lookup_table::ProcessingState::{FetchedOpcode, Finished};
use crate::nes::mem::read_mem;
use crate::nes::Nes;

pub fn fetch_opcode_from_pc_and_increment_pc(nes: &mut Nes) -> ProcessingState {
    let opcode = read_mem(nes.cpu.reg.pc, nes);
    nes.cpu.instr = INSTRUCTIONS[opcode as usize];
    increment_pc(nes);
    FetchedOpcode
}

pub fn immediate_instruction_cycles(cycle: ProcessingState, nes: &mut Nes) -> ProcessingState {
    match cycle {
        FetchedOpcode => {
            fetch_immediate_from_pc(nes);
            nes.cpu.instr.func()(nes);
            increment_pc(nes);
            Finished
        }
        _ => unreachable!(),
    }
}

pub fn nonmemory_instruction_cycles(cycle: ProcessingState, nes: &mut Nes) -> ProcessingState {
    match cycle {
        FetchedOpcode => {
            nes.cpu.instr.func()(nes);
            dummy_read_from_pc_address(nes);
            Finished
        }
        _ => unreachable!(),
    }
}