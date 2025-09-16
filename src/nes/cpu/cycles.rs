use crate::nes::cpu::addressing::*;
use crate::nes::cpu::lookup_table::{Category, Category::*, Instruction, InstructionProgress, Mode, Mode::*, Name::*};
use crate::nes::cpu::lookup_table::InstructionProgress::{AddrResolution, FetchedOpcode, FinishedAddrResolution, NotStarted, Processing};
use crate::nes::cpu::operation_funcs::{set_interrupt_inhibit_flag, update_p_nz};
use crate::nes::Nes;


pub fn branch_instruction_cycles(cycle: InstructionProgress, nes: &mut Nes) -> InstructionProgress {
    match cycle {
        FetchedOpcode => {
            fetch_branch_offset_from_pc(nes);
            increment_pc(nes);
            let taking_branch = match nes.cpu.proc_state.instr.unwrap().name {
                BCC => !nes.cpu.reg.p_c,
                BCS => nes.cpu.reg.p_c,
                BVC => !nes.cpu.reg.p_v,
                BVS => nes.cpu.reg.p_v,
                BNE => !nes.cpu.reg.p_z,
                BEQ => nes.cpu.reg.p_z,
                BPL => !nes.cpu.reg.p_n,
                BMI => nes.cpu.reg.p_n,
                _ => unreachable!(),
            };
            if taking_branch {
                Processing(0)
            } else {
                NotStarted
            }
        }
        Processing(0) => {
            add_branch_offset_to_lower_pc_and_set_carry(nes);
            if nes.cpu.ireg.carry_out {
                Processing(1)
            } else {
                NotStarted
            }
        }
        Processing(1) => {
            fix_upper_pc_after_page_crossing_branch(nes);
            NotStarted
        }
        _ => unreachable!(),
    }
}

pub fn processing_cycles(category: Category, cycle: InstructionProgress, nes: &mut Nes) -> InstructionProgress {
    match category {
        Read => match cycle {
            FinishedAddrResolution => {
                read_from_address(nes);
                add_lower_address_carry_bit_to_upper_address(nes);
                if nes.cpu.ireg.carry_out {
                    Processing(0)
                } else {
                    nes.cpu.proc_state.instr.unwrap().func()(nes);
                    NotStarted
                }
            }
            Processing(0) => {
                read_from_address(nes);
                nes.cpu.proc_state.instr.unwrap().func()(nes);
                NotStarted
            }
            _ => unreachable!(),

        },
        Write => match cycle {
            FinishedAddrResolution => {
                dummy_read_from_address(nes);
                add_lower_address_carry_bit_to_upper_address(nes);
                Processing(0)
            }
            Processing(0) => {
                nes.cpu.proc_state.instr.unwrap().func()(nes);
                write_to_address(nes);
                NotStarted

            }
            _ => unreachable!(),
        },
        ReadModifyWrite => match cycle {
            FinishedAddrResolution => {
                dummy_read_from_address(nes);
                add_lower_address_carry_bit_to_upper_address(nes);
                Processing(0)
            }
            Processing(0) => {
                read_from_address(nes);
                Processing(1)
            }
            Processing(1) => {
                dummy_write_to_address(nes);
                nes.cpu.proc_state.instr.unwrap().func()(nes);
                Processing(2)
            }
            Processing(2) => {
                write_to_address(nes);
                NotStarted

            }
            _ => unreachable!(),
        },
        _ => unreachable!(),
    }
}
