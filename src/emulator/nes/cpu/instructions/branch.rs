use crate::emulator::nes::cpu::addressing::{fetch_branch_offset_from_pc, increment_pc};
use crate::nes::Nes;
use crate::nes::cpu::addressing::*;
use serde::{Deserialize, Serialize};
use crate::emulator::nes::cpu::instructions::ControlThingy;
use crate::emulator::nes::Nes;

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct BranchInstr {
    opc: BranchOpc,
    state: BranchCycle,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum BranchOpc {
    BCC,
    BCS,
    BVC,
    BVS,
    BNE,
    BEQ,
    BPL,
    BMI,
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum BranchCycle {
    FetchBranchOffset,
    OffsetLowerPc,
    FixUpperPc,
    Finished,
}

impl BranchInstr {
    pub const fn new(opc: BranchOpc) -> Self {
        Self {
            opc,
            state: BranchCycle::FetchBranchOffset,
        }
    }
    
    fn new_state(&mut self, nes: &mut Nes) {
        match self.state {
            BranchCycle::FetchBranchOffset => {
                nes.state.cpu.ireg.branch_offset = read_mem(nes.cpu.reg.pc, nes);
                increment_pc(nes);
                if self.is_branch_condition_true(nes) {
                    BranchCycle::OffsetLowerPc
                } else {
                    BranchCycle::Finished
                }
            }
        }
    }

    fn is_branch_condition_true(&self, nes: &Nes) -> bool {
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

impl ControlThingy for BranchInstr {
    fn next_cycle(&mut self, nes: &mut Nes) -> bool {
            self.state = match self.state {
                
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
                BranchCycle::Finished => BranchCycle::Finished,
            };
        }

    fn name(&self) -> &str {
        todo!()
    }
}

pub fn opcode(&self) -> String {
        format!("{:?}", self.opc)
    }

    pub fn is_finished(&self) -> bool {
        self.state == BranchCycle::Finished
    }


}
