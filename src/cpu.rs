use std::io;
use crate::hw::*;
use crate::mem::*;
use crate::instr_defs::{INSTRUCTIONS, Mode::*, Category::*};
use crate::addressing_funcs::*;


const DUMMY_READ_FROM_PC:      fn(&mut Nes) = read_from_pc;
const DUMMY_READ_FROM_ADDRESS: fn(&mut Nes) = read_from_address;
const DUMMY_READ_FROM_POINTER: fn(&mut Nes) = read_from_pointer;

pub fn step_cpu(nes: &mut Nes) {

    if nes.cpu.instruction_cycle == 0 {
        let opcode = read_mem(nes.cpu.get_address(), nes);
        nes.cpu.instruction = INSTRUCTIONS[opcode as usize];
        return;
    }

    /*
        Firstly, deal with control instructions that need special handling
        The match below tells the CPU what to do at each instruction cycle
    */

    let i = nes.cpu.instruction;
    let c = nes.cpu.instruction_cycle;

    if nes.cpu.instruction.category == C {
        match (i.name, i.mode) {
            (BRK, _) => { match c {
                1 => {DUMMY_READ_FROM_PC(nes); increment_pc(nes);}
                2 => {push_upper_pc_to_stack(nes); decrement_s(nes);}
                3 => {push_lower_pc_to_stack(nes); decrement_s(nes);}
                4 => {push_p_to_stack_with_brk_flag(nes); decrement_s(nes);}
                5 => {fetch_lower_pc_from_interrupt_vector(nes);}
                6 => {fetch_upper_pc_from_interrupt_vector(nes);}
            }}
            (RTI, _) => { match c {
                1 => {DUMMY_READ_FROM_PC(nes);}
                2 => {increment_s(nes);}
                3 => {pull_p_from_stack(nes); increment_s(nes);}
                4 => {pull_lower_pc_from_stack(nes); increment_s(nes);}
                5 => {pull_upper_pc_from_stack(nes);}                
            }}
            (RTS, _) => { match c {
                1 => {DUMMY_READ_FROM_PC(nes);}
                2 => {increment_s(nes);}
                3 => {pull_lower_pc_from_stack(nes); increment_s(nes);}
                4 => {pull_upper_pc_from_stack(nes);}
                5 => {increment_pc(nes);}
            }}
            (JSR, _) => { match c {
                1 => {fetch_lower_address_from_pc(nes); increment_pc(nes);}
                2 => {none(nes);}
                3 => {push_upper_pc_to_stack(nes); decrement_s(nes);}
                4 => {push_lower_pc_to_stack(nes); decrement_s(nes);}
                5 => {fetch_upper_address_from_pc(nes); copy_address_to_pc(nes);} 
            }}
            (PHA, _) => { match c {
                1 => {DUMMY_READ_FROM_PC(nes);}
                2 => {push_a_to_stack(nes); decrement_s(nes);}
            }}
            (PHP, _) => { match c {
                1 => {DUMMY_READ_FROM_PC(nes);}
                2 => {push_p_to_stack(nes); decrement_s(nes);}
            }}
            (PLA, _) => { match c {
                1 => {DUMMY_READ_FROM_PC(nes);}
                2 => {increment_s(nes);}
                3 => {pull_a_from_stack(nes);}
            }}
            (PLP, _) => { match c {
                1 => {DUMMY_READ_FROM_PC(nes);}
                2 => {increment_s(nes);}
                3 => {pull_p_from_stack(nes);}
            }}
            (JMP, Absolute) => { match c {
                1 => {fetch_lower_address_from_pc(nes); increment_pc(nes);}
                2 => {fetch_upper_address_from_pc(nes); copy_address_to_pc(nes);}
            }}
            (JMP, AbsoluteI) => { match c {
                1 => {fetch_lower_pointer_address_from_pc(nes); increment_pc(nes);}
                2 => {fetch_upper_pointer_address_from_pc(nes); increment_pc(nes);}
                3 => {fetch_lower_address_from_pointer(nes);}
                4 => {fetch_upper_address_from_pointer(nes); copy_address_to_pc(nes);}
            }}
            _ => panic!("Control instruction not implemented"),
        };
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

    let cat = nes.cpu.instruction.category; 
    if cat == R || cat == W || cat == RMW {
        match nes.cpu.instruction.mode {
            Implied | Accumulator => { match c {
                1 => DUMMY_READ_FROM_PC(nes),
            }}
            Immediate => { match c {
                1 => {fetch_immediate_from_pc(nes); increment_pc(nes);}
            }}
            ZeroPage => { match c {
                1 => {fetch_lower_address_from_pc(nes); increment_pc(nes);}
            }}
            ZeroPageX => { match c {
                1 => {fetch_lower_address_from_pc(nes); increment_pc(nes);}
                2 => {DUMMY_READ_FROM_ADDRESS(nes); add_x_to_lower_address(nes);}
            }}
            ZeroPageY => { match c {
                1 => {fetch_lower_address_from_pc(nes); increment_pc(nes);}
                2 => {DUMMY_READ_FROM_ADDRESS(nes); add_y_to_lower_address(nes);}
            }}
            Absolute => { match c {
                1 => {fetch_lower_address_from_pc(nes); increment_pc(nes);}
                2 => {fetch_upper_address_from_pc(nes); increment_pc(nes);}
            }}
            AbsoluteX => { match c {
                1 => {fetch_lower_address_from_pc(nes); increment_pc(nes);}
                2 => {fetch_upper_address_from_pc(nes); add_x_to_lower_address(nes); increment_pc(nes);}
            }}
            AbsoluteY => { match c {
                1 => {fetch_lower_address_from_pc(nes); increment_pc(nes);}
                2 => {fetch_upper_address_from_pc(nes); add_y_to_lower_address(nes); increment_pc(nes);}
            }}
            IndirectX => { match c {
                1 => {fetch_lower_pointer_address_from_pc(nes); increment_pc(nes);}
                2 => {DUMMY_READ_FROM_POINTER(nes); add_x_to_lower_address(nes);}
                3 => {fetch_lower_address_from_pointer(nes); increment_lower_pointer(nes);}
                4 => {fetch_upper_address_from_pointer(nes);}
            }}
            IndirectY => { match c {
                1 => {fetch_lower_pointer_address_from_pc(nes); increment_pc(nes);}
                2 => {fetch_lower_address_from_pointer(nes); increment_lower_pointer(nes);}
                3 => {fetch_upper_address_from_pointer(nes); add_y_to_lower_address(nes);}
            }}
            _ => (),
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




        */


        let c = nes.cpu.instruction_cycle - nes.cpu.instruction.mode.address_resolution_cycles();
        let func = nes.cpu.instruction.name.function();

        match nes.cpu.instruction.mode {
            Accumulator => { match c {
                0 => {nes.cpu.data = nes.cpu.a; func(nes); nes.cpu.a = nes.cpu.data;}
            }}
            Implied => { match c {
                0 => {func(nes);}
            }}
            Absolute | ZeroPage | ZeroPageX | ZeroPageY | IndirectX => { match (m, c) {
                (R,   0) => {read_from_address(nes); func(nes);}
                (W,   0) => {func(nes);}
                (RMW, 0) => read_from_address(nes),
                (RMW, 1) => {write_to_address(nes); func(nes);}
                (RMW, 2) => write_to_address(nes),
                (_,   _) => panic!(),
            }}
            AbsoluteX | AbsoluteY | IndirectY => { match (m, c) {
                (R, 0)   => {read_from_address(nes); add_lower_address_carry_bit_to_upper_address(nes);}
                (R, 1)   => read_from_address(nes),
                (W, 0)   => {DUMMY_READ_FROM_ADDRESS(nes); add_lower_address_carry_bit_to_upper_address(nes);}
                (W, 1)   => read_from_address(nes),
                (RMW, 0) => {DUMMY_READ_FROM_ADDRESS(nes); add_lower_address_carry_bit_to_upper_address(nes);}
                (RMW, 1) => read_from_address(nes),
                (RMW, 2) => {write_to_address(nes); func(nes);}
                (RMW, 3) => write_to_address(nes),
                (_, _) => panic!(),
            }}
        }

    }


    



    

    
    // match instr.mode {
    //     absolute|zeropage|zeropageX|zeropageY|indirectX => {
    //         read => 
    //         rmw => 
    //         write => 
    //     }
    //     absoluteX|absoluteY|indirectY => {
    //         read => 
    //         rmw => 
    //         write => 
    //     }
    // }
    
    /*
    absolute 
    
    read
    read, (write, operate), write
    write
    
    zero page

    read
    read, (write, operate), write
    write

    zero page x/y

    read
    read, (write, operate), write
    write
    
    absolute x/y
    
    (read, fix high byte), optionally re-read if page was crossed
    (read, fix high byte), re-read, (write value, do operation), write new value
    (read, fix high byte), write

    indirect x

    read
    read, (write, operation), write
    write

    indirect y

    (read, fix high byte), optionally re-read if page was crossed
    (read, fix high byte), re-read, (write value, do operation), write new value
    (read, fix high byte), write
    
    */


}
// fn log(nes: &Nes) -> String{

//     // pc, opcode
//     let mut log_line = format!("{pc:04X}  {opc:02X} ", pc=nes.cpu.pc, opc=opcode);

//     // byte2, byte3
//     match instruction.mode.num_bytes() {
//         1 => log_line.push_str("      "),
//         2 => log_line.push_str(&format!("{byte2:02X}    ")),
//         3 => log_line.push_str(&format!("{byte2:02X} {byte3:02X} ")),
//         _ => {},
//     }

//     // opc name
//     log_line.push_str(&format!("{name:>4} ", name=instruction.name));

//     // mode formatting
//     match instruction.mode {
//         Mode::Implied => log_line.push_str("                            "),
//         Mode::Accumulator => log_line.push_str("A                           "),
//         Mode::Immediate => log_line.push_str(&format!("#${byte2:02X}                        ")),
//         Mode::Absolute => {
//             if opcode != 0x4C && opcode != 0x20 {
//                 log_line.push_str(&format!("${instr_addr:04X} = {instr_val:02X}                  "));
//             } else {
//                 log_line.push_str(&format!("${instr_addr:04X}                       "));
//             }
//         },

//         Mode::Relative => log_line.push_str(&format!("${instr_addr:04X}                       ")),
//         Mode::AbsoluteX => log_line.push_str(&format!("${byte3:02X}{byte2:02X},X @ {instr_addr:04X} = {instr_val:02X}         ")),
//         Mode::AbsoluteY => log_line.push_str(&format!("${byte3:02X}{byte2:02X},Y @ {instr_addr:04X} = {instr_val:02X}         ")),
//         Mode::ZeroPage => log_line.push_str(&format!("${byte2:02X} = {instr_val:02X}                    ")),
//         Mode::ZeroPageX => log_line.push_str(&format!("${byte2:02X},X @ {offset:02X} = {instr_val:02X}             ", offset=byte2.wrapping_add(nes.cpu.x))),
//         Mode::ZeroPageY => log_line.push_str(&format!("${byte2:02X},Y @ {offset:02X} = {instr_val:02X}             ", offset=byte2.wrapping_add(nes.cpu.y))),
//         Mode::IndirectX => log_line.push_str(&format!("(${byte2:02X},X) @ {ind_addr:02X} = {instr_addr:04X} = {instr_val:02X}    ", ind_addr=byte2.wrapping_add(nes.cpu.x))),
//         Mode::IndirectY => log_line.push_str(&format!("(${byte2:02X}),Y = {ind_addr:04X} @ {instr_addr:04X} = {instr_val:02X}  ",ind_addr=read_mem_u16_zp(byte2 as u16, nes))),
//         Mode::AbsoluteI => log_line.push_str(&format!("(${byte3:02X}{byte2:02X}) = {instr_addr:04X}              ")),
//     }

//     log_line.push_str(&format!("A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X}", nes.cpu.a, nes.cpu.x, nes.cpu.y, p_to_byte(nes), nes.cpu.s));

//     log_line
// }