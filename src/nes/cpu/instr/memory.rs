mod memory_instr;

use serde::{Deserialize, Serialize};
use crate::nes::cpu::instr::instruction::{Instruction, IsInstructionFinished};
use crate::nes::cpu::instr::addressing::{add_lower_address_carry_bit_to_upper_address, add_x_to_low_address_byte, add_x_to_low_indirect_address_byte, add_y_to_low_address_byte, dummy_read_from_address, dummy_read_from_indirect_address, dummy_write_to_address, fetch_high_address_byte_using_indirect_address, fetch_immediate_from_pc, fetch_low_address_byte_using_indirect_address, increment_pc, read_from_address, take_operand_as_high_address_byte, take_operand_as_low_address_byte, take_operand_as_low_indirect_address_byte, write_to_address};
use crate::nes::cpu::operation_funcs::{add_with_carry, and, and_a_with_x_with_upper_address_then_store_in_memory, and_memory_with_a_then_rotate_right, and_memory_with_a_then_set_carry_flag_to_negative_flag, and_memory_with_a_then_shift_right, and_memory_with_s_and_load_into_a_x_s, and_s_with_upper_address_then_store_in_memory, and_x_with_a_then_subtract_memory_and_store_in_x, and_x_with_upper_address_then_store_in_memory, and_y_with_upper_address_then_store_in_memory, arithmetic_shift_left, compare_memory_with_a, compare_memory_with_x, compare_memory_with_y, decrement_memory, decrement_memory_then_compare_with_a, increment_memory, increment_memory_then_subtract_from_a, load_a, load_a_and_x, load_x, load_y, logical_shift_right, no_op, nondeterministic_nonsense, or, rotate_left, rotate_left_then_and_result_with_a, rotate_right, rotate_right_then_and_result_with_a, shift_left_then_or_result_with_a, shift_right_then_xor_result_with_a, store_a, store_a_and_x, store_x, store_y, subtract_with_carry, test_bits_in_memory_with_a, exclusive_or};
use crate::nes::Nes;

pub struct MemoryInstr {
    opc: MemoryOpc,
    config: AddressingConfig,
    state: MemoryState
}

#[derive(Debug)]
pub enum MemoryOpc {
    LDA, LDX, LDY,
    STA, STX, STY,
    ASL, LSR, ROL, ROR,
    AND, ORA, EOR, BIT,
    ADC, SBC,
    DEC, INC,
    CMP, CPX, CPY,
    NOP,
    LAS, LAX, SAX, SHA, SHX, SHY, SHS,
    ANC, ARR, ASR,
    DCP, RLA, RRA, SLO, SRE,
    ISB, SBX, XAA,
}

#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum AddressingConfig {
    Immediate,
    Addressed {
        addr_mode: AddressingMode,
        access_type: MemoryAccessType,
    }
}



#[derive(Default, Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum AddressingMode {
    #[default]
    Absolute,
    AbsoluteX,
    AbsoluteY,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
    IndirectX,
    IndirectY,
}

#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum MemoryAccessType {
    Read,
    Write,
    ReadModifyWrite,
}


type Cycle = u8;
#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize)]
enum MemoryState {
    AddressResolution(Cycle),
    PendingCarry,
    MemoryCycles(Cycle),
    Finished,

}

impl MemoryInstr {
    pub const fn new(opc: MemoryOpc, config: AddressingConfig) -> Self {
        Self {
            opc,
            config,
            state: MemoryState::AddressResolution(0)
        }
    }
    fn operation(&self) -> fn(&mut Nes) {
        match self.opc {
            // Official
            MemoryOpc::LDA => load_a,
            MemoryOpc::LDX => load_x,
            MemoryOpc::LDY => load_y,
            MemoryOpc::STA => store_a,
            MemoryOpc::STX => store_x,
            MemoryOpc::STY => store_y,
            MemoryOpc::ASL => arithmetic_shift_left,
            MemoryOpc::LSR => logical_shift_right,
            MemoryOpc::ROL => rotate_left,
            MemoryOpc::ROR => rotate_right,
            MemoryOpc::AND => and,
            MemoryOpc::ORA => or,
            MemoryOpc::EOR => exclusive_or,
            MemoryOpc::BIT => test_bits_in_memory_with_a,
            MemoryOpc::ADC => add_with_carry,
            MemoryOpc::SBC => subtract_with_carry,
            MemoryOpc::DEC => decrement_memory,
            MemoryOpc::INC => increment_memory,
            MemoryOpc::CMP => compare_memory_with_a,
            MemoryOpc::CPX => compare_memory_with_x,
            MemoryOpc::CPY => compare_memory_with_y,
            MemoryOpc::NOP => no_op,
            // Unofficial
            MemoryOpc::LAS => and_memory_with_s_and_load_into_a_x_s,
            MemoryOpc::LAX => load_a_and_x,
            MemoryOpc::SAX => store_a_and_x,
            MemoryOpc::SHA => and_a_with_x_with_upper_address_then_store_in_memory,
            MemoryOpc::SHX => and_x_with_upper_address_then_store_in_memory,
            MemoryOpc::SHY => and_y_with_upper_address_then_store_in_memory,
            MemoryOpc::SHS => and_s_with_upper_address_then_store_in_memory,
            MemoryOpc::ANC => and_memory_with_a_then_set_carry_flag_to_negative_flag,
            MemoryOpc::ARR => and_memory_with_a_then_rotate_right,
            MemoryOpc::ASR => and_memory_with_a_then_shift_right,
            MemoryOpc::DCP => decrement_memory_then_compare_with_a,
            MemoryOpc::RLA => rotate_left_then_and_result_with_a,
            MemoryOpc::RRA => rotate_right_then_and_result_with_a,
            MemoryOpc::SLO => shift_left_then_or_result_with_a,
            MemoryOpc::SRE => shift_right_then_xor_result_with_a,
            MemoryOpc::ISB => increment_memory_then_subtract_from_a,
            MemoryOpc::SBX => and_x_with_a_then_subtract_memory_and_store_in_x,
            MemoryOpc::XAA => nondeterministic_nonsense,
        }
    }
}

impl Instruction for MemoryInstr {
    fn opcode(&self) -> String {
        format!("{:?}", self.opc)
    }
    fn do_next_instruction_cycle(&mut self, nes: &mut Nes) -> IsInstructionFinished {
        self.state = match self.config {
            AddressingConfig::Immediate =>
                immediate_cycle(nes),
            AddressingConfig::Addressed { addr_mode, access_type } => match self.state {
                s @ MemoryState::AddressResolution(_) =>
                    address_resolution_cycles(addr_mode, s, nes),
                MemoryState::PendingCarry =>
                    handle_upper_address_overflow(access_type, nes),
                s @ MemoryState::MemoryCycles(_) =>
                    memory_cycles(s, access_type, nes),
                state => state,
            }
        };
        self.state == MemoryState::Finished
    }
}

pub fn immediate_cycle(nes: &mut Nes) -> MemoryState {
    fetch_immediate_from_pc(nes);
    nes.cpu.instr.func()(nes);
    increment_pc(nes);
    MemoryState::Finished
}

pub fn address_resolution_cycles(addr_mode: AddressingMode, state: MemoryState, nes: &mut Nes) -> MemoryState {
    match addr_mode {
        AddressingMode::ZeroPage => zero_page_cycle(nes),
        AddressingMode::ZeroPageX => zero_page_x_cycles(state, nes),
        AddressingMode::ZeroPageY => zero_page_y_cycles(state, nes),
        AddressingMode::Absolute => absolute_cycles(state, nes),
        AddressingMode::AbsoluteX => absolute_x_cycles(state, nes),
        AddressingMode::AbsoluteY => absolute_y_cycles(state, nes),
        AddressingMode::IndirectX => indirect_x_cycles(state, nes),
        AddressingMode::IndirectY => indirect_y_cycles(state, nes),
    }
}

fn zero_page_cycle(nes: &mut Nes) -> MemoryState {
    take_operand_as_low_address_byte(nes);
    increment_pc(nes);
    MemoryState::MemoryCycles(0)   
}

fn zero_page_x_cycles(state: MemoryState, nes: &mut Nes) -> MemoryState {
    match state {
        MemoryState::AddressResolution(0) => {
            take_operand_as_low_address_byte(nes);
            increment_pc(nes);
            MemoryState::AddressResolution(1)
        }
        MemoryState::AddressResolution(1) => {
            dummy_read_from_address(nes);
            add_x_to_low_address_byte(nes);
            MemoryState::MemoryCycles(0)
        }
        _ => state,
    }
}

fn zero_page_y_cycles(state: MemoryState, nes: &mut Nes) -> MemoryState {
    match state {
        MemoryState::AddressResolution(0) => {
            take_operand_as_low_address_byte(nes);
            increment_pc(nes);
            MemoryState::AddressResolution(1)
        }
        MemoryState::AddressResolution(1) => {
            dummy_read_from_address(nes);
            add_y_to_low_address_byte(nes);
            MemoryState::MemoryCycles(0)
        }
        _ => state,
    }
}

fn absolute_cycles(state: MemoryState, nes: &mut Nes) -> MemoryState {
    match state {
        MemoryState::AddressResolution(0) => {
            take_operand_as_low_address_byte(nes);
            increment_pc(nes);
            MemoryState::AddressResolution(1)
        }
        MemoryState::AddressResolution(1) => {
            take_operand_as_high_address_byte(nes);
            increment_pc(nes);
            MemoryState::MemoryCycles(0)
        }
        _ => state,
    }
}

fn absolute_x_cycles(state: MemoryState, nes: &mut Nes) -> MemoryState {
    match state {
        MemoryState::AddressResolution(0) => {
            take_operand_as_low_address_byte(nes);
            increment_pc(nes);
            MemoryState::AddressResolution(1)
        }
        MemoryState::AddressResolution(1) => {
            take_operand_as_high_address_byte(nes);
            add_x_to_low_address_byte(nes);
            increment_pc(nes);
            MemoryState::PendingCarry
        }
        _ => state,
    }
}

fn absolute_y_cycles(state: MemoryState, nes: &mut Nes) -> MemoryState {
    match state {
        MemoryState::AddressResolution(0) => {
            take_operand_as_low_address_byte(nes);
            increment_pc(nes);
            MemoryState::AddressResolution(1)
        }
        MemoryState::AddressResolution(1) => {
            take_operand_as_high_address_byte(nes);
            add_y_to_low_address_byte(nes);
            increment_pc(nes);
            MemoryState::PendingCarry
        }
        _ => state,
    }
}

fn indirect_x_cycles(state: MemoryState, nes: &mut Nes) -> MemoryState {
    match state {
        MemoryState::AddressResolution(0) => {
            take_operand_as_low_indirect_address_byte(nes);
            increment_pc(nes);
            MemoryState::AddressResolution(1)
        }
        MemoryState::AddressResolution(1) => {
            dummy_read_from_indirect_address(nes);
            add_x_to_low_indirect_address_byte(nes);
            MemoryState::AddressResolution(2)
        }
        MemoryState::AddressResolution(2) => {
            fetch_low_address_byte_using_indirect_address(nes);
            MemoryState::AddressResolution(3)
        }
        MemoryState::AddressResolution(3) => {
            fetch_high_address_byte_using_indirect_address(nes);
            MemoryState::MemoryCycles(0)
        }
        _ => state,
    }
}

fn indirect_y_cycles(state: MemoryState, nes: &mut Nes) -> MemoryState {
    match state {
        MemoryState::AddressResolution(0) => {
            take_operand_as_low_indirect_address_byte(nes);
            increment_pc(nes);
            MemoryState::AddressResolution(1)
        }
        MemoryState::AddressResolution(1) => {
            fetch_low_address_byte_using_indirect_address(nes);
            MemoryState::AddressResolution(2)
        }
        MemoryState::AddressResolution(2) => {
            fetch_high_address_byte_using_indirect_address(nes);
            add_y_to_low_address_byte(nes);
            MemoryState::PendingCarry
        }
        _ => state,
    }
}

pub fn handle_upper_address_overflow(category: MemoryAccessType, nes: &mut Nes) -> MemoryState {
    match (category, nes.cpu.ireg.carry_out) {
        (MemoryAccessType::Read, true) => {
            dummy_read_from_address(nes);
            add_lower_address_carry_bit_to_upper_address(nes);
            MemoryState::MemoryCycles(0)
        }
        // Early finish if addr + index doesn't cross page
        (MemoryAccessType::Read, false) => {
            read_from_address(nes);
            nes.cpu.instr.func()(nes);
            MemoryState::Finished
        }
        (MemoryAccessType::Write | MemoryAccessType::ReadModifyWrite, _) => {
            dummy_read_from_address(nes);
            add_lower_address_carry_bit_to_upper_address(nes);
            MemoryState::MemoryCycles(0)
        }
    }
}

pub fn memory_cycles(state: MemoryState, access_type: MemoryAccessType, nes: &mut Nes) -> MemoryState {
    match access_type {
        MemoryAccessType::Read => match state {
            MemoryState::MemoryCycles(0) => {
                read_from_address(nes);
                nes.cpu.instr.func()(nes);
                MemoryState::Finished
            }
            _ => state,
        }
        MemoryAccessType::Write => match state {
            MemoryState::MemoryCycles(0) => {
                nes.cpu.instr.func()(nes);
                write_to_address(nes);
                MemoryState::Finished
            }
            _ => state,
        }
        MemoryAccessType::ReadModifyWrite => match state {
            MemoryState::MemoryCycles(0) => {
                read_from_address(nes);
                MemoryState::MemoryCycles(1)
            }
            MemoryState::MemoryCycles(1) => {
                dummy_write_to_address(nes);
                nes.cpu.instr.func()(nes);
                MemoryState::MemoryCycles(2)
            }
            MemoryState::MemoryCycles(2) => {
                write_to_address(nes);
                MemoryState::Finished
            }
            _ => state,
        }
    }
}
