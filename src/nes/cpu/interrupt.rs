use crate::nes::cpu::addressing::{decrement_s, dummy_read_from_pc_address, fetch_lower_pc_from_interrupt_vector, fetch_upper_pc_from_interrupt_vector, push_lower_pc_to_stack, push_p_to_stack_during_interrupt, push_upper_pc_to_stack};
use crate::nes::cpu::lookup_table::ProcessingState::{Finished, InIRQ, InNMI, NotStarted};
use crate::nes::cpu::lookup_table::ProcessingState;
use crate::nes::cpu::operation_funcs::set_interrupt_inhibit_flag;
use crate::nes::Nes;

pub fn nmi_cycles(cycle: ProcessingState, nes: &mut Nes) -> ProcessingState {
    match cycle {
        NotStarted => {
            dummy_read_from_pc_address(nes);
            nes.cpu.interrupts.irq_pending = false;
            nes.cpu.interrupts.interrupt_vector = 0xFFFA;
            InNMI(0)
        }
        InNMI(0) => {
            dummy_read_from_pc_address(nes);
            InNMI(1)
        },
        InNMI(1) => {
            push_upper_pc_to_stack(nes);
            decrement_s(nes);
            InNMI(2)
        }
        InNMI(2) => {
            push_lower_pc_to_stack(nes);
            decrement_s(nes);
            InNMI(3)
        }
        InNMI(3) => {
            push_p_to_stack_during_interrupt(nes);
            decrement_s(nes);
            InNMI(4)
        }
        InNMI(4) => {
            fetch_lower_pc_from_interrupt_vector(nes);
            set_interrupt_inhibit_flag(nes);
            InNMI(5)
        }
        InNMI(5) => {
            nes.cpu.interrupts.nmi_edge_detector_output = false;
            nes.cpu.interrupts.nmi_pending = false;
            fetch_upper_pc_from_interrupt_vector(nes);
            Finished
        }
        _ => unreachable!(),
    }
}

pub fn irq_cycles(cycle: ProcessingState, nes: &mut Nes) -> ProcessingState {
    match cycle {
        NotStarted => {
            dummy_read_from_pc_address(nes);
            nes.cpu.interrupts.interrupt_vector = 0xFFFE;
            InIRQ(0)
        }
        InIRQ(0) => {
            dummy_read_from_pc_address(nes);
            InIRQ(1)
        },
        InIRQ(1) => {
            push_upper_pc_to_stack(nes);
            decrement_s(nes);
            InIRQ(2)
        }
        InIRQ(2) => {
            push_lower_pc_to_stack(nes);
            decrement_s(nes);
            InIRQ(3)
        }
        InIRQ(3) => {
            push_p_to_stack_during_interrupt(nes);
            decrement_s(nes);
            InIRQ(4)
        }
        InIRQ(4) => {
            fetch_lower_pc_from_interrupt_vector(nes);
            set_interrupt_inhibit_flag(nes);
            InIRQ(5)
        }
        InIRQ(5) => {
            set_interrupt_inhibit_flag(nes);
            nes.cpu.interrupts.irq_pending = false;
            fetch_upper_pc_from_interrupt_vector(nes);
            Finished
        }
        _ => unreachable!(),
    }
}
