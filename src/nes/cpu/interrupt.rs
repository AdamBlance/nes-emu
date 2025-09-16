use crate::nes::cpu::addressing::{decrement_s, dummy_read_from_pc_address, fetch_lower_pc_from_interrupt_vector, fetch_upper_pc_from_interrupt_vector, push_lower_pc_to_stack, push_p_to_stack_during_interrupt, push_upper_pc_to_stack};
use crate::nes::cpu::lookup_table::{InstructionProgress, InterruptType};
use crate::nes::cpu::lookup_table::InstructionProgress::{InInterrupt, Finished};
use crate::nes::cpu::lookup_table::InterruptType::{IRQ, NMI};
use crate::nes::cpu::operation_funcs::set_interrupt_inhibit_flag;
use crate::nes::Nes;

pub fn interrupt_cycles(i_type: InterruptType, cycle: InstructionProgress, nes: &mut Nes) -> InstructionProgress {
    match cycle {
        Finished => {
            dummy_read_from_pc_address(nes);
            match i_type {
                NMI => {
                    nes.cpu.interrupts.irq_pending = false;
                    nes.cpu.interrupts.interrupt_vector = 0xFFFA;
                }
                IRQ => {
                    nes.cpu.interrupts.interrupt_vector = 0xFFFE;
                }
            }
            InInterrupt(i_type, 1)
        }
        InInterrupt(i_type, 1) => {
            dummy_read_from_pc_address(nes);
            InInterrupt(i_type, 2)
        },
        InInterrupt(i_type, 2) => {
            push_upper_pc_to_stack(nes);
            decrement_s(nes);
            InInterrupt(i_type, 3)
        }
        InInterrupt(i_type, 3) => {
            push_lower_pc_to_stack(nes);
            decrement_s(nes);
            InInterrupt(i_type, 4)
        }
        InInterrupt(i_type, 4) => {
            push_p_to_stack_during_interrupt(nes);
            decrement_s(nes);
            InInterrupt(i_type, 5)
        }
        InInterrupt(i_type, 5) => {
            fetch_lower_pc_from_interrupt_vector(nes);
            set_interrupt_inhibit_flag(nes);
            InInterrupt(i_type, 6)
        }
        InInterrupt(i_type, 6) => {
            match i_type {
                NMI => {
                    nes.cpu.interrupts.nmi_edge_detector_output = false;
                    nes.cpu.interrupts.nmi_pending = false;
                }
                IRQ => {
                    set_interrupt_inhibit_flag(nes);
                    nes.cpu.interrupts.irq_pending = false;
                }
            }
            fetch_upper_pc_from_interrupt_vector(nes);
            Finished
        }
        _ => unreachable!(),
    }
}