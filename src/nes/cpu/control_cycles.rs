use crate::nes::cpu::addressing::{copy_address_to_pc, decrement_s, dummy_read_from_pc_address, dummy_read_from_stack, fetch_high_address_byte_using_indirect_address, fetch_low_address_byte_using_indirect_address, fetch_lower_pc_from_interrupt_vector, fetch_upper_pc_from_interrupt_vector, increment_pc, increment_s, pull_a_from_stack, pull_lower_pc_from_stack, pull_p_from_stack, pull_upper_pc_from_stack, push_a_to_stack, push_lower_pc_to_stack, push_p_to_stack_during_break_or_php, push_upper_pc_to_stack, take_operand_as_high_address_byte, take_operand_as_high_indirect_address_byte, take_operand_as_low_address_byte, take_operand_as_low_indirect_address_byte};
use crate::nes::cpu::lookup_table::{ProcessingState};
use crate::nes::cpu::lookup_table::ProcessingState::{FetchedOpcode, Finished, SimpleCycle};
use crate::nes::cpu::lookup_table::Mode::{Absolute, AbsoluteI};
use crate::nes::cpu::lookup_table::Name::{BRK, JMP, JSR, PHA, PHP, PLA, PLP, RTI, RTS};
use crate::nes::cpu::operation_funcs::{set_interrupt_inhibit_flag, update_p_nz};
use crate::nes::Nes;

pub fn control_instruction_cycles(cycle: ProcessingState, nes: &mut Nes) -> ProcessingState {
    match (nes.cpu.instr.name, nes.cpu.instr.mode) {
        (BRK, _) => match cycle {
            FetchedOpcode => {
                dummy_read_from_pc_address(nes);
                increment_pc(nes);
                nes.cpu.interrupts.interrupt_vector = 0xFFFE;
                SimpleCycle(0)
            }
            SimpleCycle(0) => {
                push_upper_pc_to_stack(nes);
                decrement_s(nes);
                SimpleCycle(1)
            }
            SimpleCycle(1) => {
                push_lower_pc_to_stack(nes);
                decrement_s(nes);
                SimpleCycle(2)
            }
            SimpleCycle(2) => {
                push_p_to_stack_during_break_or_php(nes);
                decrement_s(nes);
                SimpleCycle(3)
            }
            SimpleCycle(3) => {
                fetch_lower_pc_from_interrupt_vector(nes);
                set_interrupt_inhibit_flag(nes);
                SimpleCycle(4)
            }
            SimpleCycle(4) => {
                fetch_upper_pc_from_interrupt_vector(nes);
                Finished
            }
            _ => unreachable!(),
        },
        (RTI, _) => match cycle {
            FetchedOpcode => {
                dummy_read_from_pc_address(nes);
                SimpleCycle(0)
            }
            SimpleCycle(0) => {
                increment_s(nes);
                SimpleCycle(1)
            }
            SimpleCycle(1) => {
                pull_p_from_stack(nes);
                increment_s(nes);
                SimpleCycle(2)
            }
            SimpleCycle(2) => {
                pull_lower_pc_from_stack(nes);
                increment_s(nes);
                SimpleCycle(3)
            }
            SimpleCycle(3) => {
                pull_upper_pc_from_stack(nes);
                Finished
            }
            _ => unreachable!(),
        },
        (RTS, _) => match cycle {
            FetchedOpcode => {
                dummy_read_from_pc_address(nes);
                SimpleCycle(0)
            }
            SimpleCycle(0) => {
                increment_s(nes);
                SimpleCycle(1)
            }
            SimpleCycle(1) => {
                pull_lower_pc_from_stack(nes);
                increment_s(nes);
                SimpleCycle(2)
            }
            SimpleCycle(2) => {
                pull_upper_pc_from_stack(nes);
                SimpleCycle(3)
            }
            SimpleCycle(3) => {
                increment_pc(nes);
                Finished
            }
            _ => unreachable!(),
        },
        (JSR, _) => match cycle {
            FetchedOpcode => {
                take_operand_as_low_address_byte(nes);
                increment_pc(nes);
                SimpleCycle(0)
            }
            SimpleCycle(0) => {
                dummy_read_from_stack(nes);
                SimpleCycle(1)
            }
            SimpleCycle(1) => {
                push_upper_pc_to_stack(nes);
                decrement_s(nes);
                SimpleCycle(2)
            }
            SimpleCycle(2) => {
                push_lower_pc_to_stack(nes);
                decrement_s(nes);
                SimpleCycle(3)
            }
            SimpleCycle(3) => {
                take_operand_as_high_address_byte(nes);
                copy_address_to_pc(nes);
                Finished
            }
            _ => unreachable!(),
        },
        (PHA, _) => match cycle {
            FetchedOpcode => {
                dummy_read_from_pc_address(nes);
                SimpleCycle(0)
            }
            SimpleCycle(0) => {
                push_a_to_stack(nes);
                decrement_s(nes);
                Finished
            }
            _ => unreachable!(),
        },
        (PHP, _) => match cycle {
            FetchedOpcode => {
                dummy_read_from_pc_address(nes);
                SimpleCycle(0)
            }
            SimpleCycle(0) => {
                push_p_to_stack_during_break_or_php(nes);
                decrement_s(nes);
                Finished
            }
            _ => unreachable!(),
        },
        (PLA, _) => match cycle {
            FetchedOpcode => {
                dummy_read_from_pc_address(nes);
                SimpleCycle(0)
            }
            SimpleCycle(0) => {
                increment_s(nes);
                SimpleCycle(1)
            }
            SimpleCycle(1) => {
                pull_a_from_stack(nes);
                update_p_nz(nes, nes.cpu.reg.a);
                Finished
            }
            _ => unreachable!(),
        },
        (PLP, _) => match cycle {
            FetchedOpcode => {
                dummy_read_from_pc_address(nes);
                SimpleCycle(0)
            }
            SimpleCycle(0) => {
                increment_s(nes);
                SimpleCycle(1)
            }
            SimpleCycle(1) => {
                pull_p_from_stack(nes);
                Finished
            }
            _ => unreachable!(),
        },
        (JMP, Absolute) => match cycle {
            FetchedOpcode => {
                take_operand_as_low_address_byte(nes);
                increment_pc(nes);
                SimpleCycle(0)
            }
            SimpleCycle(0) => {
                take_operand_as_high_address_byte(nes);
                copy_address_to_pc(nes);
                Finished
            }
            _ => unreachable!(),
        },
        (JMP, AbsoluteI) => match cycle {
            FetchedOpcode => {
                take_operand_as_low_indirect_address_byte(nes);
                increment_pc(nes);
                SimpleCycle(0)
            }
            SimpleCycle(0) => {
                take_operand_as_high_indirect_address_byte(nes);
                increment_pc(nes);
                SimpleCycle(1)
            }
            SimpleCycle(1) => {
                fetch_low_address_byte_using_indirect_address(nes);
                SimpleCycle(2)
            }
            SimpleCycle(2) => {
                fetch_high_address_byte_using_indirect_address(nes);
                copy_address_to_pc(nes);
                Finished
            }
            _ => unreachable!(),
        },
        _ => unreachable!(),
    }
}