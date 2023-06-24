use crate::nes::Nes;
use super::addressing::*;
use super::cycles::{control_instruction_cycles, address_resolution_cycles, branch_instruction_cycles, processing_cycles};
use crate::mem::{read_mem, read_mem_safe};
use super::lookup_table::{
    INSTRUCTIONS,
    Mode::*,
    Category::*,
    Name::*,
};
use super::operation_funcs::{set_interrupt_inhibit_flag};
use crate::util::concat_u8;
use std::io::Write;



pub fn step_cpu(nes: &mut Nes) {

    nes.cart.cpu_tick();

    if nes.cpu.instruction_cycle == 0 {

        nes.cpu.trace_opc_addr = nes.cpu.pc;
        nes.cpu.trace_a = nes.cpu.a;
        nes.cpu.trace_x = nes.cpu.x;
        nes.cpu.trace_y = nes.cpu.y;
        nes.cpu.trace_p = nes.cpu.get_p();
        nes.cpu.trace_s = nes.cpu.s;
        nes.cpu.trace_initial_cycle = nes.cpu.cycles;
        nes.cpu.trace_initial_ppu_scanline = nes.ppu.scanline;
        nes.cpu.trace_initial_ppu_scanline_cycle = nes.ppu.scanline_cycle;
        nes.cpu.trace_vblank = nes.ppu.in_vblank;

        let op1 = read_mem_safe(nes.cpu.pc.wrapping_add(1), nes);
        let op2 = read_mem_safe(nes.cpu.pc.wrapping_add(2), nes);
        nes.cpu.trace_read_absolute_addr_first_cycle = read_mem_safe(concat_u8(op2, op1), nes);


        if nes.cpu.nmi_pending {
            match nes.cpu.interrupt_cycle {
                0 => {dummy_read_from_pc_address(nes); nes.cpu.irq_pending = false; nes.cpu.interrupt_vector = 0xFFFA;}
                1 => dummy_read_from_pc_address(nes),
                2 => {push_upper_pc_to_stack(nes); decrement_s(nes);}
                3 => {push_lower_pc_to_stack(nes); decrement_s(nes);}
                4 => {push_p_to_stack_during_interrupt(nes); decrement_s(nes);}
                5 => {fetch_lower_pc_from_interrupt_vector(nes); set_interrupt_inhibit_flag(nes)}
                6 => {
                    fetch_upper_pc_from_interrupt_vector(nes);
                    nes.cpu.nmi_edge_detector_output = false; 
                    nes.cpu.nmi_pending = false;
                    nes.cpu.interrupt_cycle = -1;
                }
                _ => unreachable!(),
            }
            nes.cpu.interrupt_cycle += 1;
        }

        // Ignore IRQ until the interrupt inhibit status flag is cleared
        else if nes.cpu.irq_pending && !nes.cpu.p_i {
            match nes.cpu.interrupt_cycle {
                0 => {dummy_read_from_pc_address(nes); nes.cpu.interrupt_vector = 0xFFFE;}
                1 => dummy_read_from_pc_address(nes),
                2 => {push_upper_pc_to_stack(nes); decrement_s(nes);}
                3 => {push_lower_pc_to_stack(nes); decrement_s(nes);}
                4 => {push_p_to_stack_during_interrupt(nes); decrement_s(nes);}
                5 => {fetch_lower_pc_from_interrupt_vector(nes);}
                6 => {
                    set_interrupt_inhibit_flag(nes);
                    fetch_upper_pc_from_interrupt_vector(nes);
                    nes.cpu.irq_pending = false;
                    nes.cpu.interrupt_cycle = -1;
                }
                _ => unreachable!(),
            }
            nes.cpu.interrupt_cycle += 1;
        }
        
        else {
            let opcode = read_mem(nes.cpu.pc, nes);
            nes.cpu.trace_opc = opcode;
            nes.cpu.instruction = INSTRUCTIONS[opcode as usize];
            if nes.cpu.instruction.category == Unimplemented {
                unimplemented!("Unofficial instruction {:?} not implemented!", nes.cpu.instruction.name);
            }

            increment_pc(nes);
        
            // acknowledge interrupts on opcode fetch cycle for 2 cycle instructions
            if nes.cpu.instruction.does_interrupt_poll_early() {
                nes.cpu.nmi_pending = nes.cpu.nmi_edge_detector_output;
                nes.cpu.irq_pending = nes.cpu.prev_irq_signal && !nes.cpu.p_i;
            }

            end_cycle(nes);
        }
        return
    }


    let instr = nes.cpu.instruction;
    let cat = nes.cpu.instruction.category; 
    let func = nes.cpu.instruction.operation;
    

    match instr.category {
        Control => control_instruction_cycles(nes, nes.cpu.instruction_cycle),
        Branch => branch_instruction_cycles(nes, nes.cpu.instruction_cycle),
        Imm => {fetch_immediate_from_pc(nes); func(nes); increment_pc(nes); nes.cpu.instruction_done = true;}
        Read | Write | ReadModifyWrite => {
            address_resolution_cycles(nes, nes.cpu.instruction_cycle);
            let offset_cycles = nes.cpu.instruction_cycle - instr.address_resolution_cycles();
            if offset_cycles > 0 {
                processing_cycles(nes, offset_cycles);
            }
        }
        NonMemory => {func(nes); dummy_read_from_pc_address(nes); nes.cpu.instruction_done = true;}
        _ => unreachable!()
    }

    if nes.cpu.instruction_done {
        end_instr(nes);
    }
    end_cycle(nes);
    
}

fn end_cycle(nes: &mut Nes) {

    if nes.cpu.prev_nmi_signal == false && nes.ppu.nmi_line == true {
        nes.cpu.nmi_edge_detector_output = true;
    }
    nes.cpu.prev_nmi_signal = nes.ppu.nmi_line;
    nes.cpu.prev_irq_signal = nes.apu.asserting_irq() || nes.cart.asserting_irq();

    nes.cpu.cycles += 1;
    nes.cpu.instruction_cycle += 1;
    
}

fn end_instr(nes: &mut Nes) {
    // writeln!(nes.logfile, "{}", create_log_line_mesen(nes)).unwrap();

    nes.cpu.data = 0;
    nes.cpu.lower_address = 0;
    nes.cpu.upper_address = 0;
    nes.cpu.low_indirect_address = 0;
    nes.cpu.high_indirect_address = 0;
    nes.cpu.internal_carry_out = false;
    nes.cpu.branch_offset = 0;
    nes.cpu.branching = false;

    // For most instructions, interrupt polling happens on final cycle, so here
    // Two cycle instructions do the polling at the end of the first cycle instead
    // PLP also? It's not a two cycle instruction though.

    if !nes.cpu.instruction.does_interrupt_poll_early() {
        nes.cpu.nmi_pending = nes.cpu.nmi_edge_detector_output;
        nes.cpu.irq_pending = nes.cpu.prev_irq_signal && !nes.cpu.p_i;
    }
    
    nes.cpu.instruction_cycle = -1;
    nes.cpu.instruction_done = false;

    nes.cpu.instruction_count += 1;

}

fn create_log_line_nintendulator(nes: &Nes) -> String {
    
    let part1 = match nes.cpu.instruction.number_of_operands() {
        0 => format!("{:02X}", nes.cpu.trace_opc),
        1 => format!("{:02X} {:02X}", nes.cpu.trace_opc, nes.cpu.trace_operand_1),
        2 => format!("{:02X} {:02X} {:02X}", nes.cpu.trace_opc, nes.cpu.trace_operand_1, nes.cpu.trace_operand_2),
        _ => unreachable!()
    };

    let part2 = if nes.cpu.instruction.is_unofficial {
        format!("*{:?}", nes.cpu.instruction.name)
    } else {
        format!("{:?}", nes.cpu.instruction.name)
    };

    let part3 = match nes.cpu.instruction.mode {
        Implied => String::from(""),
        Accumulator => format!(
            "A"
        ),
        Immediate => format!(
            "#${:02X}", 
            nes.cpu.trace_operand_1
        ),
        ZeroPage => format!(
            "${:02X} = {:02X}", 
            nes.cpu.trace_operand_1, 
            nes.cpu.trace_data
        ),
        ZeroPageX => format!(
            "${:02X},X @ {:02X} = {:02X}", 
            nes.cpu.trace_operand_1, 
            nes.cpu.trace_operand_1.wrapping_add(nes.cpu.trace_x), 
            nes.cpu.trace_data
        ),
        ZeroPageY => format!(
            "${:02X},Y @ {:02X} = {:02X}", 
            nes.cpu.trace_operand_1, 
            nes.cpu.trace_operand_1.wrapping_add(nes.cpu.trace_y), 
            nes.cpu.trace_data
        ),
        Absolute => {
            match nes.cpu.instruction.category {
                Control => format!(
                    "${:04X}", 
                    concat_u8(nes.cpu.trace_operand_2, nes.cpu.trace_operand_1)
                ),
                _ => format!(
                    "${:04X} = {:02X}", 
                    concat_u8(nes.cpu.trace_operand_2, nes.cpu.trace_operand_1), 
                    nes.cpu.trace_data
                ),
            }
        }

        AbsoluteX => format!(
            "${:04X},X @ {:04X} = {:02X}", 
            concat_u8(nes.cpu.trace_operand_2, nes.cpu.trace_operand_1),
            concat_u8(nes.cpu.trace_operand_2, nes.cpu.trace_operand_1).wrapping_add(nes.cpu.trace_x as u16),
            nes.cpu.trace_data
        ),
        AbsoluteY => format!(
            "${:04X},Y @ {:04X} = {:02X}", 
            concat_u8(nes.cpu.trace_operand_2, nes.cpu.trace_operand_1),
            concat_u8(nes.cpu.trace_operand_2, nes.cpu.trace_operand_1).wrapping_add(nes.cpu.trace_y as u16),
            nes.cpu.trace_data
        ),
        IndirectX => format!(
            "(${:02X},X) @ {:02X} = {:04X} = {:02X}", 
            nes.cpu.trace_operand_1, 
            nes.cpu.trace_operand_1.wrapping_add(nes.cpu.trace_x), 
            concat_u8(nes.cpu.trace_high_address, nes.cpu.trace_low_address),
            nes.cpu.trace_data
        ),
        IndirectY => format!(
            "(${:02X}),Y = {:04X} @ {:04X} = {:02X}", 
            nes.cpu.trace_operand_1, 
            concat_u8(nes.cpu.trace_high_address, nes.cpu.trace_low_address),
            concat_u8(nes.cpu.trace_high_address, nes.cpu.trace_low_address).wrapping_add(nes.cpu.trace_y as u16),
            nes.cpu.trace_data
        ),
        AbsoluteI => format!(
            "(${:04X}) = {:04X}",
            concat_u8(nes.cpu.trace_operand_2, nes.cpu.trace_operand_1),
            nes.cpu.pc,
        ),
        Relative => format!(
            "${:04X}",
            nes.cpu.trace_opc_addr.wrapping_add(2).wrapping_add_signed((nes.cpu.trace_operand_1 as i8) as i16),
        ),
    };

    let part4 = format!(
        "A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X} PPU:{:3},{:3} CYC:{}", 
        nes.cpu.trace_a,
        nes.cpu.trace_x,
        nes.cpu.trace_y,
        nes.cpu.trace_p,
        nes.cpu.trace_s,
        nes.cpu.trace_initial_ppu_scanline,
        nes.cpu.trace_initial_ppu_scanline_cycle,
        nes.cpu.trace_initial_cycle
    );

    let log_line = format!(
        "{:04X}  {:8} {:>4} {:30} {}", 
        nes.cpu.trace_opc_addr,
        part1,
        part2,
        part3,
        part4
    );

    log_line

}

fn create_log_line_mesen(nes: &Nes) -> String {
    
    let part1 = match nes.cpu.instruction.number_of_operands() {
        0 => format!("{:02X}", nes.cpu.trace_opc),
        1 => format!("{:02X} {:02X}", nes.cpu.trace_opc, nes.cpu.trace_operand_1),
        2 => format!("{:02X} {:02X} {:02X}", nes.cpu.trace_opc, nes.cpu.trace_operand_1, nes.cpu.trace_operand_2),
        _ => unreachable!()
    };

    let part2 = if nes.cpu.instruction.is_unofficial {
        format!("*{:?}", nes.cpu.instruction.name)
    } else {
        format!("{:?}", nes.cpu.instruction.name)
    };

    let part3 = match nes.cpu.instruction.mode {
        Implied => String::from(""),
        Accumulator => format!(
            "A"
        ),
        Immediate => format!(
            "#${:02X}", 
            nes.cpu.trace_operand_1
        ),
        ZeroPage => format!(
            "${:02X} = ${:02X}", 
            nes.cpu.trace_operand_1, 
            nes.cpu.trace_data
        ),
        ZeroPageX => format!(
            "${:02X},X [${:04X}] = ${:02X}", 
            nes.cpu.trace_operand_1, 
            nes.cpu.trace_operand_1.wrapping_add(nes.cpu.trace_x), 
            nes.cpu.trace_data
        ),
        ZeroPageY => format!(
            "${:02X},Y [${:04X}] = ${:02X}", 
            nes.cpu.trace_operand_1, 
            nes.cpu.trace_operand_1.wrapping_add(nes.cpu.trace_y), 
            nes.cpu.trace_data
        ),
        Absolute => {
            match nes.cpu.instruction.category {
                Control => format!(
                    "${:04X}", 
                    concat_u8(nes.cpu.trace_operand_2, nes.cpu.trace_operand_1)
                ),
                _ => format!(
                    "${:04X} = ${:02X}", 
                    concat_u8(nes.cpu.trace_operand_2, nes.cpu.trace_operand_1), 
                    nes.cpu.trace_read_absolute_addr_first_cycle,
                ),
            }
        }

        AbsoluteX => format!(
            "${:04X},X [${:04X}] = ${:02X}", 
            concat_u8(nes.cpu.trace_operand_2, nes.cpu.trace_operand_1),
            concat_u8(nes.cpu.trace_operand_2, nes.cpu.trace_operand_1).wrapping_add(nes.cpu.trace_x as u16),
            nes.cpu.trace_data
        ),
        AbsoluteY => format!(
            "${:04X},Y [${:04X}] = ${:02X}", 
            concat_u8(nes.cpu.trace_operand_2, nes.cpu.trace_operand_1),
            concat_u8(nes.cpu.trace_operand_2, nes.cpu.trace_operand_1).wrapping_add(nes.cpu.trace_y as u16),
            nes.cpu.trace_data
        ),
        IndirectX => format!(
            "(${:02X},X) {:02X} = {:04X} = ${:02X}", 
            nes.cpu.trace_operand_1, 
            nes.cpu.trace_operand_1.wrapping_add(nes.cpu.trace_x), 
            concat_u8(nes.cpu.trace_high_address, nes.cpu.trace_low_address),
            nes.cpu.trace_data
        ),
        IndirectY => format!(
            "(${:02X}),Y [${:04X}] = ${:02X}", 
            nes.cpu.trace_operand_1, 
            concat_u8(nes.cpu.trace_high_address, nes.cpu.trace_low_address).wrapping_add(nes.cpu.trace_y as u16),
            nes.cpu.trace_data
        ),
        AbsoluteI => format!(
            "(${:04X}) = ${:04X}",
            concat_u8(nes.cpu.trace_operand_2, nes.cpu.trace_operand_1),
            nes.cpu.pc,
        ),
        Relative => format!(
            "${:04X}",
            nes.cpu.trace_opc_addr.wrapping_add(2).wrapping_add_signed((nes.cpu.trace_operand_1 as i8) as i16),
        ),
    };

    let part4 = format!(
        "A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X} CYC:{}", 
        // "A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X} PPU:{},{} CYC:{}", 
        // "A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X} PPU:{},{} CYC:{} VBLANK:{} TRACE_2002:{:08b}", 
        nes.cpu.trace_a,
        nes.cpu.trace_x,
        nes.cpu.trace_y,
        nes.cpu.trace_p,
        nes.cpu.trace_s,
        // nes.cpu.trace_initial_ppu_scanline,
        // nes.cpu.trace_initial_ppu_scanline_cycle,
        nes.cpu.trace_initial_cycle,
        // nes.cpu.trace_vblank,
        // nes.cpu.trace_read_absolute_addr_first_cycle,
    );

    let log_line = format!(
        "{:04X}  {:8} {:>4} {:30} {}", 
        nes.cpu.trace_opc_addr,
        part1,
        part2,
        part3,
        part4
    );

    log_line

}
