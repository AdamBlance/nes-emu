use crate::nes::cpu::addressing::*;
use crate::nes::cpu::lookup_table::{Category::*, Mode::*, Name::*};
use crate::nes::cpu::operation_funcs::{set_interrupt_inhibit_flag, update_p_nz};
use crate::nes::Nes;

pub fn control_instruction_cycles(nes: &mut Nes, instruction_cycle: i8) {
    match (nes.cpu.instruction.name, nes.cpu.instruction.mode) {
        (BRK, _) => match instruction_cycle {
            1 => {
                dummy_read_from_pc_address(nes);
                increment_pc(nes);
                nes.cpu.interrupt_vector = 0xFFFE;
            }
            2 => {
                push_upper_pc_to_stack(nes);
                decrement_s(nes);
            }
            3 => {
                push_lower_pc_to_stack(nes);
                decrement_s(nes);
            }
            4 => {
                push_p_to_stack_during_break_or_php(nes);
                decrement_s(nes);
            }
            5 => {
                fetch_lower_pc_from_interrupt_vector(nes);
                set_interrupt_inhibit_flag(nes);
            }
            6 => {
                fetch_upper_pc_from_interrupt_vector(nes);
                nes.cpu.instruction_done = true;
            }
            _ => unreachable!(),
        },
        (RTI, _) => match instruction_cycle {
            1 => {
                dummy_read_from_pc_address(nes);
            }
            2 => {
                increment_s(nes);
            }
            3 => {
                pull_p_from_stack(nes);
                increment_s(nes);
            }
            4 => {
                pull_lower_pc_from_stack(nes);
                increment_s(nes);
            }
            5 => {
                pull_upper_pc_from_stack(nes);
                nes.cpu.instruction_done = true;
            }
            _ => unreachable!(),
        },
        (RTS, _) => match instruction_cycle {
            1 => {
                dummy_read_from_pc_address(nes);
            }
            2 => {
                increment_s(nes);
            }
            3 => {
                pull_lower_pc_from_stack(nes);
                increment_s(nes);
            }
            4 => {
                pull_upper_pc_from_stack(nes);
            }
            5 => {
                increment_pc(nes);
                nes.cpu.instruction_done = true;
            }
            _ => unreachable!(),
        },
        (JSR, _) => match instruction_cycle {
            1 => {
                take_operand_as_low_address_byte(nes);
                increment_pc(nes);
            }
            2 => {
                dummy_read_from_stack(nes);
            }
            3 => {
                push_upper_pc_to_stack(nes);
                decrement_s(nes);
            }
            4 => {
                push_lower_pc_to_stack(nes);
                decrement_s(nes);
            }
            5 => {
                take_operand_as_high_address_byte(nes);
                copy_address_to_pc(nes);
                nes.cpu.instruction_done = true;
            }
            _ => unreachable!(),
        },
        (PHA, _) => match instruction_cycle {
            1 => {
                dummy_read_from_pc_address(nes);
            }
            2 => {
                push_a_to_stack(nes);
                decrement_s(nes);
                nes.cpu.instruction_done = true;
            }
            _ => unreachable!(),
        },
        (PHP, _) => match instruction_cycle {
            1 => {
                dummy_read_from_pc_address(nes);
            }
            2 => {
                push_p_to_stack_during_break_or_php(nes);
                decrement_s(nes);
                nes.cpu.instruction_done = true;
            }
            _ => unreachable!(),
        },
        (PLA, _) => match instruction_cycle {
            1 => {
                dummy_read_from_pc_address(nes);
            }
            2 => {
                increment_s(nes);
            }
            3 => {
                pull_a_from_stack(nes);
                update_p_nz(nes, nes.cpu.a);
                nes.cpu.instruction_done = true;
            }
            _ => unreachable!(),
        },
        (PLP, _) => match instruction_cycle {
            1 => {
                dummy_read_from_pc_address(nes);
            }
            2 => {
                increment_s(nes);
            }
            3 => {
                pull_p_from_stack(nes);
                nes.cpu.instruction_done = true;
            }
            _ => unreachable!(),
        },
        (JMP, Absolute) => match instruction_cycle {
            1 => {
                take_operand_as_low_address_byte(nes);
                increment_pc(nes);
            }
            2 => {
                take_operand_as_high_address_byte(nes);
                copy_address_to_pc(nes);
                nes.cpu.instruction_done = true;
            }
            _ => unreachable!(),
        },
        (JMP, AbsoluteI) => match instruction_cycle {
            1 => {
                take_operand_as_low_indirect_address_byte(nes);
                increment_pc(nes);
            }
            2 => {
                take_operand_as_high_indirect_address_byte(nes);
                increment_pc(nes);
            }
            3 => {
                fetch_low_address_byte_using_indirect_address(nes);
            }
            4 => {
                fetch_high_address_byte_using_indirect_address(nes);
                copy_address_to_pc(nes);
                nes.cpu.instruction_done = true;
            }
            _ => unreachable!(),
        },
        _ => unreachable!(),
    };
}

pub fn address_resolution_cycles(nes: &mut Nes, instruction_cycle: i8) {
    match nes.cpu.instruction.mode {
        ZeroPage => match instruction_cycle {
            1 => {
                take_operand_as_low_address_byte(nes);
                increment_pc(nes);
            }
            2 => (),
            _ => (),
        },
        ZeroPageX => match instruction_cycle {
            1 => {
                take_operand_as_low_address_byte(nes);
                increment_pc(nes);
            }
            2 => {
                dummy_read_from_address(nes);
                add_x_to_low_address_byte(nes);
            }
            _ => (),
        },
        ZeroPageY => match instruction_cycle {
            1 => {
                take_operand_as_low_address_byte(nes);
                increment_pc(nes);
            }
            2 => {
                dummy_read_from_address(nes);
                add_y_to_low_address_byte(nes);
            }
            _ => (),
        },
        Absolute => match instruction_cycle {
            1 => {
                take_operand_as_low_address_byte(nes);
                increment_pc(nes);
            }
            2 => {
                take_operand_as_high_address_byte(nes);
                increment_pc(nes);
            }
            _ => (),
        },
        AbsoluteX => match instruction_cycle {
            1 => {
                take_operand_as_low_address_byte(nes);
                increment_pc(nes);
            }
            2 => {
                take_operand_as_high_address_byte(nes);
                add_x_to_low_address_byte(nes);
                increment_pc(nes);
            }
            _ => (),
        },
        AbsoluteY => match instruction_cycle {
            1 => {
                take_operand_as_low_address_byte(nes);
                increment_pc(nes);
            }
            2 => {
                take_operand_as_high_address_byte(nes);
                add_y_to_low_address_byte(nes);
                increment_pc(nes);
            }
            _ => (),
        },
        IndirectX => match instruction_cycle {
            1 => {
                take_operand_as_low_indirect_address_byte(nes);
                increment_pc(nes);
            }
            2 => {
                dummy_read_from_indirect_address(nes);
                add_x_to_low_indirect_address_byte(nes);
            }
            3 => {
                fetch_low_address_byte_using_indirect_address(nes);
            }
            4 => {
                fetch_high_address_byte_using_indirect_address(nes);
            }
            _ => (),
        },
        IndirectY => match instruction_cycle {
            1 => {
                take_operand_as_low_indirect_address_byte(nes);
                increment_pc(nes);
            }
            2 => {
                fetch_low_address_byte_using_indirect_address(nes);
            }
            3 => {
                fetch_high_address_byte_using_indirect_address(nes);
                add_y_to_low_address_byte(nes);
            }
            _ => (),
        },
        _ => unreachable!(),
    }
}

pub fn branch_instruction_cycles(nes: &mut Nes, instruction_cycle: i8) {
    match instruction_cycle {
        1 => {
            fetch_branch_offset_from_pc(nes);
            increment_pc(nes);
            nes.cpu.branching = match nes.cpu.instruction.name {
                BCC => !nes.cpu.p_c,
                BCS => nes.cpu.p_c,
                BVC => !nes.cpu.p_v,
                BVS => nes.cpu.p_v,
                BNE => !nes.cpu.p_z,
                BEQ => nes.cpu.p_z,
                BPL => !nes.cpu.p_n,
                BMI => nes.cpu.p_n,
                _ => unreachable!(),
            };
            // Continue to next instruction if branch was not taken
            if !nes.cpu.branching {
                nes.cpu.instruction_done = true;
            }
        }
        2 => {
            let prev_pcl = nes.cpu.pc as u8;
            let (new_pcl, overflow) = prev_pcl.overflowing_add_signed(nes.cpu.branch_offset as i8);
            nes.cpu.internal_carry_out = overflow;
            nes.cpu.set_lower_pc(new_pcl);
            // If branch didn't cross page boundary, continue to next instruction
            if !nes.cpu.internal_carry_out {
                nes.cpu.instruction_done = true;
            }
        }
        3 => {
            // Fix upper PC if page was crossed
            if nes.cpu.branch_offset > 0x7F {
                nes.cpu.pc = nes.cpu.pc.wrapping_sub(1 << 8);
            } else {
                nes.cpu.pc = nes.cpu.pc.wrapping_add(1 << 8);
            }
            nes.cpu.instruction_done = true;
        }
        _ => unreachable!(),
    }
}

pub fn processing_cycles(nes: &mut Nes, instruction_cycle: i8) {
    let func = nes.cpu.instruction.func();
    let offset = match nes.cpu.instruction.mode {
        Absolute | ZeroPage | ZeroPageX | ZeroPageY | IndirectX | Immediate => 1,
        _ => 0,
    };
    let adjusted_cycle = instruction_cycle + offset;
    match nes.cpu.instruction.category {
        Read => match adjusted_cycle {
            1 => {
                read_from_address(nes);
                add_lower_address_carry_bit_to_upper_address(nes);
                // Continue to next instruction if page wasn't crossed
                if !nes.cpu.internal_carry_out {
                    func(nes);
                    nes.cpu.instruction_done = true;
                }
            }
            2 => {
                read_from_address(nes);
                func(nes);
                nes.cpu.instruction_done = true;
            }
            _ => unreachable!(),
        },
        Write => match adjusted_cycle {
            1 => {
                dummy_read_from_address(nes);
                add_lower_address_carry_bit_to_upper_address(nes);
            }
            2 => {
                func(nes);
                write_to_address(nes);
                nes.cpu.instruction_done = true;
            }
            _ => unreachable!(),
        },
        ReadModifyWrite => match adjusted_cycle {
            1 => {
                dummy_read_from_address(nes);
                add_lower_address_carry_bit_to_upper_address(nes);
            }
            2 => read_from_address(nes),
            3 => {
                write_to_address(nes);
                func(nes);
            }
            4 => {
                write_to_address(nes);
                nes.cpu.instruction_done = true;
            }
            _ => unreachable!(),
        },
        _ => unreachable!(),
    }
}
