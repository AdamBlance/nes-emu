use serde::{Deserialize, Serialize};
use crate::nes::cpu::instr::{Instruction, IsInstructionFinished};
use crate::nes::cpu::instr::addressing::{decrement_s, dummy_read_from_pc_address, fetch_lower_pc_from_interrupt_vector, fetch_upper_pc_from_interrupt_vector, push_lower_pc_to_stack, push_p_to_stack_during_interrupt, push_upper_pc_to_stack};
use crate::nes::cpu::instr::interrupt::InterruptState::{Finished, InterruptCycle};
use crate::nes::cpu::operation_funcs::set_interrupt_inhibit_flag;
use crate::nes::Nes;

pub struct Interrupt {
    interrupt_type: InterruptType,
    state: InterruptState
}

#[derive(PartialEq, Clone, Copy)]
pub enum InterruptType {
    NMI,
    IRQ,
}

#[derive(PartialEq)]
enum InterruptState {
    InterruptCycle(InterruptType, u8),
    Finished
}

impl Interrupt {
    pub fn new(interrupt_type: InterruptType) -> Self {
        Self {
            interrupt_type,
            state: InterruptCycle(interrupt_type, 0),
        }
    }
}

impl Instruction for Interrupt {
    fn opcode(&self) -> String {
        "not an instruction".to_string()
    }

    fn do_next_instruction_cycle(&mut self, nes: &mut Nes) -> IsInstructionFinished {
        self.state = match self.state {
            InterruptCycle(InterruptType::NMI, 0) => {
                dummy_read_from_pc_address(nes);
                nes.cpu.interrupts.irq_pending = false;
                nes.cpu.interrupts.interrupt_vector = 0xFFFA;
                InterruptCycle(InterruptType::NMI, 1)
            },
            InterruptCycle(InterruptType::IRQ, 0) => {
                dummy_read_from_pc_address(nes);
                nes.cpu.interrupts.interrupt_vector = 0xFFFE;
                InterruptCycle(InterruptType::IRQ, 1)
            },
            InterruptCycle(interrupt_type, 1) => {
                dummy_read_from_pc_address(nes);
                InterruptCycle(interrupt_type, 2)
            },
            InterruptCycle(interrupt_type, 2) => {
                push_upper_pc_to_stack(nes);
                decrement_s(nes);
                InterruptCycle(interrupt_type, 3)
            }
            InterruptCycle(interrupt_type, 3) => {
                push_lower_pc_to_stack(nes);
                decrement_s(nes);
                InterruptCycle(interrupt_type, 4)
            }
            InterruptCycle(interrupt_type, 4) => {
                push_p_to_stack_during_interrupt(nes);
                decrement_s(nes);
                InterruptCycle(interrupt_type, 5)
            }
            InterruptCycle(InterruptType::NMI, 5) => {
                fetch_lower_pc_from_interrupt_vector(nes);
                set_interrupt_inhibit_flag(nes);
                InterruptCycle(InterruptType::NMI, 6)
            }
            InterruptCycle(InterruptType::IRQ, 5) => {
                fetch_lower_pc_from_interrupt_vector(nes);
                InterruptCycle(InterruptType::IRQ, 6)
            }
            InterruptCycle(InterruptType::NMI, 6) => {
                fetch_upper_pc_from_interrupt_vector(nes);
                nes.cpu.interrupts.nmi_edge_detector_output = false;
                nes.cpu.interrupts.nmi_pending = false;
                Finished
            }
            InterruptCycle(InterruptType::IRQ, 6) => {
                set_interrupt_inhibit_flag(nes);
                fetch_upper_pc_from_interrupt_vector(nes);
                nes.cpu.interrupts.irq_pending = false;
                Finished
            }
            state => state,
        };
        self.state == Finished
    }
}