use crate::nes::cpu::instr::addressing::{add_branch_offset_to_lower_pc_and_set_carry, fetch_branch_offset_from_pc, fix_upper_pc_after_page_crossing_branch, increment_pc};
use crate::nes::cpu::instr::{Instruction, IsInstructionFinished};
use crate::nes::Nes;

pub struct BranchInstr {
    opc: BranchOpc,
    state: BranchCycle,
}

#[derive(Debug)]
pub enum BranchOpc {
    BCC, BCS, BVC, BVS, BNE, BEQ, BPL, BMI,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum BranchCycle {
    FetchBranchOffset,
    OffsetLowerPc,
    FixUpperPc,
    Finished
}

impl BranchInstr {
    pub const fn new(opc: BranchOpc) -> Self {
        Self {
            opc,
            state: BranchCycle::FetchBranchOffset,
        }
    }
    pub fn is_branch_condition_true(&self, nes: &Nes) -> bool {
        match self.opc {
            BranchOpc::BCC => !nes.cpu.reg.p_c,
            BranchOpc::BCS => nes.cpu.reg.p_c,
            BranchOpc::BVC => !nes.cpu.reg.p_v,
            BranchOpc::BVS => nes.cpu.reg.p_v,
            BranchOpc::BNE => !nes.cpu.reg.p_z,
            BranchOpc::BEQ => nes.cpu.reg.p_z,
            BranchOpc::BPL => !nes.cpu.reg.p_n,
            BranchOpc::BMI => nes.cpu.reg.p_n,
        }
    }
}

impl Instruction for BranchInstr {
    fn opcode(&self) -> String {
        format!("{:?}", self.opc)
    }
    fn do_next_instruction_cycle(&mut self, nes: &mut Nes) -> IsInstructionFinished {
        self.state = match self.state {
            BranchCycle::FetchBranchOffset => {
                fetch_branch_offset_from_pc(nes);
                increment_pc(nes);
                if self.is_branch_condition_true(nes) {
                    BranchCycle::OffsetLowerPc
                } else {
                    BranchCycle::Finished
                }
            }
            BranchCycle::OffsetLowerPc => {
                add_branch_offset_to_lower_pc_and_set_carry(nes);
                if nes.cpu.ireg.carry_out {
                    BranchCycle::FixUpperPc
                } else {
                    BranchCycle::Finished
                }
            }
            BranchCycle::FixUpperPc => {
                fix_upper_pc_after_page_crossing_branch(nes);
                BranchCycle::Finished
            }
            state => state,
        };
        self.state == BranchCycle::Finished
    }
}




