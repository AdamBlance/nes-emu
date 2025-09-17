use crate::nes::cpu::addressing::*;
use crate::nes::cpu::lookup_table::ProcessingState::{FetchedOpcode, Finished, FinishedAddrResolution, PendingCarry, RmwWrites, SimpleCycle};
use crate::nes::cpu::lookup_table::{handle_upper_address_overflow, Category, Category::*, Mode::*, Name::*, ProcessingState};
use crate::nes::Nes;


pub fn branch_instruction_cycles(cycle: ProcessingState, nes: &mut Nes) -> ProcessingState {
    match cycle {
        FetchedOpcode => {
            fetch_branch_offset_from_pc(nes);
            increment_pc(nes);
            let taking_branch = match nes.cpu.instr.name {
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
                SimpleCycle(0)
            } else {
                Finished
            }
        }
        SimpleCycle(0) => {
            add_branch_offset_to_lower_pc_and_set_carry(nes);
            if nes.cpu.ireg.carry_out {
                SimpleCycle(1)
            } else {
                Finished
            }
        }
        SimpleCycle(1) => {
            fix_upper_pc_after_page_crossing_branch(nes);
            Finished
        }
        _ => unreachable!(),
    }
}


pub fn processing_cycles(category: Category, cycle: ProcessingState, nes: &mut Nes) -> ProcessingState {
    match cycle {
        PendingCarry => handle_upper_address_overflow(category, nes),
        FinishedAddrResolution => match category {
            Read => {
                read_from_address(nes);
                nes.cpu.instr.func()(nes);
                Finished
            }
            Write => {
                nes.cpu.instr.func()(nes);
                write_to_address(nes);
                Finished
            }
            ReadModifyWrite => {
                read_from_address(nes);
                RmwWrites(0)
            }
            _ => unreachable!()
        }
        RmwWrites(0) => {
            dummy_write_to_address(nes);
            nes.cpu.instr.func()(nes);
            RmwWrites(1)
        }
        RmwWrites(1) => {
            write_to_address(nes);
            Finished
        }
        _ => unreachable!(),
    }
}

