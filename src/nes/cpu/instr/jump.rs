use crate::nes::cpu::instr::instruction::{Instruction, IsInstructionFinished};
use crate::nes::cpu::instr::addressing::{copy_address_to_pc, decrement_s, dummy_read_from_stack, fetch_high_address_byte_using_indirect_address, fetch_low_address_byte_using_indirect_address, increment_pc, push_lower_pc_to_stack, push_upper_pc_to_stack, take_operand_as_high_address_byte, take_operand_as_high_indirect_address_byte, take_operand_as_low_address_byte, take_operand_as_low_indirect_address_byte};
use crate::nes::Nes;

pub struct JumpInstr {
    opc: JumpOpc,
    jump_type: JumpType,
    state: JumpState,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum JumpOpc {
    JMP,
    JSR
}

#[derive(Copy, Clone)]
pub enum JumpType {
    JumpToAddr,
    JumpToPointerAddr,
    JumpToSubroutine
}
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum JumpState {
    Cycle(u8),
    Finished
}

impl JumpInstr {
    pub const fn new(opc: JumpOpc, jump_type: JumpType) -> Self {
        Self {
            opc,
            jump_type,
            state: JumpState::Cycle(0),
        }
    }
}

impl Instruction for JumpInstr {
    fn opcode(&self) -> String {
        format!("{:?}", self.opc)
    }
    fn do_next_instruction_cycle(&mut self, nes: &mut Nes) -> IsInstructionFinished {
        self.state = match self.jump_type {
            JumpType::JumpToSubroutine => jump_to_subroutine_cycles(self.state, nes),
            JumpType::JumpToAddr => jump_to_addr_cycles(self.state, nes),
            JumpType::JumpToPointerAddr => jump_to_pointer_addr_cycles(self.state, nes)
        };
        self.state == JumpState::Finished
    }
}

fn jump_to_subroutine_cycles(state: JumpState, nes: &mut Nes) -> JumpState {
    match state {
        JumpState::Cycle(0) => {
            take_operand_as_low_address_byte(nes);
            increment_pc(nes);
            JumpState::Cycle(1)
        }
        JumpState::Cycle(1) => {
            dummy_read_from_stack(nes);
            JumpState::Cycle(2)
        }
        JumpState::Cycle(2) => {
            push_upper_pc_to_stack(nes);
            decrement_s(nes);
            JumpState::Cycle(3)
        }
        JumpState::Cycle(3) => {
            push_lower_pc_to_stack(nes);
            decrement_s(nes);
            JumpState::Cycle(4)
        }
        JumpState::Cycle(4) => {
            take_operand_as_high_address_byte(nes);
            copy_address_to_pc(nes);
            JumpState::Finished
        }
        _ => state,
    }
}

fn jump_to_addr_cycles(state: JumpState, nes: &mut Nes) -> JumpState {
    match state {
        JumpState::Cycle(0) => {
            take_operand_as_low_address_byte(nes);
            increment_pc(nes);
            JumpState::Cycle(1)
        }
        JumpState::Cycle(1) => {
            take_operand_as_high_address_byte(nes);
            copy_address_to_pc(nes);
            JumpState::Finished
        }
        _ => state,
    }
}

fn jump_to_pointer_addr_cycles(state: JumpState, nes: &mut Nes) -> JumpState {
    match state {
        JumpState::Cycle(0) => {
            take_operand_as_low_indirect_address_byte(nes);
            increment_pc(nes);
            JumpState::Cycle(1)
        }
        JumpState::Cycle(1) => {
            take_operand_as_high_indirect_address_byte(nes);
            increment_pc(nes);
            JumpState::Cycle(2)
        }
        JumpState::Cycle(2) => {
            fetch_low_address_byte_using_indirect_address(nes);
            JumpState::Cycle(3)
        }
        JumpState::Cycle(3) => {
            fetch_high_address_byte_using_indirect_address(nes);
            copy_address_to_pc(nes);
            JumpState::Finished
        }
        _ => state,
    }
}
