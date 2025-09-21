use crate::nes::cpu::instr::instruction::{Instruction, IsInstructionFinished};
use crate::nes::cpu::instr::addressing::{decrement_s, dummy_read_from_pc_address, fetch_lower_pc_from_interrupt_vector, fetch_upper_pc_from_interrupt_vector, increment_pc, increment_s, pull_a_from_stack, pull_lower_pc_from_stack, pull_p_from_stack, pull_upper_pc_from_stack, push_a_to_stack, push_lower_pc_to_stack, push_p_to_stack_during_break_or_php, push_upper_pc_to_stack};
use crate::nes::cpu::operation_funcs::{set_interrupt_inhibit_flag, update_p_nz};
use crate::nes::Nes;

pub struct ControlInstr {
    opc: ControlOpc,
    state: ControlCycle,
}

#[derive(Debug)]
pub enum ControlOpc {
    BRK,
    PHP,
    PLP,
    PLA,
    PHA,
    RTI,
    RTS,
}

#[derive(Copy, Clone, Debug, PartialEq)]
enum ControlCycle {
    Cycle(u8),
    Finished
}

impl ControlInstr {
    pub const fn new(opc: ControlOpc) -> Self {
        Self {
            opc,
            state: ControlCycle::Cycle(0),
        }
    }
}
impl Instruction for ControlInstr {
    fn opcode(&self) -> String {
        format!("{:?}", self.opc)
    }
    fn do_next_instruction_cycle(&mut self, nes: &mut Nes) -> IsInstructionFinished {
        self.state = match self.opc {
            ControlOpc::BRK => break_cycles(self.state, nes),
            ControlOpc::RTI => return_from_interrupt_cycles(self.state, nes),
            ControlOpc::RTS => return_from_subroutine_cycles(self.state, nes),
            ControlOpc::PHA => push_accumulator_to_stack_cycles(self.state, nes),
            ControlOpc::PHP => push_status_register_to_stack_cycles(self.state, nes),
            ControlOpc::PLA => pull_accumulator_from_stack_cycles(self.state, nes),
            ControlOpc::PLP => pull_status_register_from_stack_cycles(self.state, nes),
        };
        self.state == ControlCycle::Finished
    }
}

fn break_cycles(state: ControlCycle, nes: &mut Nes) -> ControlCycle {
    match state {
        ControlCycle::Cycle(0) => {
            dummy_read_from_pc_address(nes);
            increment_pc(nes);
            nes.cpu.interrupts.interrupt_vector = 0xFFFE;
            ControlCycle::Cycle(1)
        }
        ControlCycle::Cycle(1) => {
            push_upper_pc_to_stack(nes);
            decrement_s(nes);
            ControlCycle::Cycle(2)
        }
        ControlCycle::Cycle(2) => {
            push_lower_pc_to_stack(nes);
            decrement_s(nes);
            ControlCycle::Cycle(3)
        }
        ControlCycle::Cycle(3) => {
            push_p_to_stack_during_break_or_php(nes);
            decrement_s(nes);
            ControlCycle::Cycle(4)
        }
        ControlCycle::Cycle(4) => {
            fetch_lower_pc_from_interrupt_vector(nes);
            set_interrupt_inhibit_flag(nes);
            ControlCycle::Cycle(5)
        }
        ControlCycle::Cycle(5) => {
            fetch_upper_pc_from_interrupt_vector(nes);
            ControlCycle::Finished
        }
        _ => state,
    }
}

fn return_from_interrupt_cycles(state: ControlCycle, nes: &mut Nes) -> ControlCycle {
    match state {
        ControlCycle::Cycle(0) => {
            dummy_read_from_pc_address(nes);
            ControlCycle::Cycle(1)
        }
        ControlCycle::Cycle(1) => {
            increment_s(nes);
            ControlCycle::Cycle(2)
        }
        ControlCycle::Cycle(2) => {
            pull_p_from_stack(nes);
            increment_s(nes);
            ControlCycle::Cycle(3)
        }
        ControlCycle::Cycle(3) => {
            pull_lower_pc_from_stack(nes);
            increment_s(nes);
            ControlCycle::Cycle(4)
        }
        ControlCycle::Cycle(4) => {
            pull_upper_pc_from_stack(nes);
            ControlCycle::Finished
        }
        _ => state,
    }
}

fn return_from_subroutine_cycles(state: ControlCycle, nes: &mut Nes) -> ControlCycle {
    match state {
        ControlCycle::Cycle(0) => {
            dummy_read_from_pc_address(nes);
            ControlCycle::Cycle(1)
        }
        ControlCycle::Cycle(1) => {
            increment_s(nes);
            ControlCycle::Cycle(2)
        }
        ControlCycle::Cycle(2) => {
            pull_lower_pc_from_stack(nes);
            increment_s(nes);
            ControlCycle::Cycle(3)
        }
        ControlCycle::Cycle(3) => {
            pull_upper_pc_from_stack(nes);
            ControlCycle::Cycle(4)
        }
        ControlCycle::Cycle(4) => {
            increment_pc(nes);
            ControlCycle::Finished
        }
        _ => state,
    }
}

fn push_accumulator_to_stack_cycles(state: ControlCycle, nes: &mut Nes) -> ControlCycle {
    match state {
        ControlCycle::Cycle(0) => {
            dummy_read_from_pc_address(nes);
            ControlCycle::Cycle(1)
        }
        ControlCycle::Cycle(1) => {
            push_a_to_stack(nes);
            decrement_s(nes);
            ControlCycle::Finished
        }
        _ => state,
    }
}

fn push_status_register_to_stack_cycles(state: ControlCycle, nes: &mut Nes) -> ControlCycle {
    match state {
        ControlCycle::Cycle(0) => {
            dummy_read_from_pc_address(nes);
            ControlCycle::Cycle(1)
        }
        ControlCycle::Cycle(1) => {
            push_p_to_stack_during_break_or_php(nes);
            decrement_s(nes);
            ControlCycle::Finished
        }
        _ => state,
    }
}

fn pull_accumulator_from_stack_cycles(state: ControlCycle, nes: &mut Nes) -> ControlCycle {
    match state {
        ControlCycle::Cycle(0) => {
            dummy_read_from_pc_address(nes);
            ControlCycle::Cycle(1)
        }
        ControlCycle::Cycle(1) => {
            increment_s(nes);
            ControlCycle::Cycle(2)
        }
        ControlCycle::Cycle(2) => {
            pull_a_from_stack(nes);
            update_p_nz(nes, nes.cpu.reg.a);
            ControlCycle::Finished
        }
        _ => state,
    }
}

fn pull_status_register_from_stack_cycles(state: ControlCycle, nes: &mut Nes) -> ControlCycle {
    match state {
        ControlCycle::Cycle(0) => {
            dummy_read_from_pc_address(nes);
            ControlCycle::Cycle(1)
        }
        ControlCycle::Cycle(1) => {
            increment_s(nes);
            ControlCycle::Cycle(2)
        }
        ControlCycle::Cycle(2) => {
            pull_p_from_stack(nes);
            ControlCycle::Finished
        }
        _ => state,
    }
}
