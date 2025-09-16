use std::thread::sleep;
use std::time::Duration;
use crate::nes::cpu::addressing::{add_lower_address_carry_bit_to_upper_address, dummy_read_from_address, read_from_address};
use crate::nes::cpu::control_cycles::control_instruction_cycles;
use crate::nes::cpu::lookup_table::{handle_address_resolution, Category, InstructionProgress::*, InterruptType::*, Mode};
use super::cycles::{branch_instruction_cycles, processing_cycles};
use super::lookup_table::{Category::*, InstructionProgress};
use crate::nes::Nes;

// Helper structs for readability
struct StateMatch {
    category: Option<Category>,
    mode: Option<Mode>,
    state: InstructionProgress,
}
use crate::nes::cpu::interrupt::interrupt_cycles;
use crate::nes::cpu::simple_cycles::{fetch_opcode_from_pc_and_increment_pc, immediate_instruction_cycles, nonmemory_instruction_cycles};

pub fn step_cpu(nes: &mut Nes) -> bool {
    nes.cart.cpu_tick();

    let state = StateMatch {
        state: nes.cpu.proc_state.progress,
        category: try {nes.cpu.proc_state.instr?.category},
        mode: try {nes.cpu.proc_state.instr?.mode},
    };

    // println!("{:?} -> {:?}", nes.cpu.proc_state.instr, state.state);
    // sleep(Duration::from_millis(100));

    // Probably should change these functions knowing about the function by just addressing nes.cpu.blah
    // Like the functions should just change the register state.


    nes.cpu.proc_state.progress = match state {

        // If no instruction is being executed, fetch the next opcode or start handling pending interrupts
        StateMatch {
            state: Finished, ..
        } => {
            if nes.cpu.interrupts.nmi_pending {
                interrupt_cycles(NMI, Finished, nes)
            } else if nes.cpu.interrupts.irq_pending && !nes.cpu.reg.p_i {
                interrupt_cycles(IRQ, Finished, nes)
            } else {
                fetch_opcode_from_pc_and_increment_pc(nes)
            }
        }

        StateMatch {
            state: s @ InInterrupt(interrupt_type, _), ..
        } =>
            interrupt_cycles(interrupt_type, s, nes),

        // Handle instructions
        StateMatch {
            category: Some(Read | Write | ReadModifyWrite),
            mode: Some(mode),
            state: state @ (FetchedOpcode | AddrResolution(_))
        } =>
            handle_address_resolution(mode, state, nes),

        // Handle instructions
        StateMatch {
            category: Some(Write | ReadModifyWrite),
            state: PendingCarry, ..
        } => {
            dummy_read_from_address(nes);
            add_lower_address_carry_bit_to_upper_address(nes);
            FinishedAddrResolution
        }

        // Handle instructions
        StateMatch {
            category: Some(Read),
            state: PendingCarry, ..
        } => {
            if nes.cpu.ireg.carry_out {
                dummy_read_from_address(nes);
                add_lower_address_carry_bit_to_upper_address(nes);
                FinishedAddrResolution
            } else {
                read_from_address(nes);
                nes.cpu.proc_state.instr.unwrap().func()(nes);
                Finished
            }

        }

        StateMatch {
            category: Some(category @ (Read | Write | ReadModifyWrite)),
            state: s @ (FinishedAddrResolution | Processing(_)), ..
        } =>
            processing_cycles(category, s, nes),
//         Absolute | ZeroPage | ZeroPageX | ZeroPageY | IndirectX | Immediate => 1,
        StateMatch {
            category: Some(category),
            state: s @ (FetchedOpcode | Processing(_)), ..
        } => match category {
            Branch => branch_instruction_cycles(s, nes),
            Control => control_instruction_cycles(s, nes),
            Imm => immediate_instruction_cycles(s, nes),
            NonMemory => nonmemory_instruction_cycles(s, nes),
            _ => unreachable!(),
        }
        _ => unreachable!(),
    };

    if nes.cpu.proc_state.progress == Finished {
        // println!("{:?}", nes.cpu.proc_state.instr.unwrap().name);
        // if !nes.cpu.proc_state.instr.unwrap().does_interrupt_poll_early() {
         {
             nes.cpu.clear_internal_registers();
             nes.cpu.proc_state.instr = None;
            nes.cpu.interrupts.nmi_pending = nes.cpu.interrupts.nmi_edge_detector_output;
            nes.cpu.interrupts.irq_pending = nes.cpu.interrupts.prev_irq_signal && !nes.cpu.reg.p_i;
        }
    }

    end_cycle(nes);




    nes.cpu.proc_state.progress == Finished
}

fn end_cycle(nes: &mut Nes) {
    if !nes.cpu.interrupts.prev_nmi_signal && nes.ppu.nmi_line {
        nes.cpu.interrupts.nmi_edge_detector_output = true;
    }
    nes.cpu.interrupts.prev_nmi_signal = nes.ppu.nmi_line;
    nes.cpu.interrupts.prev_irq_signal = nes.apu.asserting_irq() || nes.cart.asserting_irq();

    nes.cpu.debug.cycles += 1;
}
