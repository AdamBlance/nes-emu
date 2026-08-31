use crate::nes::cpu::addressing::increment_pc;
use crate::nes::cpu::instructions::Instr;
use crate::nes::cpu::instructions::interrupts::InterruptType;
use crate::nes::mem::read_mem;
use crate::nes::Nes;

pub fn step_cpu(instr: &mut Instr, nes: &mut Nes) -> bool {
    nes.cart.cpu_tick();

    if instr.is_finished() {
        *instr = if nes.cpu.interrupts.nmi_pending {
            Instr::new_interrupt(InterruptType::NMI)
        } else if nes.cpu.interrupts.irq_pending && !nes.cpu.reg.p_i {
            Instr::new_interrupt(InterruptType::IRQ)
        } else {
            let new_opcode = read_mem(nes.cpu.reg.pc, nes);
            increment_pc(nes);
            Instr::from_opcode(new_opcode)
        };
    } else {
        instr.do_next_cycle(nes);
    }



    if instr.is_finished() {
        // TODO: Interrupt polling on the correct cycle

        {
            nes.cpu.clear_internal_registers();
            nes.cpu.interrupts.nmi_pending = nes.cpu.interrupts.nmi_edge_detector_output;
            nes.cpu.interrupts.irq_pending = nes.cpu.interrupts.prev_irq_signal && !nes.cpu.reg.p_i;
        }
    }

    interrupt_line_polling(nes);
    nes.cpu.debug.cycles += 1;

    instr.is_finished()

}

fn interrupt_line_polling(nes: &mut Nes) {
    if !nes.cpu.interrupts.prev_nmi_signal && nes.ppu.nmi_line {
        nes.cpu.interrupts.nmi_edge_detector_output = true;
    }
    nes.cpu.interrupts.prev_nmi_signal = nes.ppu.nmi_line;
    nes.cpu.interrupts.prev_irq_signal = nes.apu.asserting_irq() || nes.cart.asserting_irq();
}
