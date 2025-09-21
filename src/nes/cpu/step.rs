use super::cycles::{branch_instruction_cycles, processing_cycles};
use crate::nes::cpu::control_cycles::control_instruction_cycles;
use crate::nes::Nes;

use crate::nes::cpu::interrupt::{interrupt_cycles};
use crate::nes::cpu::processing_state::{InterruptType, State};
use crate::nes::cpu::simple_cycles::{fetch_opcode_from_pc_and_increment_pc, immediate_instruction_cycles, nonmemory_instruction_cycles};

pub fn step_cpu(nes: &mut Nes) -> bool {
    nes.cart.cpu_tick();
    nes.cpu.state = get_next_cpu_state(nes.cpu.state, nes);

    let instruction_finished = nes.cpu.state == State::Finished;

    if nes.cpu.state == State::Finished {
        nes.cpu.state = State::NotStarted;

        // TODO: Interrupt polling on the correct cycle

        {
            nes.cpu.clear_internal_registers();
            nes.cpu.interrupts.nmi_pending = nes.cpu.interrupts.nmi_edge_detector_output;
            nes.cpu.interrupts.irq_pending = nes.cpu.interrupts.prev_irq_signal && !nes.cpu.reg.p_i;
        }
    }

    interrupt_line_polling(nes);
    nes.cpu.debug.cycles += 1;

    instruction_finished
}

fn get_next_cpu_state(state: State, nes: &mut Nes) -> State {
    match (state, nes.cpu.instr) {

        (State::NotStarted, _) => {
            if nes.cpu.interrupts.nmi_pending {
                interrupt_cycles(State::InInterrupt(InterruptType::NMI, 0), nes)
            } else if nes.cpu.interrupts.irq_pending && !nes.cpu.reg.p_i {
                interrupt_cycles(State::InInterrupt(InterruptType::IRQ, 0), nes)
            } else {
                fetch_opcode_from_pc_and_increment_pc(nes);
                State::FetchedOpcode
            }
        }

        (state @ State::InInterrupt(_, _), _) =>
            interrupt_cycles(state, nes),

        (
            state @ (State::FetchedOpcode | State::AddrResolution(_)),
            Instruction::MemoryInstruction {
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
