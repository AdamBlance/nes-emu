use crate::nes::Nes;
use super::addressing::*;
use crate::mem::read_mem;
use super::lookup_table::{
    INSTRUCTIONS,
    Mode::*,
    Category::*,
    Name::*,
};
use super::operation_funcs::{update_p_nz, set_interrupt_inhibit_flag};
use crate::util::{is_neg, concat_u8};
use std::io::Write;

/*
    Because of the way the CPU is designed, it often reads data from memory when it isn't needed. 
    This wouldn't matter for emulation if memory reads were "stateless", but unfortunately this 
    isn't the case. Some memory addresses have side effects when they are read; for example, 
    address 0x2007 (PPUDATA) changes the VRAM address inside the PPU when read from. Therefore, 
    we have to do these dummy reads if we want to match the NES's behaviour exactly. 
    
    These aliases are just to help distinguish "useful" reads from dummy reads.
*/
const DUMMY_READ_FROM_PC:      fn(&mut Nes) = read_from_pc;
// const dummy_read_from_address: fn(&mut Nes) = read_from_address;
const DUMMY_READ_FROM_POINTER: fn(&mut Nes) = read_from_pointer;

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
        
        if nes.cpu.nmi_pending {
            // println!("IN NMI, cycle {}", nes.cpu.interrupt_cycle);
            match nes.cpu.interrupt_cycle {
                0 => {DUMMY_READ_FROM_PC(nes); nes.cpu.irq_pending = false; nes.cpu.interrupt_vector = 0xFFFA;}
                1 => DUMMY_READ_FROM_PC(nes),
                2 => {push_upper_pc_to_stack(nes); decrement_s(nes);}
                3 => {push_lower_pc_to_stack(nes); decrement_s(nes);}
                4 => {push_p_to_stack(nes); decrement_s(nes);}
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
            if nes.cpu.pause {println!("IN IRQ, cycle {}", nes.cpu.interrupt_cycle);}
            match nes.cpu.interrupt_cycle {
                0 => {DUMMY_READ_FROM_PC(nes); nes.cpu.interrupt_vector = 0xFFFE;}
                1 => DUMMY_READ_FROM_PC(nes),
                2 => {push_upper_pc_to_stack(nes); decrement_s(nes);}
                3 => {push_lower_pc_to_stack(nes); decrement_s(nes);}
                4 => {push_p_to_stack(nes); decrement_s(nes);}
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
        
        // If no interrupts are pending, start executing next instruction
        else {
            let opcode = read_mem(nes.cpu.pc, nes);
            nes.cpu.trace_opc = opcode;
            nes.cpu.instruction = INSTRUCTIONS[opcode as usize];
            if nes.cpu.instruction.category == Unimplemented {
                unimplemented!("Unofficial instruction {:?} not implemented!", nes.cpu.instruction.name);
            }
            if nes.cpu.pause {
                println!(
                    "Instruction {:?}, opcode {:02X},  PC {:04X} cycles {} regs a {:02X} x {:02X} y {:02X} inhibit {} line {} cycle {}", 
                    nes.cpu.instruction.name, 
                    opcode, 
                    nes.cpu.pc, 
                    nes.cpu.cycles, 
                    nes.cpu.a, 
                    nes.cpu.x, 
                    nes.cpu.y,
                    nes.cpu.p_i,
                    nes.ppu.scanline,
                    nes.ppu.scanline_cycle,
                    // nes.cart.get_counter(),
                );
            }
            if nes.cpu.cycles == nes.cpu.target && nes.cpu.pause {
                let mut line = String::new();
                std::io::stdin().read_line(&mut line);
                let step_by: u64 = line.trim().parse().unwrap_or(1);
        
                nes.cpu.target = step_by;
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

    // if nes.cpu.cycles == nes.cpu.target && nes.cpu.pause {
    //     let mut line = String::new();
    //     std::io::stdin().read_line(&mut line);
    //     let step_by: u64 = line.trim().parse().unwrap_or(1);

    //     nes.cpu.target = step_by;
    // }

    /*
        The second instruction cycle (cycle 1) is when instructions start to do things. 
        
        First, we deal with all of the 2-cycle insrtuctions. The first cycle is spent fetching 
        the opcode, so these only take one additional cycle. 
    */

    let instr = nes.cpu.instruction;
    let cyc = nes.cpu.instruction_cycle;
    let cat = nes.cpu.instruction.category; 
    let func = nes.cpu.instruction.get_associated_function();
    
    if instr.mode == Accumulator {
        DUMMY_READ_FROM_PC(nes);
        nes.cpu.data = nes.cpu.a;
        func(nes);
        nes.cpu.a = nes.cpu.data;
        nes.cpu.instruction_done = true;
    }
    else if cat == Register || instr.name == NOP {
        DUMMY_READ_FROM_PC(nes);
        func(nes);
        nes.cpu.instruction_done = true;
    }
    else if instr.mode == Immediate {
        fetch_immediate_from_pc(nes);
        increment_pc(nes);
        func(nes);
        nes.cpu.instruction_done = true;
    }

    // Next, deal with control instructions. These need special handling. 

    else if cat == Control {
        match (instr.name, instr.mode) {
            (BRK, _) => { match cyc {
                1 => {DUMMY_READ_FROM_PC(nes); increment_pc(nes); nes.cpu.interrupt_vector = 0xFFFE;}
                2 => {push_upper_pc_to_stack(nes); decrement_s(nes);}
                3 => {push_lower_pc_to_stack(nes); decrement_s(nes);}
                4 => {push_p_to_stack_during_break(nes); decrement_s(nes);}
                5 => {fetch_lower_pc_from_interrupt_vector(nes); set_interrupt_inhibit_flag(nes);}
                6 => {fetch_upper_pc_from_interrupt_vector(nes); nes.cpu.instruction_done = true;}
                _ => unreachable!(),
            }}
            (RTI, _) => { match cyc {
                1 => {DUMMY_READ_FROM_PC(nes);}
                2 => {increment_s(nes);}
                3 => {pull_p_from_stack(nes); increment_s(nes);}
                4 => {pull_lower_pc_from_stack(nes); increment_s(nes);}
                5 => {pull_upper_pc_from_stack(nes); nes.cpu.instruction_done = true;}
                _ => unreachable!(),
            }}
            (RTS, _) => { match cyc {
                1 => {DUMMY_READ_FROM_PC(nes);}
                2 => {increment_s(nes);}
                3 => {pull_lower_pc_from_stack(nes); increment_s(nes);}
                4 => {pull_upper_pc_from_stack(nes);}
                5 => {increment_pc(nes); nes.cpu.instruction_done = true;}
                _ => unreachable!(),
            }}
            (JSR, _) => { match cyc {
                1 => {take_operand_as_low_address_byte(nes); increment_pc(nes);}
                2 => {none(nes);}
                3 => {push_upper_pc_to_stack(nes); decrement_s(nes);}
                4 => {push_lower_pc_to_stack(nes); decrement_s(nes);}
                5 => {take_operand_as_high_address_byte(nes); copy_address_to_pc(nes); nes.cpu.instruction_done = true;} 
                _ => unreachable!(),
            }}
            (PHA, _) => { match cyc {
                1 => {DUMMY_READ_FROM_PC(nes);}
                2 => {push_a_to_stack(nes); decrement_s(nes); nes.cpu.instruction_done = true;}
                _ => unreachable!(),
            }}
            (PHP, _) => { match cyc {
                1 => {DUMMY_READ_FROM_PC(nes);}
                2 => {push_p_to_stack(nes); decrement_s(nes); nes.cpu.instruction_done = true;}
                _ => unreachable!(),
            }}
            (PLA, _) => { match cyc {
                1 => {DUMMY_READ_FROM_PC(nes);}
                2 => {increment_s(nes);}
                3 => {pull_a_from_stack(nes); update_p_nz(nes, nes.cpu.a); nes.cpu.instruction_done = true;}
                _ => unreachable!(),
            }}
            (PLP, _) => { match cyc {
                1 => {DUMMY_READ_FROM_PC(nes);}
                2 => {increment_s(nes);}
                3 => {pull_p_from_stack(nes); nes.cpu.instruction_done = true;}
                _ => unreachable!(),
            }}
            (JMP, Absolute) => { match cyc {
                1 => {take_operand_as_low_address_byte(nes); increment_pc(nes);}
                2 => {take_operand_as_high_address_byte(nes); copy_address_to_pc(nes); nes.cpu.instruction_done = true;}
                _ => unreachable!(),
            }}
            (JMP, AbsoluteI) => { match cyc {
                1 => {take_operand_as_low_indirect_address_byte(nes); increment_pc(nes);}
                2 => {take_operand_as_high_indirect_address_byte(nes); increment_pc(nes);}
                3 => {fetch_low_address_byte_using_indirect_address(nes);}
                4 => {fetch_high_address_byte_using_indirect_address(nes); copy_address_to_pc(nes); nes.cpu.instruction_done = true;}
                _ => unreachable!(),
            }}
            _ => unreachable!(),
        };
    }

    // Next, deal with branches, which behave differently from other instructions.

    else if cat == Branch {
        match cyc {
            1 => {
                fetch_branch_offset_from_pc(nes); 
                increment_pc(nes); 
                nes.cpu.branching = match nes.cpu.instruction.name {
                    BCC => !nes.cpu.p_c,
                    BCS =>  nes.cpu.p_c,
                    BVC => !nes.cpu.p_v,
                    BVS =>  nes.cpu.p_v,
                    BNE => !nes.cpu.p_z,
                    BEQ =>  nes.cpu.p_z,
                    BPL => !nes.cpu.p_n,
                    BMI =>  nes.cpu.p_n,
                    _   =>  unreachable!(),
                };
                // Continue to next instruction if branch was not taken
                if !nes.cpu.branching {
                    nes.cpu.instruction_done = true;
                }
            }
            2 => {
                let prev_pcl = nes.cpu.pc as u8;
                let (new_pcl, overflow) = prev_pcl.overflowing_add_signed(nes.cpu.branch_offset as i8);
                nes.cpu.internal_carry_out = overflow;
                nes.cpu.set_lower_pc(new_pcl);
                // If branch didn't cross page boundary, continue to next instruction
                if !nes.cpu.internal_carry_out {
                    nes.cpu.instruction_done = true;
                }
            }
            3 => {
                // Fix upper PC if page was crossed
                if is_neg(nes.cpu.branch_offset) {
                    nes.cpu.pc = nes.cpu.pc.wrapping_sub(1 << 8);
                } else {
                    nes.cpu.pc = nes.cpu.pc.wrapping_add(1 << 8);
                }
                nes.cpu.instruction_done = true;
            }
            _ => unreachable!(),
        }
    }
    
    /*
        Instructions that aren't control or branch instructions read from and/or write to memory. 
        Each of these instructions has an addressing mode. This determines what location in memory
        the instruction works with. This is called the "effective address". Calculating the 
        effective address takes a different number of cycles depending on the addressing mode used. 
        The table at the top of instr_defs.rs summarises each mode.

        The next else-if tells the instruction what operations to do during each address resolution
        cycle. This can take between 1 and 4 cycles.
    */

    else if (cat == Read || cat == Write || cat == ReadModifyWrite) 
            && (nes.cpu.instruction_cycle <= instr.mode.address_resolution_cycles()) {

        match nes.cpu.instruction.mode {
            ZeroPage => { match cyc {
                1 => {take_operand_as_low_address_byte(nes); increment_pc(nes);}
                _ => unreachable!(),
            }}
            ZeroPageX => { match cyc {
                1 => {take_operand_as_low_address_byte(nes); increment_pc(nes);}
                2 => {dummy_read_from_address(nes); add_x_to_low_address_byte(nes);}
                _ => unreachable!(),
            }}
            ZeroPageY => { match cyc {
                1 => {take_operand_as_low_address_byte(nes); increment_pc(nes);}
                2 => {dummy_read_from_address(nes); add_y_to_low_address_byte(nes);}
                _ => unreachable!(),
            }}
            Absolute => { match cyc {
                1 => {take_operand_as_low_address_byte(nes); increment_pc(nes);}
                2 => {take_operand_as_high_address_byte(nes); increment_pc(nes);}
                _ => unreachable!(),
            }}
            AbsoluteX => { match cyc {
                1 => {take_operand_as_low_address_byte(nes); increment_pc(nes);}
                2 => {take_operand_as_high_address_byte(nes); add_x_to_low_address_byte(nes); increment_pc(nes);}
                _ => unreachable!(),
            }}
            AbsoluteY => { match cyc {
                1 => {take_operand_as_low_address_byte(nes); increment_pc(nes);}
                2 => {take_operand_as_high_address_byte(nes); add_y_to_low_address_byte(nes); increment_pc(nes);}
                _ => unreachable!(),
            }}
            IndirectX => { match cyc {
                1 => {take_operand_as_low_indirect_address_byte(nes); increment_pc(nes);}
                2 => {DUMMY_READ_FROM_POINTER(nes); add_x_to_low_indirect_address_byte(nes);}
                3 => {fetch_low_address_byte_using_indirect_address(nes);}
                4 => {fetch_high_address_byte_using_indirect_address(nes);}
                _ => unreachable!(),
            }}
            IndirectY => { match cyc {
                1 => {take_operand_as_low_indirect_address_byte(nes); increment_pc(nes);}
                2 => {fetch_low_address_byte_using_indirect_address(nes);}
                3 => {fetch_high_address_byte_using_indirect_address(nes); add_y_to_low_address_byte(nes);}
                _ => unreachable!(),
            }}
            _ => unreachable!(),
        }

    }

    /*
        This final else-if block tells the instruction what to do after it has calculated the 
        effective address. 
        There are 3 main types of instruction: read, write, and read-modify-write.
        
        Read instructions read a value from memory, optionally operate on it, and modify the CPU's 
        internal state by writing the value to a register, or updating a flag. 
        Some examples are LDX, AND, and SUB.

        Write instructions write a value from a register to memory. The only write instructions are 
        STA, STX, STY. 

        Read-modify-write instructions load a value into the CPU, modify the value, and write it 
        back to memory. These update flags like read instructions, but the value is never stored
        in a register. Some examples are ASL, ROR, and DEC.

        The instruction's addressing mode, along with whether the instruction is read, write, 
        or read-modify-write, determine what cycles are executed after the effective address 
        has been resolved. 
    */

    else if (cat == Read || cat == Write || cat == ReadModifyWrite) 
            && (nes.cpu.instruction_cycle > instr.mode.address_resolution_cycles()) {
        
        // This is the number of cycles that has elapsed since resolving the effective address
        let eac = nes.cpu.instruction_cycle - nes.cpu.instruction.mode.address_resolution_cycles();

        match nes.cpu.instruction.mode {

            Absolute | ZeroPage | ZeroPageX | ZeroPageY | IndirectX => { match (cat, eac) {

                (Read, 1) => {read_from_address(nes); func(nes); nes.cpu.instruction_done = true;}

                (Write, 1) => {func(nes); write_to_address(nes); nes.cpu.instruction_done = true;}  // the way this works is pretty stupid

                (ReadModifyWrite, 1) => read_from_address(nes),
                (ReadModifyWrite, 2) => {write_to_address(nes); func(nes);}
                (ReadModifyWrite, 3) => {write_to_address(nes); nes.cpu.instruction_done = true;}

                _ => unreachable!(),
            }}

            AbsoluteX | AbsoluteY | IndirectY => { match (cat, eac) {

                (Read, 1) => {
                    read_from_address(nes); 
                    add_lower_address_carry_bit_to_upper_address(nes);
                    // Continue to next instruction if page wasn't crossed
                    if !nes.cpu.internal_carry_out {
                        func(nes);
                        nes.cpu.instruction_done = true;
                    }
                }
                (Read, 2) => {read_from_address(nes); func(nes); nes.cpu.instruction_done = true;}

                (Write, 1) => {dummy_read_from_address(nes); add_lower_address_carry_bit_to_upper_address(nes);}
                (Write, 2) => {func(nes); write_to_address(nes); nes.cpu.instruction_done = true;}

                (ReadModifyWrite, 1) => {dummy_read_from_address(nes); add_lower_address_carry_bit_to_upper_address(nes);}
                (ReadModifyWrite, 2) => read_from_address(nes),
                (ReadModifyWrite, 3) => {write_to_address(nes); func(nes);}
                (ReadModifyWrite, 4) => {write_to_address(nes); nes.cpu.instruction_done = true;}

                _ => unreachable!(),
            }}
            _ => unreachable!(),
        }
    }

    /*
        At this point, the instruction is at the end of one of its cycles. 
        If the instruction just completed its last cycle, it will have called end_instr().
        This resets the instruction_cycle counter to -1, so it will be incremented here to 0. 
    */

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
    if nes.cpu.pause {println!("cart irq {} pending {}", nes.cart.asserting_irq(), nes.cpu.irq_pending);}
    nes.cpu.prev_irq_signal = nes.apu.asserting_irq() || nes.cart.asserting_irq();

    nes.cpu.cycles += 1;
    nes.cpu.instruction_cycle += 1;
    
}

fn end_instr(nes: &mut Nes) {
    // let log_str = log(nes);
    // println!("{}", log_str);

    writeln!(nes.logfile, "{}", create_log_line(nes)).unwrap();

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

    // 
    if !nes.cpu.instruction.does_interrupt_poll_early() {
        nes.cpu.nmi_pending = nes.cpu.nmi_edge_detector_output;
        nes.cpu.irq_pending = nes.cpu.prev_irq_signal && !nes.cpu.p_i;
    }
    
    nes.cpu.instruction_cycle = -1;
    nes.cpu.instruction_done = false;

    nes.cpu.instruction_count += 1;

    // if nes.cpu.instruction_count == 1717875 { nes.logfile.sync_all().unwrap(); panic!() }


}

fn create_log_line(nes: &Nes) -> String {
    
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
            nes.cpu.trace_operand_1.wrapping_add(nes.cpu.x), 
            nes.cpu.trace_data
        ),
        ZeroPageY => format!(
            "${:02X},Y @ {:02X} = {:02X}", 
            nes.cpu.trace_operand_1, 
            nes.cpu.trace_operand_1.wrapping_add(nes.cpu.y), 
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
            concat_u8(nes.cpu.trace_operand_2, nes.cpu.trace_operand_1).wrapping_add(nes.cpu.x as u16),
            nes.cpu.trace_data
        ),
        AbsoluteY => format!(
            "${:04X},Y @ {:04X} = {:02X}", 
            concat_u8(nes.cpu.trace_operand_2, nes.cpu.trace_operand_1),
            concat_u8(nes.cpu.trace_operand_2, nes.cpu.trace_operand_1).wrapping_add(nes.cpu.y as u16),
            nes.cpu.trace_data
        ),
        IndirectX => format!(
            "(${:02X},X) @ {:02X} = {:04X} = {:02X}", 
            nes.cpu.trace_operand_1, 
            nes.cpu.trace_operand_1.wrapping_add(nes.cpu.x), 
            concat_u8(nes.cpu.trace_high_address, nes.cpu.trace_low_address),
            nes.cpu.trace_data
        ),
        IndirectY => format!(
            "(${:02X}),Y = {:04X} @ {:04X} = {:02X}", 
            nes.cpu.trace_operand_1, 
            concat_u8(nes.cpu.trace_high_address, nes.cpu.trace_low_address),  // ok problem is trace_operand_1 and 2 don't get populated with the instructions that do stuff with indirect address
            concat_u8(nes.cpu.trace_high_address, nes.cpu.trace_low_address).wrapping_add(nes.cpu.y as u16),  // maybe get rid of trace indirect, just use operand 1 and 2
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
        "{:04X}  {:8} {:>4} {:27} {}", 
        nes.cpu.trace_opc_addr,
        part1,
        part2,
        part3,
        part4
    );

    log_line

}
