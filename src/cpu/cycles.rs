use crate::nes::Nes;
use crate::cpu::lookup_table::{Name::*, Mode::*};
use crate::cpu::addressing::*;
use crate::cpu::operation_funcs::{set_interrupt_inhibit_flag, update_p_nz};


pub fn control_instruction_cycles(nes: &mut Nes, instruction_cycle: i8) {
    match (nes.cpu.instruction.name, nes.cpu.instruction.mode) {
        (BRK, _) => { match instruction_cycle {
            1 => {dummy_read_from_pc_address(nes); increment_pc(nes); nes.cpu.interrupt_vector = 0xFFFE;}
            2 => {push_upper_pc_to_stack(nes); decrement_s(nes);}
            3 => {push_lower_pc_to_stack(nes); decrement_s(nes);}
            4 => {push_p_to_stack_during_break(nes); decrement_s(nes);}
            5 => {fetch_lower_pc_from_interrupt_vector(nes); set_interrupt_inhibit_flag(nes);}
            6 => {fetch_upper_pc_from_interrupt_vector(nes); nes.cpu.instruction_done = true;}
            _ => unreachable!(),
        }}
        (RTI, _) => { match instruction_cycle {
            1 => {dummy_read_from_pc_address(nes);}
            2 => {increment_s(nes);}
            3 => {pull_p_from_stack(nes); increment_s(nes);}
            4 => {pull_lower_pc_from_stack(nes); increment_s(nes);}
            5 => {pull_upper_pc_from_stack(nes); nes.cpu.instruction_done = true;}
            _ => unreachable!(),
        }}
        (RTS, _) => { match instruction_cycle {
            1 => {dummy_read_from_pc_address(nes);}
            2 => {increment_s(nes);}
            3 => {pull_lower_pc_from_stack(nes); increment_s(nes);}
            4 => {pull_upper_pc_from_stack(nes);}
            5 => {increment_pc(nes); nes.cpu.instruction_done = true;}
            _ => unreachable!(),
        }}
        (JSR, _) => { match instruction_cycle {
            1 => {take_operand_as_low_address_byte(nes); increment_pc(nes);}
            2 => {none(nes);}
            3 => {push_upper_pc_to_stack(nes); decrement_s(nes);}
            4 => {push_lower_pc_to_stack(nes); decrement_s(nes);}
            5 => {take_operand_as_high_address_byte(nes); copy_address_to_pc(nes); nes.cpu.instruction_done = true;} 
            _ => unreachable!(),
        }}
        (PHA, _) => { match instruction_cycle {
            1 => {dummy_read_from_pc_address(nes);}
            2 => {push_a_to_stack(nes); decrement_s(nes); nes.cpu.instruction_done = true;}
            _ => unreachable!(),
        }}
        (PHP, _) => { match instruction_cycle {
            1 => {dummy_read_from_pc_address(nes);}
            2 => {push_p_to_stack(nes); decrement_s(nes); nes.cpu.instruction_done = true;}
            _ => unreachable!(),
        }}
        (PLA, _) => { match instruction_cycle {
            1 => {dummy_read_from_pc_address(nes);}
            2 => {increment_s(nes);}
            3 => {pull_a_from_stack(nes); update_p_nz(nes, nes.cpu.a); nes.cpu.instruction_done = true;}
            _ => unreachable!(),
        }}
        (PLP, _) => { match instruction_cycle {
            1 => {dummy_read_from_pc_address(nes);}
            2 => {increment_s(nes);}
            3 => {pull_p_from_stack(nes); nes.cpu.instruction_done = true;}
            _ => unreachable!(),
        }}
        (JMP, Absolute) => { match instruction_cycle {
            1 => {take_operand_as_low_address_byte(nes); increment_pc(nes);}
            2 => {take_operand_as_high_address_byte(nes); copy_address_to_pc(nes); nes.cpu.instruction_done = true;}
            _ => unreachable!(),
        }}
        (JMP, AbsoluteI) => { match instruction_cycle {
            1 => {take_operand_as_low_indirect_address_byte(nes); increment_pc(nes);}
            2 => {take_operand_as_high_indirect_address_byte(nes); increment_pc(nes);}
            3 => {fetch_low_address_byte_using_indirect_address(nes);}
            4 => {fetch_high_address_byte_using_indirect_address(nes); copy_address_to_pc(nes); nes.cpu.instruction_done = true;}
            _ => unreachable!(),
        }}
        _ => unreachable!(),
    };
}

pub fn address_resolution_cycles(nes: &mut Nes, instruction_cycle: i8) {
    match nes.cpu.instruction.mode {
        ZeroPage => { match  instruction_cycle  {
            1 => {take_operand_as_low_address_byte(nes); increment_pc(nes);}
            _ => unreachable!(),
        }}
        ZeroPageX => { match  instruction_cycle  {
            1 => {take_operand_as_low_address_byte(nes); increment_pc(nes);}
            2 => {dummy_read_from_address(nes); add_x_to_low_address_byte(nes);}
            _ => unreachable!(),
        }}
        ZeroPageY => { match  instruction_cycle  {
            1 => {take_operand_as_low_address_byte(nes); increment_pc(nes);}
            2 => {dummy_read_from_address(nes); add_y_to_low_address_byte(nes);}
            _ => unreachable!(),
        }}
        Absolute => { match  instruction_cycle  {
            1 => {take_operand_as_low_address_byte(nes); increment_pc(nes);}
            2 => {take_operand_as_high_address_byte(nes); increment_pc(nes);}
            _ => unreachable!(),
        }}
        AbsoluteX => { match  instruction_cycle  {
            1 => {take_operand_as_low_address_byte(nes); increment_pc(nes);}
            2 => {take_operand_as_high_address_byte(nes); add_x_to_low_address_byte(nes); increment_pc(nes);}
            _ => unreachable!(),
        }}
        AbsoluteY => { match  instruction_cycle  {
            1 => {take_operand_as_low_address_byte(nes); increment_pc(nes);}
            2 => {take_operand_as_high_address_byte(nes); add_y_to_low_address_byte(nes); increment_pc(nes);}
            _ => unreachable!(),
        }}
        IndirectX => { match  instruction_cycle  {
            1 => {take_operand_as_low_indirect_address_byte(nes); increment_pc(nes);}
            2 => {dummy_read_from_indirect_address(nes); add_x_to_low_indirect_address_byte(nes);}
            3 => {fetch_low_address_byte_using_indirect_address(nes);}
            4 => {fetch_high_address_byte_using_indirect_address(nes);}
            _ => unreachable!(),
        }}
        IndirectY => { match  instruction_cycle  {
            1 => {take_operand_as_low_indirect_address_byte(nes); increment_pc(nes);}
            2 => {fetch_low_address_byte_using_indirect_address(nes);}
            3 => {fetch_high_address_byte_using_indirect_address(nes); add_y_to_low_address_byte(nes);}
            _ => unreachable!(),
        }}
        _ => unreachable!(),
    }
}
