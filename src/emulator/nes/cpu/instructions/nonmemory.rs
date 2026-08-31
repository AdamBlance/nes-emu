mod operations;

use serde::{Deserialize, Serialize};
use crate::nes::cpu::addressing::dummy_read_from_pc_address;
use operations::*;
use crate::nes::Nes;

#[derive(Default, Clone, Copy, Debug, Serialize, Deserialize)]
pub struct NonMemoryInstr {
    opc: NonMemoryOpc,
    is_finished: bool,
}

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub enum NonMemoryOpc {
    TAX, TAY, TSX, TXA, TXS, TYA,
    DEX, DEY,
    INX, INY,
    CLC, CLD, CLI, CLV,
    SEC, SED, SEI,
    ASL, LSR,
    ROL, ROR,
    #[default]
    NOP,
}

impl NonMemoryInstr {
    pub const DUMMY_INSTR: Self = Self { opc: NonMemoryOpc::NOP, is_finished: true };
    pub const fn new(opc: NonMemoryOpc) -> Self {
        Self { opc, is_finished: false }
    }
    fn opcode(&self) -> String {
        format!("{:?}", self.opc)
    }
    pub(crate) fn do_next_instruction_cycle(&mut self, nes: &mut Nes) {
        self.do_operation(nes);
        dummy_read_from_pc_address(nes);
        self.is_finished = true;
    }
    pub fn is_finished(&self) -> bool {
        self.is_finished
    }
    fn do_operation(&self, nes: &mut Nes) {
        match self.opc {
            NonMemoryOpc::TAX => transfer_a_to_x(nes),
            NonMemoryOpc::TAY => transfer_a_to_y(nes),
            NonMemoryOpc::TSX => transfer_s_to_x(nes),
            NonMemoryOpc::TXA => transfer_x_to_a(nes),
            NonMemoryOpc::TXS => transfer_x_to_s(nes),
            NonMemoryOpc::TYA => transfer_y_to_a(nes),
            NonMemoryOpc::DEX => decrement_x(nes),
            NonMemoryOpc::DEY => decrement_y(nes),
            NonMemoryOpc::INX => increment_x(nes),
            NonMemoryOpc::INY => increment_y(nes),
            NonMemoryOpc::CLC => clear_carry_flag(nes),
            NonMemoryOpc::CLD => clear_decimal_flag(nes),
            NonMemoryOpc::CLI => clear_interrupt_flag(nes),
            NonMemoryOpc::CLV => clear_overflow_flag(nes),
            NonMemoryOpc::SEC => set_carry_flag(nes),
            NonMemoryOpc::SED => set_decimal_flag(nes),
            NonMemoryOpc::SEI => set_interrupt_inhibit_flag(nes),
            NonMemoryOpc::ASL => arithmetic_shift_left_accumulator(nes),
            NonMemoryOpc::LSR => logical_shift_right_accumulator(nes),
            NonMemoryOpc::ROL => rotate_left_accumulator(nes),
            NonMemoryOpc::ROR => rotate_right_accumulator(nes),
            NonMemoryOpc::NOP => {},
        }
    }
}
