use crate::hw::*;
use crate::mem::*;
use crate::instr_defs::{INSTRUCTIONS, Mode::*, Category::*, Name::*};
use crate::addressing_funcs::*;
use crate::util::*;
use crate::instr_funcs::update_p_nz;


const DUMMY_READ_FROM_PC:      fn(&mut Nes) = read_from_pc;
const DUMMY_READ_FROM_ADDRESS: fn(&mut Nes) = read_from_address;
const DUMMY_READ_FROM_POINTER: fn(&mut Nes) = read_from_pointer;

pub fn step_cpu(nes: &mut Nes) {

    /*
        If the NMI signal has been raised by the PPU 
        AND the instruction that was running at the time it was raised has completed
        AND the CPU is not already transferring control to the NMI handler,
        start transferring control to the NMI handler. 

        Once completed, reset all flags. 
        The CPU doesn't actually reset the NMI signal when it's done since the PPU controls it. 
        Not sure about the exact steps involved.
    */
    if nes.cpu.nmi_interrupt && nes.cpu.instruction_cycle == 0 && !nes.cpu.nmi_internal_flag {
        nes.cpu.nmi_internal_flag = true;
        nes.cpu.interrupt_request = false; // I think this happens
    }

    if nes.cpu.nmi_internal_flag {
        match nes.cpu.instruction_cycle {
            0 => DUMMY_READ_FROM_PC(nes),
            1 => {push_upper_pc_to_stack(nes); decrement_s(nes);}
            2 => {push_lower_pc_to_stack(nes); decrement_s(nes);}
            3 => {push_p_to_stack_during_interrupt(nes); decrement_s(nes);}
            4 => {fetch_lower_pc_from_nmi_vector(nes);}
            5 => {
                fetch_upper_pc_from_nmi_vector(nes); 
                nes.cpu.nmi_interrupt = false;
                nes.cpu.nmi_internal_flag = false;
                nes.cpu.instruction_cycle = -1;
            }
            _ => unreachable!(),
        }
        nes.cpu.instruction_cycle += 1;
        return;
    }

    // not considering interrupt hijacking yet

    if nes.cpu.interrupt_request && nes.cpu.instruction_cycle == 0 && !nes.cpu.irq_internal_flag
       && !nes.cpu.p_i {
        nes.cpu.irq_internal_flag = true;
    }

    if nes.cpu.irq_internal_flag {
        match nes.cpu.instruction_cycle {
            0 => DUMMY_READ_FROM_PC(nes),
            1 => {push_upper_pc_to_stack(nes); decrement_s(nes);}
            2 => {push_lower_pc_to_stack(nes); decrement_s(nes);}
            3 => {push_p_to_stack_during_interrupt(nes); decrement_s(nes);}
            4 => {fetch_lower_pc_from_interrupt_vector(nes);}
            5 => {
                fetch_upper_pc_from_interrupt_vector(nes); 
                nes.cpu.interrupt_request = false;
                nes.cpu.irq_internal_flag = false;
                nes.cpu.instruction_cycle = -1;
            }
            _ => unreachable!(),
        }
        nes.cpu.instruction_cycle += 1;
        return;
    }



    if nes.cpu.instruction_cycle == 0 {

        let opcode = read_mem(nes.cpu.pc, nes);

        nes.cpu.trace_opcode = opcode;
        nes.cpu.instruction = INSTRUCTIONS[opcode as usize];
        
        if nes.cpu.instruction.name == UJAM {
            nes.jammed = true;
        }

        // nes.old_cpu_state = nes.cpu;
        // nes.old_ppu_state = nes.ppu;

        increment_pc(nes);

        nes.cpu.cycles += 1;
        nes.cpu.instruction_cycle += 1;
        
        return;
    }



    /*
        Firstly, deal with control instructions that need special handling
        The match below tells the CPU what to do at each instruction cycle
    */

    let i = nes.cpu.instruction;
    let c = nes.cpu.instruction_cycle;
    let cat = nes.cpu.instruction.category; 

    if nes.cpu.instruction.category == C {
        match (i.name, i.mode) {
            (BRK, _) => { match c {
                1 => {DUMMY_READ_FROM_PC(nes); increment_pc(nes);}
                2 => {push_upper_pc_to_stack(nes); decrement_s(nes);}
                3 => {push_lower_pc_to_stack(nes); decrement_s(nes);}
                4 => {push_p_to_stack(nes); decrement_s(nes);}
                5 => {fetch_lower_pc_from_interrupt_vector(nes);}
                6 => {fetch_upper_pc_from_interrupt_vector(nes); end_instr(nes);}
                _ => unreachable!(),
            }}
            (RTI, _) => { match c {
                1 => {DUMMY_READ_FROM_PC(nes);}
                2 => {increment_s(nes);}
                3 => {pull_p_from_stack(nes); increment_s(nes);}
                4 => {pull_lower_pc_from_stack(nes); increment_s(nes);}
                5 => {pull_upper_pc_from_stack(nes); end_instr(nes);}
                _ => unreachable!(),
            }}
            (RTS, _) => { match c {
                1 => {DUMMY_READ_FROM_PC(nes);}
                2 => {increment_s(nes);}
                3 => {pull_lower_pc_from_stack(nes); increment_s(nes);}
                4 => {pull_upper_pc_from_stack(nes);}
                5 => {increment_pc(nes); end_instr(nes);}
                _ => unreachable!(),
            }}
            (JSR, _) => { match c {
                1 => {fetch_lower_address_from_pc(nes); increment_pc(nes);}
                2 => {none(nes);}
                3 => {push_upper_pc_to_stack(nes); decrement_s(nes);}
                4 => {push_lower_pc_to_stack(nes); decrement_s(nes);}
                5 => {fetch_upper_address_from_pc(nes); copy_address_to_pc(nes); end_instr(nes);} 
                _ => unreachable!(),
            }}
            (PHA, _) => { match c {
                1 => {DUMMY_READ_FROM_PC(nes);}
                2 => {push_a_to_stack(nes); decrement_s(nes); end_instr(nes);}
                _ => unreachable!(),
            }}
            (PHP, _) => { match c {
                1 => {DUMMY_READ_FROM_PC(nes);}
                2 => {push_p_to_stack(nes); decrement_s(nes); end_instr(nes);}
                _ => unreachable!(),
            }}
            (PLA, _) => { match c {
                1 => {DUMMY_READ_FROM_PC(nes);}
                2 => {increment_s(nes);}
                3 => {pull_a_from_stack(nes); update_p_nz(nes.cpu.a, nes); end_instr(nes);}
                _ => unreachable!(),
            }}
            (PLP, _) => { match c {
                1 => {DUMMY_READ_FROM_PC(nes);}
                2 => {increment_s(nes);}
                3 => {pull_p_from_stack(nes); end_instr(nes);}
                _ => unreachable!(),
            }}
            (JMP, Absolute) => { match c {
                1 => {fetch_lower_address_from_pc(nes); increment_pc(nes);}
                2 => {fetch_upper_address_from_pc(nes); copy_address_to_pc(nes); end_instr(nes);}
                _ => unreachable!(),
            }}
            (JMP, AbsoluteI) => { match c {
                1 => {fetch_lower_pointer_address_from_pc(nes); increment_pc(nes);}
                2 => {fetch_upper_pointer_address_from_pc(nes); increment_pc(nes);}
                3 => {fetch_lower_address_from_pointer(nes); increment_lower_pointer(nes);}
                4 => {fetch_upper_address_from_pointer(nes); copy_address_to_pc(nes); end_instr(nes);}
                _ => unreachable!(),
            }}
            _ => unreachable!(),
        };
    }

    // Next, deal with branches, which behave differently from other instructions

    else if nes.cpu.instruction.category == B {
        match c {
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
                    _   => unreachable!(),
                };

                if !nes.cpu.branching {end_instr(nes);}
            }
            2 => {
                // DUMMY READ!
                // Idk where it's reading from 
                // come back to this later, should work fine for the now
                let prev_pcl = nes.cpu.pc as u8;
                let (new_pcl, overflow) = prev_pcl.overflowing_add_signed(nes.cpu.branch_offset as i8);
                
                nes.cpu.set_lower_pc(new_pcl);

                nes.cpu.internal_carry_out = overflow;

                if !nes.cpu.internal_carry_out {end_instr(nes);}
            }
            3 => {
                // need more dummy reads here
                if is_neg(nes.cpu.branch_offset) {
                    nes.cpu.pc = nes.cpu.pc.wrapping_sub(1 << 8);
                } else {
                    nes.cpu.pc = nes.cpu.pc.wrapping_add(1 << 8);
                }
                end_instr(nes);
            }
            _ => unreachable!(),
        }
    }
    

    
    /*
        If not a control instruction, must be a read/write/read-modify-write instruction
        The per-cycle operation of these instructions is defined by three things:
            - The addressing mode of the instruction (e.g. ZeroPage, IndirectX)
            - A data operation (e.g. shift left, subtract)
            - The instruction's category (read, write, read-modify-write)

        The first cycles of an instruction are determined by the addressing mode

        Explain addressing modes? 

    */

    // These category matches could be in a match? idk
    // could have an instruction state thing, like an enum for each section of 

    
    else if cat == R || cat == W || cat == RMW {
        match nes.cpu.instruction.mode {
            Implied | Accumulator => { match c {
                1 => DUMMY_READ_FROM_PC(nes),
                _ => (),
            }}
            Immediate => { match c {
                1 => {fetch_immediate_from_pc(nes); increment_pc(nes);}
                _ => (),
            }}
            ZeroPage => { match c {
                1 => {fetch_lower_address_from_pc(nes); increment_pc(nes);}
                _ => (),
            }}
            ZeroPageX => { match c {
                1 => {fetch_lower_address_from_pc(nes); increment_pc(nes);}
                2 => {DUMMY_READ_FROM_ADDRESS(nes); add_x_to_lower_address(nes);}
                _ => (),
            }}
            ZeroPageY => { match c {
                1 => {fetch_lower_address_from_pc(nes); increment_pc(nes);}
                2 => {DUMMY_READ_FROM_ADDRESS(nes); add_y_to_lower_address(nes);}
                _ => (),
            }}
            Absolute => { match c {
                1 => {fetch_lower_address_from_pc(nes); increment_pc(nes);}
                2 => {fetch_upper_address_from_pc(nes); increment_pc(nes);}
                _ => (),
            }}
            AbsoluteX => { match c {
                1 => {fetch_lower_address_from_pc(nes); increment_pc(nes);}
                2 => {fetch_upper_address_from_pc(nes); add_x_to_lower_address(nes); increment_pc(nes);}
                _ => (),
            }}
            AbsoluteY => { match c {
                1 => {fetch_lower_address_from_pc(nes); increment_pc(nes);}
                2 => {fetch_upper_address_from_pc(nes); add_y_to_lower_address(nes); increment_pc(nes);}
                _ => (),
            }}
            IndirectX => { match c {
                1 => {fetch_lower_pointer_address_from_pc(nes); increment_pc(nes);}
                2 => {DUMMY_READ_FROM_POINTER(nes); add_x_to_lower_pointer(nes);}
                3 => {fetch_lower_address_from_pointer(nes); increment_lower_pointer(nes);}
                4 => {fetch_upper_address_from_pointer(nes);}
                _ => (),
            }}
            IndirectY => { match c {
                1 => {fetch_lower_pointer_address_from_pc(nes); increment_pc(nes);}
                2 => {fetch_lower_address_from_pointer(nes); increment_lower_pointer(nes);}
                3 => {fetch_upper_address_from_pointer(nes); add_y_to_lower_address(nes);}
                _ => (),
            }}
            _ => unreachable!(),
        }

        /*
            Each addressing mode takes a different number of cycles to calculate the address in 
            memory that the instruction needs to work with.
            
            After the address has been resolved, the instruction will either read from this address,
            write to it, or read it, modify the data, and write it back to memory. 

            Subtracting the current (explain more)
        
        */ 


        /*

            The only accumulator instructions are:
                0A - ASL
                2A - ROL
                4A - LSR
                6A - ROR

            These are all 2 cycles, with the third overlapping with the next instruction 

            They are all RMW instructions as well 


            Implied instructions that haven't already been covered

            All flag clears/sets

            decrement x/y
            increment x/y
            transfers
            nop 

            all of these can just call the function!
            no other stuff needed

            Then the only thing left is branches? 

            so, branches
            https://archive.nes.science/nesdev-forums/f3/t1421.xhtml

            first cycle, get opcode
            second, get offset, determine branch status

            if branch, add offset to pcl 
            
            fetch opcode from new pc, fix pch, if it didn't change, increment 

            if it was fixed, fetch opcode again, increment pc


            all immediate instructions just do things on registers, doesn't seem to be any memory stuff
            just the same as implied then, or at least the ones that aren't control instructions


        */


        // Actually there can be a match here straight after matching with something above
        // accumulator or implied or immediate instructions



        let c = nes.cpu.instruction_cycle as i8 - nes.cpu.instruction.mode.address_resolution_cycles() as i8;
        let func = nes.cpu.instruction.name.function();

        match nes.cpu.instruction.mode {
            Accumulator => { match c {
                0 => {nes.cpu.data = nes.cpu.a; func(nes); nes.cpu.a = nes.cpu.data; end_instr(nes);}
                _ => unreachable!(),
            }}
            Implied | Immediate => { match c {
                0 => {func(nes); end_instr(nes);}
                _ => unreachable!(),
            }}
            Absolute | ZeroPage | ZeroPageX | ZeroPageY | IndirectX => { match (cat, c) {
                (R,   1) => {read_from_address(nes); func(nes); end_instr(nes);}
                (W,   1) => {func(nes); end_instr(nes);}
                (RMW, 1) => read_from_address(nes),
                (RMW, 2) => {write_to_address(nes); func(nes);}
                (RMW, 3) => {write_to_address(nes); end_instr(nes);}
                (_,   _) => (),
            }}
            AbsoluteX | AbsoluteY | IndirectY => { match (cat, c) {
                (R, 1)   => {
                    read_from_address(nes); 
                    add_lower_address_carry_bit_to_upper_address(nes);
                    if !nes.cpu.internal_carry_out {
                        func(nes);
                        end_instr(nes);
                    }
                }
                (R, 2)   => {read_from_address(nes); func(nes); end_instr(nes);}

                (W, 1)   => {DUMMY_READ_FROM_ADDRESS(nes); add_lower_address_carry_bit_to_upper_address(nes);}
                (W, 2)   => {func(nes); end_instr(nes);}

                (RMW, 1) => {DUMMY_READ_FROM_ADDRESS(nes); add_lower_address_carry_bit_to_upper_address(nes);}
                (RMW, 2) => read_from_address(nes),
                (RMW, 3) => {write_to_address(nes); func(nes);}
                (RMW, 4) => {write_to_address(nes); end_instr(nes);}

                (_, _) => (),
            }}
            _ => unreachable!(),
        }

    }

    // println!("End of instruction cycle {}, address is {}", nes.cpu.instruction_cycle, nes.cpu.get_address());


    nes.cpu.cycles += 1;
    nes.cpu.instruction_cycle += 1;

}


fn end_instr(nes: &mut Nes) {

    // let log_str = log(nes);

    // if nes.cpu.cycles > 10000000 {println!("{}", &log_str);}

    nes.cpu.data = 0;
    nes.cpu.lower_address = 0;
    nes.cpu.upper_address = 0;
    nes.cpu.lower_pointer = 0;
    nes.cpu.upper_pointer = 0;
    nes.cpu.internal_carry_out = false;
    nes.cpu.branch_offset = 0;
    nes.cpu.branching = false;

    nes.cpu.instruction_cycle = -1;
    nes.cpu.instruction_count += 1;
}


fn log(nes: &Nes) -> String {

    let instr_len = match nes.cpu.instruction.mode {
        Accumulator => 1,
        Implied => 1,
        Immediate => 2,
        Absolute  => 3,
        AbsoluteX => 3,
        AbsoluteY => 3,
        ZeroPage  => 2,
        ZeroPageX => 2,
        ZeroPageY => 2,
        Relative  => 2,
        IndirectX => 2,
        IndirectY => 2,
        AbsoluteI => 3,
    };

    let mut bytes_str = String::new();

    let opcode_str = format!("{:02X} ", nes.cpu.trace_opcode);
    bytes_str.push_str(&opcode_str);

    if instr_len >= 2 {
        let byte2_str = format!("{:02X} ", nes.cpu.trace_byte2);
        bytes_str.push_str(&byte2_str);
    }
    if instr_len == 3 {
        let byte3_str = format!("{:02X}", nes.cpu.trace_byte3);
        bytes_str.push_str(&byte3_str);
    }

    let mut instr_str = format!("{:?} ", nes.cpu.instruction.name);

    let addressing_str = match nes.cpu.instruction.mode {
        Implied => String::new(),
        Accumulator => format!(
            "A"
        ),
        Immediate => format!(
            "#${:02X}", 
            nes.cpu.trace_imm
        ),
        ZeroPage => format!(
            "${:02X} = {:02X}", 
            nes.cpu.trace_byte2, 
            nes.cpu.trace_stored_val
        ),
        ZeroPageX => format!(
            "${:02X},X @ {:02X} = {:02X}", 
            nes.cpu.trace_byte2, 
            nes.cpu.get_address(), 
            nes.cpu.trace_stored_val
        ),
        ZeroPageY => format!(
            "${:02X},Y @ {:02X} = {:02X}", 
            nes.cpu.trace_byte2, 
            nes.cpu.get_address(), 
            nes.cpu.trace_stored_val
        ),
        Absolute => {
            if nes.cpu.instruction.name != JMP && nes.cpu.instruction.name != JSR {
                format!(
                    "${:04X?} = {:02X}", 
                    nes.cpu.get_address(), 
                    nes.cpu.trace_stored_val
                )
            } else {
                format!(
                    "${:04X?}", 
                    nes.cpu.get_address(), 
                )
            }
        }

        AbsoluteX => format!(
            "${:04X?},X @ {:04X} = {:02X}", 
            concat_u8(nes.cpu.trace_byte3, nes.cpu.trace_byte2), 
            nes.cpu.get_address(), 
            nes.cpu.trace_stored_val
        ),
        AbsoluteY => format!(
            "${:04X?},Y @ {:04X} = {:02X}", 
            concat_u8(nes.cpu.trace_byte3, nes.cpu.trace_byte2), 
            nes.cpu.get_address(), 
            nes.cpu.trace_stored_val
        ),
        IndirectX => format!(
            "(${:02X},X) @ {:02X} = {:04X} = {:02X}", 
            nes.cpu.trace_byte2, 
            nes.cpu.trace_byte2.wrapping_add(nes.cpu.x), 
            nes.cpu.get_address(), 
            nes.cpu.trace_stored_val
        ),
        IndirectY => format!(
            "(${:02X}),Y = {:04X} @ {:04X} = {:02X}", 
            nes.cpu.trace_byte2, 
            nes.cpu.get_address().wrapping_sub(nes.cpu.y as u16),
            nes.cpu.get_address(), 
            nes.cpu.trace_stored_val
        ),
        AbsoluteI => format!(
            "(${:04X}) = {:04X}",
            concat_u8(nes.cpu.upper_pointer, nes.cpu.lower_pointer.wrapping_sub(1)),
            nes.cpu.pc,
        ),
        Relative => format!(
            "${:04X}",
            nes.old_cpu_state.pc.wrapping_add_signed(2 + nes.cpu.branch_offset as i8 as i16),
        ),
    };

    instr_str.push_str(&addressing_str);
    

    let register_str = format!(
        "A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X} PPU:{:>3},{:>3} CYC:{}",
        nes.old_cpu_state.a,
        nes.old_cpu_state.x,
        nes.old_cpu_state.y,
        nes.old_cpu_state.get_p(),
        nes.old_cpu_state.s,
        nes.old_ppu_state.scanline,
        nes.old_ppu_state.scanline_cycle,
        nes.old_cpu_state.cycles,
    );
    
    let log_str = format!(
        "{:04X}  {:10}{:32}{}",
        nes.old_cpu_state.pc,
        &bytes_str,
        &instr_str,
        &register_str,
    );

    log_str
}