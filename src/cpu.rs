use std::io;
use crate::hw::*;
use crate::opc::{*, Mode::*, Method::*};
use crate::mem::*;
use crate::instr_func::*;

use crate::outfile::*;




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


const DUMMY_READ_FROM_PC:      fn(&mut Nes) = read_from_pc;
const DUMMY_READ_FROM_ADDRESS: fn(&mut Nes) = read_from_address;
const DUMMY_READ_FROM_POINTER: fn(&mut Nes) = read_from_pointer;

pub fn step_cpu(nes: &mut Nes) {

    if nes.cpu.instruction_cycle == 0 {
        let opcode = read_mem(nes.cpu.get_address(), nes);
        nes.cpu.instruction = INSTRUCTIONS[opcode as usize];
        return;
    }
    



    let instruction = match nes.cpu.instruction.name {
        LDA => load_a,
        LDX => load_x,
        LDY => load_y,

        STA => store_a,
        STX => store_x,
        STY => store_y,

        TAX => transfer_a_to_x,
        TAY => transfer_a_to_y,
        TSX => transfer_s_to_x,
        TXA => transfer_x_to_a,
        TXS => transfer_x_to_s,
        TYA => transfer_y_to_a,

        ASL => arithmetic_shift_left,
        LSR => logical_shift_right,
        ROL => rotate_left,
        ROR => rotate_right,

        AND => and,
        BIT => bit,
        EOR => xor,
        ORA => or,

        ADC => add_with_carry,
        SBC => subtract_with_carry,

        DEC => decrement_memory,
        DEX => decrement_x,
        DEY => decrement_y,

        INC => increment_memory,
        INX => increment_x,
        INY => increment_y,

        CLC => clear_carry_flag,
        CLD => clear_decimal_flag,
        CLI => clear_interrupt_flag,
        CLV => clear_overflow_flag,

        SEC => set_carry_flag,
        SED => set_decimal_flag,
        SEI => set_interrupt_flag,

        NOP => nop,
        _ => none
    };





    let c = nes.cpu.instruction_cycle;

    match nes.cpu.instruction.mode {

        Implied | Accumulator => {
            match c {
                1 => DUMMY_READ_FROM_PC(nes),
            }
        }
        Immediate => {
            match c {
                1 => {fetch_immediate_from_pc(nes); increment_pc(nes);}
            }
        }
        ZeroPage => {
            match c {
                1 => {fetch_lower_address_from_pc(nes); increment_pc(nes);}
            }
        }
        ZeroPageX => {
            match c {
                1 => {fetch_lower_address_from_pc(nes); increment_pc(nes);}
                2 => {DUMMY_READ_FROM_ADDRESS(nes); add_x_to_lower_address(nes);}

            }
        }
        ZeroPageY => {
            match c {
                1 => {fetch_lower_address_from_pc(nes); increment_pc(nes);}
                2 => {DUMMY_READ_FROM_ADDRESS(nes); add_y_to_lower_address(nes);}

            }
        }
        Absolute => {
            match c {
                1 => {fetch_lower_address_from_pc(nes); increment_pc(nes);}
                2 => {fetch_upper_address_from_pc(nes); increment_pc(nes);}

            }
        }
        AbsoluteX => {
            match c {
                1 => {fetch_lower_address_from_pc(nes); increment_pc(nes);}
                2 => {fetch_upper_address_from_pc(nes); add_x_to_lower_address(nes); increment_pc(nes);}
                // optional read fix byte stuff
            }
        }
        AbsoluteY => {
            match c {
                1 => {fetch_lower_address_from_pc(nes); increment_pc(nes);}
                2 => {fetch_upper_address_from_pc(nes); add_y_to_lower_address(nes); increment_pc(nes);}
                // optional read fix byte stuff
            }
        }
        
        IndirectX => {
            match c {
                1 => {fetch_lower_pointer_address_from_pc(nes); increment_pc(nes);}
                2 => {DUMMY_READ_FROM_POINTER(nes); add_x_to_lower_address(nes);}
                3 => {fetch_lower_address_from_pointer(nes); increment_lower_pointer(nes);}
                4 => {fetch_upper_address_from_pointer(nes);}
            }
        }

        IndirectY => {
            match c {
                1 => {fetch_lower_pointer_address_from_pc(nes); increment_pc(nes);}
                2 => {fetch_lower_address_from_pointer(nes); increment_lower_pointer(nes);}
                3 => {fetch_upper_address_from_pointer(nes); add_y_to_lower_address(nes);}
                // optional fix byte stuff here
            }
        }

    }

    let m = nes.cpu.instruction.method;

    let c = nes.cpu.instruction_cycle - nes.cpu.instruction.mode.address_resolution_cycles();

    match nes.cpu.instruction.mode {
        Absolute | ZeroPage | ZeroPageX | ZeroPageY | IndirectX => {
            match (m, c) {
                (R,   0) => {read_from_address(nes); instruction(nes);}
                (W,   0) => {instruction(nes);}
                (RMW, 0) => read_from_address(nes),
                (RMW, 1) => {write_to_address(nes); instruction(nes);}
                (RMW, 2) => write_to_address(nes),
                (_,   _) => panic!(),
            }
        }
        AbsoluteX | AbsoluteY | IndirectY => {
            match (m, c) {
                (R, 0)   => {read_from_address(nes); add_lower_address_carry_bit_to_upper_address(nes);}
                (R, 1)   => read_from_address(nes),
                (W, 0)   => {DUMMY_READ_FROM_ADDRESS(nes); add_lower_address_carry_bit_to_upper_address(nes);}
                (W, 1)   => read_from_address(nes),
                (RMW, 0) => {DUMMY_READ_FROM_ADDRESS(nes); add_lower_address_carry_bit_to_upper_address(nes);}
                (RMW, 1) => read_from_address(nes),
                (RMW, 2) => {write_to_address(nes); instruction(nes);}
                (RMW, 3) => write_to_address(nes),
                (_, _) => panic!(),
            }
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
