use super::addressing::*;
use super::cycles::{
    address_resolution_cycles, branch_instruction_cycles, control_instruction_cycles,
    processing_cycles,
};
use super::lookup_table::{Category::*, INSTRUCTIONS};
use super::operation_funcs::set_interrupt_inhibit_flag;
use crate::nes::mem::read_mem;
use crate::nes::Nes;

pub fn step_cpu(nes: &mut Nes) -> bool {
    nes.cart.cpu_tick();

    if nes.cpu.instruction_cycle == 0 {
        if nes.cpu.nmi_pending {
            match nes.cpu.interrupt_cycle {
                0 => {
                    dummy_read_from_pc_address(nes);
                    nes.cpu.irq_pending = false;
                    nes.cpu.interrupt_vector = 0xFFFA;
                }
                1 => dummy_read_from_pc_address(nes),
                2 => {
                    push_upper_pc_to_stack(nes);
                    decrement_s(nes);
                }
                3 => {
                    push_lower_pc_to_stack(nes);
                    decrement_s(nes);
                }
                4 => {
                    push_p_to_stack_during_interrupt(nes);
                    decrement_s(nes);
                }
                5 => {
                    fetch_lower_pc_from_interrupt_vector(nes);
                    set_interrupt_inhibit_flag(nes)
                }
                6 => {
                    fetch_upper_pc_from_interrupt_vector(nes);
                    nes.cpu.nmi_edge_detector_output = false;
                    nes.cpu.nmi_pending = false;
                    nes.cpu.interrupt_cycle = -1;
                }
                _ => unreachable!(),
            }
            nes.cpu.interrupt_cycle += 1;
        }
        // Ignore IRQ until the interrupt inhibit status flag is cleared
        else if nes.cpu.irq_pending && !nes.cpu.p_i {
            match nes.cpu.interrupt_cycle {
                0 => {
                    dummy_read_from_pc_address(nes);
                    nes.cpu.interrupt_vector = 0xFFFE;
                }
                1 => dummy_read_from_pc_address(nes),
                2 => {
                    push_upper_pc_to_stack(nes);
                    decrement_s(nes);
                }
                3 => {
                    push_lower_pc_to_stack(nes);
                    decrement_s(nes);
                }
                4 => {
                    push_p_to_stack_during_interrupt(nes);
                    decrement_s(nes);
                }
                5 => {
                    fetch_lower_pc_from_interrupt_vector(nes);
                }
                6 => {
                    set_interrupt_inhibit_flag(nes);
                    fetch_upper_pc_from_interrupt_vector(nes);
                    nes.cpu.irq_pending = false;
                    nes.cpu.interrupt_cycle = -1;
                }
                _ => unreachable!(),
            }
            nes.cpu.interrupt_cycle += 1;
        } else {
            let opcode = read_mem(nes.cpu.pc, nes);
            nes.cpu.instruction = INSTRUCTIONS[opcode as usize];
            if nes.cpu.instruction.category == Unimplemented {
                unimplemented!(
                    "Unofficial instruction {:?} not implemented!",
                    nes.cpu.instruction.name
                );
            }

            increment_pc(nes);

            // acknowledge interrupts on opcode fetch cycle for 2 cycle instructions
            if nes.cpu.instruction.does_interrupt_poll_early() {
                nes.cpu.nmi_pending = nes.cpu.nmi_edge_detector_output;
                nes.cpu.irq_pending = nes.cpu.prev_irq_signal && !nes.cpu.p_i;
            }

            end_cycle(nes);
        }
        return false;
    }

    let instr = nes.cpu.instruction;
    let func = nes.cpu.instruction.func();

    match instr.category {
        Control => control_instruction_cycles(nes, nes.cpu.instruction_cycle),
        Branch => branch_instruction_cycles(nes, nes.cpu.instruction_cycle),
        Imm => {
            fetch_immediate_from_pc(nes);
            func(nes);
            increment_pc(nes);
            nes.cpu.instruction_done = true;
        }
        Read | Write | ReadModifyWrite => {
            address_resolution_cycles(nes, nes.cpu.instruction_cycle);
            let offset_cycles = nes.cpu.instruction_cycle - instr.address_resolution_cycles();
            if offset_cycles > 0 {
                processing_cycles(nes, offset_cycles);
            }
        }
        NonMemory => {
            func(nes);
            dummy_read_from_pc_address(nes);
            nes.cpu.instruction_done = true;
        }
        _ => unreachable!(),
    }

    let instr_done = nes.cpu.instruction_done;
    if nes.cpu.instruction_done {
        end_instr(nes);
    }
    end_cycle(nes);
    instr_done
}

fn end_cycle(nes: &mut Nes) {
    if !nes.cpu.prev_nmi_signal && nes.ppu.nmi_line {
        nes.cpu.nmi_edge_detector_output = true;
    }
    nes.cpu.prev_nmi_signal = nes.ppu.nmi_line;
    nes.cpu.prev_irq_signal = nes.apu.asserting_irq() || nes.cart.asserting_irq();

    nes.cpu.cycles += 1;
    nes.cpu.instruction_cycle += 1;
}

fn end_instr(nes: &mut Nes) {
    nes.cpu.data = 0;
    nes.cpu.lower_address = 0;
    nes.cpu.upper_address = 0;
    nes.cpu.low_indirect_address = 0;
    nes.cpu.high_indirect_address = 0;
    nes.cpu.internal_carry_out = false;
    nes.cpu.branch_offset = 0;
    nes.cpu.branching = false;

    // For most instructions, interrupt polling happens on final cycle, so here
    // Two cycle instructions do the polling at the end of the first cycle instead
    // PLP also? It's not a two cycle instruction though.

    if !nes.cpu.instruction.does_interrupt_poll_early() {
        nes.cpu.nmi_pending = nes.cpu.nmi_edge_detector_output;
        nes.cpu.irq_pending = nes.cpu.prev_irq_signal && !nes.cpu.p_i;
    }

    nes.cpu.instruction_cycle = -1;
    nes.cpu.instruction_done = false;

    nes.cpu.instruction_count += 1;
}
