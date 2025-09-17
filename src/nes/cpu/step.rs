use super::cycles::{branch_instruction_cycles, processing_cycles};
use super::lookup_table::{Category::*, ProcessingState};
use crate::nes::cpu::control_cycles::control_instruction_cycles;
use crate::nes::cpu::lookup_table::{handle_address_resolution, Instruction, InterruptType::*, ProcessingState::*};
use crate::nes::Nes;

use crate::nes::cpu::interrupt::{irq_cycles, nmi_cycles};
use crate::nes::cpu::simple_cycles::{fetch_opcode_from_pc_and_increment_pc, immediate_instruction_cycles, nonmemory_instruction_cycles};

pub fn step_cpu(nes: &mut Nes) -> bool {
    nes.cart.cpu_tick();
    nes.cpu.state = get_next_cpu_state(nes.cpu.state, nes);

    if nes.cpu.state == Finished {
        nes.cpu.state = NotStarted;

        // TODO: Interrupt polling on the correct cycle

        {
            nes.cpu.clear_internal_registers();
            nes.cpu.interrupts.nmi_pending = nes.cpu.interrupts.nmi_edge_detector_output;
            nes.cpu.interrupts.irq_pending = nes.cpu.interrupts.prev_irq_signal && !nes.cpu.reg.p_i;
        }
    }

    interrupt_line_polling(nes);
    nes.cpu.debug.cycles += 1;

    nes.cpu.state == Finished
}

fn get_next_cpu_state(state: ProcessingState, nes: &mut Nes) -> ProcessingState {
    match (state, nes.cpu.instr) {

        (NotStarted, _) => {
            if nes.cpu.interrupts.nmi_pending {
                nmi_cycles(NotStarted, nes)
            } else if nes.cpu.interrupts.irq_pending && !nes.cpu.reg.p_i {
                irq_cycles(NotStarted, nes)
            } else {
                fetch_opcode_from_pc_and_increment_pc(nes)
            }
        }

        (InNMI(cycle), _) =>
            nmi_cycles(InNMI(cycle), nes),
        (InIRQ(cycle), _) =>
            irq_cycles(InIRQ(cycle), nes),

        (
            state @ (FetchedOpcode | AddrResolution(_)),
            Instruction {
                category: Read | Write | ReadModifyWrite,
                mode, ..
            }
        ) =>
            handle_address_resolution(mode, state, nes),

        (
            state @ (FinishedAddrResolution | PendingCarry | RmwWrites(_)),
            Instruction {
                category: cat @ (Read | Write | ReadModifyWrite), ..
            }
        ) =>
            processing_cycles(cat, state, nes),

        (
            s @ (FetchedOpcode | SimpleCycle(_)),
            Instruction {
                category, ..
            }
        ) => match category {
            Branch => branch_instruction_cycles(s, nes),
            Control => control_instruction_cycles(s, nes),
            Imm => immediate_instruction_cycles(s, nes),
            NonMemory => nonmemory_instruction_cycles(s, nes),
            _ => unreachable!(),
        }
        _ => unreachable!(),
    }
}

fn interrupt_line_polling(nes: &mut Nes) {
    if !nes.cpu.interrupts.prev_nmi_signal && nes.ppu.nmi_line {
        nes.cpu.interrupts.nmi_edge_detector_output = true;
    }
    nes.cpu.interrupts.prev_nmi_signal = nes.ppu.nmi_line;
    nes.cpu.interrupts.prev_irq_signal = nes.apu.asserting_irq() || nes.cart.asserting_irq();
}
