use crate::nes::cpu::instr::addressing::dummy_read_from_pc_address;
use crate::nes::cpu::operation_funcs::{arithmetic_shift_left_accumulator, clear_carry_flag, clear_decimal_flag, clear_interrupt_flag, clear_overflow_flag, decrement_x, decrement_y, increment_x, increment_y, logical_shift_right_accumulator, no_op, rotate_left_accumulator, rotate_right_accumulator, set_carry_flag, set_decimal_flag, set_interrupt_inhibit_flag, transfer_a_to_x, transfer_a_to_y, transfer_s_to_x, transfer_x_to_a, transfer_x_to_s, transfer_y_to_a};
use crate::nes::Nes;

#[derive(Default)]
pub struct NonMemoryInstr {
    opc: NonMemoryOpc,
}

#[derive(Debug, Default)]
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
    pub const fn new(opc: NonMemoryOpc) -> Self {
        Self { opc }
    }
    fn operation(&self) -> fn(&mut Nes) {
        match self.opc {
            NonMemoryOpc::TAX => transfer_a_to_x,
            NonMemoryOpc::TAY => transfer_a_to_y,
            NonMemoryOpc::TSX => transfer_s_to_x,
            NonMemoryOpc::TXA => transfer_x_to_a,
            NonMemoryOpc::TXS => transfer_x_to_s,
            NonMemoryOpc::TYA => transfer_y_to_a,
            NonMemoryOpc::DEX => decrement_x,
            NonMemoryOpc::DEY => decrement_y,
            NonMemoryOpc::INX => increment_x,
            NonMemoryOpc::INY => increment_y,
            NonMemoryOpc::CLC => clear_carry_flag,
            NonMemoryOpc::CLD => clear_decimal_flag,
            NonMemoryOpc::CLI => clear_interrupt_flag,
            NonMemoryOpc::CLV => clear_overflow_flag,
            NonMemoryOpc::SEC => set_carry_flag,
            NonMemoryOpc::SED => set_decimal_flag,
            NonMemoryOpc::SEI => set_interrupt_inhibit_flag,
            NonMemoryOpc::ASL => arithmetic_shift_left_accumulator,
            NonMemoryOpc::LSR => logical_shift_right_accumulator,
            NonMemoryOpc::ROL => rotate_left_accumulator,
            NonMemoryOpc::ROR => rotate_right_accumulator,
            NonMemoryOpc::NOP => no_op
        }
    }
}

impl Instruction for NonMemoryInstr {
    fn opcode(&self) -> String {
        format!("{:?}", self.opc)
    }

    fn do_next_instruction_cycle(&mut self, nes: &mut Nes) -> IsInstructionFinished {
        nes.cpu.instr.func()(nes);
        dummy_read_from_pc_address(nes);
        true
    }
}