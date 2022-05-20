/* 
+------+-------------------+---+--------------------------------------------------------------+
| ACC  | Accumulator       | 1 | operates on the accumulator                                  |
| IMM  | Immediate         | 2 | 2nd byte contains operand                                    |
| ABS  | Absolute          | 3 | 2nd and 3rd bytes (lower, higher) encode address             |
| ABSX | Indexed absolute  | 3 | 2nd and 3rd bytes (lower, higher) encode address, X is added |
| ABSY | Indexed absolute  | 3 | 2nd and 3rd bytes (lower, higher) encode address, Y is added |
| ZPG  | Zero page         | 2 | 2nd byte encodes address                                     |
| ZPGX | Indexed zero page | 2 | 2nd byte encodes address, X is added (mod 2^8)               |
| ZPGY | Indexed zero page | 2 | 2nd byte encodes address, Y is added (mod 2^8)               |
| INDX | Indexed indirect  | 2 | 2nd byte encodes address, X is added (mod 2^8),              |
|      |                   |   | location and neighbour contain indirect address              |
| INDY | Indirect indexed  | 2 | 2nd byte encodes address, Y is added to value in address,    |
|      |                   |   | producing new indirect address                               |
| ---- | Implied           | 1 | address is hard coded into instruction                       |
| ---- | Relative          | 2 | used for conditional branch, 2nd byte is an offset for PC    |
| ---- | Absolute indirect | 3 | used for JMP only                                            |
+------+-------------------+---+--------------------------------------------------------------+
*/


use crate::hw::*;
use crate::mem::*;
use Mode::*;
use Method::*;
use Name::*;
use crate::instr_func;
use once_cell::sync::Lazy;

type OpList = Vec<Vec<fn(&mut Nes)>>;

#[derive(Clone)]
pub struct Instruction { 
    pub name: Name,
    pub mode: Mode,
    pub method: Method,
    pub ops: OpList,
}
// this isn't neccesary 
impl Default for Instruction {
    fn default() -> Instruction {
        Instruction {name: NOP, mode: Implied, method: X,   ops: Vec::with_capacity(8)}
    }
}


#[derive(Copy, Clone)]
pub enum Name {
    // Official opcodes
    LDA, LDX, LDY, 
    STA, STX, STY,
    TAX, TAY, TSX, TXA, TXS, TYA,
    PHA, PHP, 
    PLA, PLP,
    ASL, LSR, 
    ROL, ROR,
    AND, ORA, EOR, BIT,
    ADC, SBC,
    DEC, DEX, DEY, 
    INC, INX, INY,
    CMP, CPX, CPY,
    BCC, BCS, BEQ, BMI, BNE, BPL, BVC, BVS,
    SEC, SED, SEI,
    CLC, CLD, CLI, CLV,
    NOP,
    BRK, JMP, JSR, RTI, RTS,
    // Unofficial opcodes
    ULAS, ULAX, USAX, USHA, USHX, USHY, USHS, UANC, UARR, UASR, UDCP, UISC, 
    URLA, URRA, USBC, USBX, USLO, USRE, UXAA, UJAM, UNOP, UALR, UISB, UAXS,
}


#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Mode { 
    Accumulator,
    Immediate,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
    Implied,
    Relative,
    IndirectX,
    IndirectY,
    AbsoluteI,
}
impl Mode {
    pub fn address_resolution_cycles(&self) -> u8 {
        match self {
            Implied     => 1,   // this isn't quite right? Wait it is? 
            Accumulator => 1,
            Immediate   => 1,
            ZeroPage    => 1,
            ZeroPageX   => 2,
            ZeroPageY   => 2,
            Absolute    => 2,
            AbsoluteX   => 2,
            AbsoluteY   => 2,
            IndirectX   => 4,
            IndirectY   => 3,
            AbsoluteI   => 0,
            Relative    => 0,
        }
    }
}


#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Method {
    R,
    W,
    RMW,
    X,
}



fn initialise_instruction(instr: &mut Instruction) {

    // Match the CPU instruction to its associated function

    let instruction = match instr.name {
        LDA => instr_func::load_a,
        LDX => instr_func::load_x,
        LDY => instr_func::load_y,

        STA => instr_func::store_a,
        STX => instr_func::store_x,
        STY => instr_func::store_y,

        TAX => instr_func::transfer_a_to_x,
        TAY => instr_func::transfer_a_to_y,
        TSX => instr_func::transfer_s_to_x,
        TXA => instr_func::transfer_x_to_a,
        TXS => instr_func::transfer_x_to_s,
        TYA => instr_func::transfer_y_to_a,

        ASL => instr_func::arithmetic_shift_left,
        LSR => instr_func::logical_shift_right,
        ROL => instr_func::rotate_left,
        ROR => instr_func::rotate_right,

        AND => instr_func::and,
        BIT => instr_func::bit,
        EOR => instr_func::xor,
        ORA => instr_func::or,

        ADC => instr_func::add_with_carry,
        SBC => instr_func::subtract_with_carry,

        DEC => instr_func::decrement_memory,
        DEX => instr_func::decrement_x,
        DEY => instr_func::decrement_y,

        INC => instr_func::increment_memory,
        INX => instr_func::increment_x,
        INY => instr_func::increment_y,

        CLC => instr_func::clear_carry_flag,
        CLD => instr_func::clear_decimal_flag,
        CLI => instr_func::clear_interrupt_flag,
        CLV => instr_func::clear_overflow_flag,

        SEC => instr_func::set_carry_flag,
        SED => instr_func::set_decimal_flag,
        SEI => instr_func::set_interrupt_flag,

        NOP => instr_func::nop,
        _ => none
    };

    // Each addressing mode has a different set of cycles
    // These cycles get the value from memory into the CPU so that it can be operated on
    
    let addressing_mode_cycles = match instr.mode {
        Accumulator | Implied => {
            let ops: OpList = vec![
                vec![DUMMY_READ_FROM_PC],
            ];
            ops
        }
        Immediate => {
            let ops: OpList = vec![
                vec![fetch_immediate_from_pc, increment_pc],
            ];
            ops
        }
        ZeroPage => {
            let ops: OpList = vec![
                vec![fetch_lower_address_from_pc, increment_pc],
            ];
            ops
        }
        // need to remember to zero out the address register between instructions
        ZeroPageX => {
            let ops: OpList = vec![
                vec![fetch_lower_address_from_pc, increment_pc],
                vec![DUMMY_READ_FROM_ADDRESS, add_x_to_lower_address],
            ];
            ops
        }
        ZeroPageY => {
            let ops: OpList = vec![
                vec![fetch_lower_address_from_pc, increment_pc],
                vec![DUMMY_READ_FROM_ADDRESS, add_y_to_lower_address],
            ];
            ops
        }
        Absolute => {
            let ops: OpList = vec![
                vec![fetch_lower_address_from_pc, increment_pc],
                vec![fetch_upper_address_from_pc, increment_pc],
            ];
            ops
        }
        AbsoluteX => {
            let ops: OpList = vec![
                vec![fetch_lower_address_from_pc, increment_pc],
                vec![fetch_upper_address_from_pc, add_x_to_lower_address, increment_pc],
                vec![read_from_address, fix_upper_address],
            ];
            ops  // not sure about the dummy read here, could put it here or in the next bit
        }
        AbsoluteY => {
            let ops: OpList = vec![
                vec![fetch_lower_address_from_pc, increment_pc],
                vec![fetch_upper_address_from_pc, add_x_to_lower_address, increment_pc],
                vec![read_from_address, fix_upper_address],
            ];
            ops
        }
        IndirectX => {
            let ops: OpList = vec![
                vec![fetch_lower_pointer_address_from_pc, increment_pc],
                vec![DUMMY_READ_FROM_POINTER, add_x_to_lower_pointer],
                vec![fetch_lower_address_from_pointer],
                vec![fetch_upper_address_from_pointer],
            ];
            ops
        }
        IndirectY => {
            let ops: OpList = vec![
                vec![fetch_lower_pointer_address_from_pc, increment_pc],
                vec![fetch_lower_address_from_pointer],
                vec![fetch_upper_address_from_pointer, add_y_to_lower_address],
                vec![read_from_address, fix_upper_address],
            ];
            ops
        }
        _ => panic!(),
    };

    
    let last_cycles = match instr.mode {
        Absolute | ZeroPage | ZeroPageX | ZeroPageY | IndirectX => {
            match instr.method {
                R => {  
                    let ops: OpList = vec![
                        vec![read_from_address, increment_pc],
                    ];
                    ops
                }
            }
        }

        AbsoluteX | AbsoluteY | IndirectY => {

        }
    }







    // match (instr.name, instr.mode) {
    //     (BRK, _) => {
    //         instr.ops = vec![
    //             vec![DUMMY_READ_FROM_PC, increment_pc],
    //             vec![push_upper_pc_to_stack, decrement_s],
    //             vec![push_lower_pc_to_stack, decrement_s],
    //             vec![push_p_to_stack_with_brk_flag, decrement_s],
    //             vec![fetch_lower_pc_from_interrupt_vector],
    //             vec![fetch_upper_pc_from_interrupt_vector],
    //         ];
    //         return;;
    //     }
    //     (RTI, _) => {
    //         instr.ops = vec![
    //             vec![DUMMY_READ_FROM_PC],
    //             vec![increment_s],
    //             vec![pull_p_from_stack, increment_s],
    //             vec![pull_lower_pc_from_stack, increment_s],
    //             vec![pull_upper_pc_from_stack],
    //         ];
    //         return;;
    //     }
    //     (RTS, _) => {
    //         instr.ops = vec![
    //             vec![DUMMY_READ_FROM_PC],
    //             vec![increment_s],
    //             vec![pull_lower_pc_from_stack, increment_s],
    //             vec![pull_upper_pc_from_stack],
    //             vec![increment_pc],
    //         ];
    //         return;;
    //     }
    //     (JSR, _) => {
    //         instr.ops = vec![
    //             vec![fetch_lower_address_from_pc, increment_pc],
    //             vec![none],
    //             vec![push_upper_pc_to_stack, decrement_s],
    //             vec![push_lower_pc_to_stack, decrement_s],
    //             vec![fetch_upper_address_from_pc, copy_address_to_pc], 
    //         ];
    //         return;;
    //     }
    //     (PHA, _) => {
    //         instr.ops = vec![
    //             vec![DUMMY_READ_FROM_PC],
    //             vec![push_a_to_stack, decrement_s],
    //         ];
    //         return;;
    //     }
    //     (PHP, _) => {
    //         instr.ops = vec![
    //             vec![DUMMY_READ_FROM_PC],
    //             vec![push_p_to_stack, decrement_s],
    //         ];
    //         return;;
    //     }
    //     (PLA, _) => {
    //         instr.ops = vec![
    //             vec![DUMMY_READ_FROM_PC],
    //             vec![increment_s],
    //             vec![pull_a_from_stack],
    //         ];
    //         return;;
    //     }
    //     (PLP, _) => {
    //         instr.ops = vec![
    //             vec![DUMMY_READ_FROM_PC],
    //             vec![increment_s],
    //             vec![pull_p_from_stack],
    //         ];
    //         return;;
    //     }
    //     (JMP, Absolute) => {
    //         instr.ops = vec![
    //             vec![fetch_lower_address_from_pc, increment_pc],
    //             vec![fetch_upper_address_from_pc, copy_address_to_pc],
    //         ];
    //         return;
    //     }
    //     (JMP, AbsoluteI) => {
    //         instr.ops = vec![
    //             vec![fetch_lower_pointer_address_from_pc, increment_pc],
    //             vec![fetch_upper_pointer_address_from_pc, increment_pc],
    //             vec![fetch_lower_address_from_pointer],
    //             vec![fetch_upper_address_from_pointer, copy_address_to_pc]
    //         ];
    //         return;
    //     }
    //     _ => panic!(),
        
    // };

}




const EMPTY: OpList         = Vec::new();
const DUMMY_READ_FROM_PC: fn(&mut Nes)      = read_from_pc;
const DUMMY_READ_FROM_ADDRESS: fn(&mut Nes) = read_from_address;
const DUMMY_READ_FROM_POINTER: fn(&mut Nes) = read_from_pointer;


pub static INSTRUCTIONS: Lazy<[Instruction; 256]> = Lazy::new(|| {

    let mut instrs: [Instruction; 256] = [
        Instruction {name:  BRK, mode: Implied,     method: X,   ops: EMPTY},
        Instruction {name:  ORA, mode: IndirectX,   method: R,   ops: EMPTY},
        Instruction {name: UJAM, mode: Implied,     method: R,   ops: EMPTY},
        Instruction {name: USLO, mode: IndirectX,   method: R,   ops: EMPTY},
        Instruction {name: UNOP, mode: ZeroPage,    method: R,   ops: EMPTY},
        Instruction {name:  ORA, mode: ZeroPage,    method: R,   ops: EMPTY},
        Instruction {name:  ASL, mode: ZeroPage,    method: RMW, ops: EMPTY},
        Instruction {name: USLO, mode: ZeroPage,    method: R,   ops: EMPTY},
        Instruction {name:  PHP, mode: Implied,     method: X,   ops: EMPTY},
        Instruction {name:  ORA, mode: Immediate,   method: R,   ops: EMPTY},
        Instruction {name:  ASL, mode: Accumulator, method: R,   ops: EMPTY},
        Instruction {name: UANC, mode: Immediate,   method: R,   ops: EMPTY},
        Instruction {name: UNOP, mode: Absolute,    method: R,   ops: EMPTY},
        Instruction {name:  ORA, mode: Absolute,    method: R,   ops: EMPTY},
        Instruction {name:  ASL, mode: Absolute,    method: RMW, ops: EMPTY},
        Instruction {name: USLO, mode: Absolute,    method: R,   ops: EMPTY},
        Instruction {name:  BPL, mode: Relative,    method: X,   ops: EMPTY},
        Instruction {name:  ORA, mode: IndirectY,   method: R,   ops: EMPTY},
        Instruction {name: UJAM, mode: Implied,     method: R,   ops: EMPTY},
        Instruction {name: USLO, mode: IndirectY,   method: R,   ops: EMPTY},
        Instruction {name: UNOP, mode: ZeroPageX,   method: R,   ops: EMPTY},
        Instruction {name:  ORA, mode: ZeroPageX,   method: R,   ops: EMPTY},
        Instruction {name:  ASL, mode: ZeroPageX,   method: RMW, ops: EMPTY},
        Instruction {name: USLO, mode: ZeroPageX,   method: R,   ops: EMPTY},
        Instruction {name:  CLC, mode: Implied,     method: R,   ops: EMPTY},
        Instruction {name:  ORA, mode: AbsoluteY,   method: R,   ops: EMPTY},
        Instruction {name: UNOP, mode: Implied,     method: R,   ops: EMPTY},
        Instruction {name: USLO, mode: AbsoluteY,   method: R,   ops: EMPTY},
        Instruction {name: UNOP, mode: AbsoluteX,   method: R,   ops: EMPTY},
        Instruction {name:  ORA, mode: AbsoluteX,   method: R,   ops: EMPTY},
        Instruction {name:  ASL, mode: AbsoluteX,   method: RMW, ops: EMPTY},
        Instruction {name: USLO, mode: AbsoluteX,   method: R,   ops: EMPTY},
        Instruction {name:  JSR, mode: Absolute,    method: R,   ops: EMPTY},
        Instruction {name:  AND, mode: IndirectX,   method: R,   ops: EMPTY},
        Instruction {name: UJAM, mode: Implied,     method: R,   ops: EMPTY},
        Instruction {name: URLA, mode: IndirectX,   method: R,   ops: EMPTY},
        Instruction {name:  BIT, mode: ZeroPage,    method: R,   ops: EMPTY},
        Instruction {name:  AND, mode: ZeroPage,    method: R,   ops: EMPTY},
        Instruction {name:  ROL, mode: ZeroPage,    method: RMW, ops: EMPTY},
        Instruction {name: URLA, mode: ZeroPage,    method: R,   ops: EMPTY},
        Instruction {name:  PLP, mode: Implied,     method: R,   ops: EMPTY},
        Instruction {name:  AND, mode: Immediate,   method: R,   ops: EMPTY},
        Instruction {name:  ROL, mode: Accumulator, method: R,   ops: EMPTY},
        Instruction {name: UANC, mode: Immediate,   method: R,   ops: EMPTY},
        Instruction {name:  BIT, mode: Absolute,    method: R,   ops: EMPTY},
        Instruction {name:  AND, mode: Absolute,    method: R,   ops: EMPTY},
        Instruction {name:  ROL, mode: Absolute,    method: RMW, ops: EMPTY},
        Instruction {name: URLA, mode: Absolute,    method: R,   ops: EMPTY},
        Instruction {name:  BMI, mode: Relative,    method: R,   ops: EMPTY},
        Instruction {name:  AND, mode: IndirectY,   method: R,   ops: EMPTY},
        Instruction {name: UJAM, mode: Implied,     method: R,   ops: EMPTY},
        Instruction {name: URLA, mode: IndirectY,   method: R,   ops: EMPTY},
        Instruction {name: UNOP, mode: ZeroPageX,   method: R,   ops: EMPTY},
        Instruction {name:  AND, mode: ZeroPageX,   method: R,   ops: EMPTY},
        Instruction {name:  ROL, mode: ZeroPageX,   method: RMW, ops: EMPTY},
        Instruction {name: URLA, mode: ZeroPageX,   method: R,   ops: EMPTY},
        Instruction {name:  SEC, mode: Implied,     method: R,   ops: EMPTY},
        Instruction {name:  AND, mode: AbsoluteY,   method: R,   ops: EMPTY},
        Instruction {name: UNOP, mode: Implied,     method: R,   ops: EMPTY},
        Instruction {name: URLA, mode: AbsoluteY,   method: R,   ops: EMPTY},
        Instruction {name: UNOP, mode: AbsoluteX,   method: R,   ops: EMPTY},
        Instruction {name:  AND, mode: AbsoluteX,   method: R,   ops: EMPTY},
        Instruction {name:  ROL, mode: AbsoluteX,   method: RMW, ops: EMPTY},
        Instruction {name: URLA, mode: AbsoluteX,   method: R,   ops: EMPTY},
        Instruction {name:  RTI, mode: Implied,     method: R,   ops: EMPTY},
        Instruction {name:  EOR, mode: IndirectX,   method: R,   ops: EMPTY},
        Instruction {name: UJAM, mode: Implied,     method: R,   ops: EMPTY},
        Instruction {name: USRE, mode: IndirectX,   method: R,   ops: EMPTY},
        Instruction {name: UNOP, mode: ZeroPage,    method: R,   ops: EMPTY},
        Instruction {name:  EOR, mode: ZeroPage,    method: R,   ops: EMPTY},
        Instruction {name:  LSR, mode: ZeroPage,    method: RMW, ops: EMPTY},
        Instruction {name: USRE, mode: ZeroPage,    method: R,   ops: EMPTY},
        Instruction {name:  PHA, mode: Implied,     method: R,   ops: EMPTY},
        Instruction {name:  EOR, mode: Immediate,   method: R,   ops: EMPTY},
        Instruction {name:  LSR, mode: Accumulator, method: R,   ops: EMPTY},
        Instruction {name: UALR, mode: Immediate,   method: R,   ops: EMPTY},
        Instruction {name:  JMP, mode: Absolute,    method: R,   ops: EMPTY},
        Instruction {name:  EOR, mode: Absolute,    method: R,   ops: EMPTY},
        Instruction {name:  LSR, mode: Absolute,    method: RMW, ops: EMPTY},
        Instruction {name: USRE, mode: Absolute,    method: R,   ops: EMPTY},
        Instruction {name:  BVC, mode: Relative,    method: R,   ops: EMPTY},
        Instruction {name:  EOR, mode: IndirectY,   method: R,   ops: EMPTY},
        Instruction {name: UJAM, mode: Implied,     method: R,   ops: EMPTY},
        Instruction {name: USRE, mode: IndirectY,   method: R,   ops: EMPTY},
        Instruction {name: UNOP, mode: ZeroPageX,   method: R,   ops: EMPTY},
        Instruction {name:  EOR, mode: ZeroPageX,   method: R,   ops: EMPTY},
        Instruction {name:  LSR, mode: ZeroPageX,   method: RMW, ops: EMPTY},
        Instruction {name: USRE, mode: ZeroPageX,   method: R,   ops: EMPTY},
        Instruction {name:  CLI, mode: Implied,     method: R,   ops: EMPTY},
        Instruction {name:  EOR, mode: AbsoluteY,   method: R,   ops: EMPTY},
        Instruction {name: UNOP, mode: Implied,     method: R,   ops: EMPTY},
        Instruction {name: USRE, mode: AbsoluteY,   method: R,   ops: EMPTY},
        Instruction {name: UNOP, mode: AbsoluteX,   method: R,   ops: EMPTY},
        Instruction {name:  EOR, mode: AbsoluteX,   method: R,   ops: EMPTY},
        Instruction {name:  LSR, mode: AbsoluteX,   method: RMW, ops: EMPTY},
        Instruction {name: USRE, mode: AbsoluteX,   method: R,   ops: EMPTY},
        Instruction {name:  RTS, mode: Implied,     method: R,   ops: EMPTY},
        Instruction {name:  ADC, mode: IndirectX,   method: R,   ops: EMPTY},
        Instruction {name: UJAM, mode: Implied,     method: R,   ops: EMPTY},
        Instruction {name: URRA, mode: IndirectX,   method: R,   ops: EMPTY},
        Instruction {name: UNOP, mode: ZeroPage,    method: R,   ops: EMPTY},
        Instruction {name:  ADC, mode: ZeroPage,    method: R,   ops: EMPTY},
        Instruction {name:  ROR, mode: ZeroPage,    method: RMW, ops: EMPTY},
        Instruction {name: URRA, mode: ZeroPage,    method: R,   ops: EMPTY},
        Instruction {name:  PLA, mode: Implied,     method: R,   ops: EMPTY},
        Instruction {name:  ADC, mode: Immediate,   method: R,   ops: EMPTY},
        Instruction {name:  ROR, mode: Accumulator, method: R,   ops: EMPTY},
        Instruction {name: UARR, mode: Immediate,   method: R,   ops: EMPTY},
        Instruction {name:  JMP, mode: AbsoluteI,   method: R,   ops: EMPTY},
        Instruction {name:  ADC, mode: Absolute,    method: R,   ops: EMPTY},
        Instruction {name:  ROR, mode: Absolute,    method: RMW, ops: EMPTY},
        Instruction {name: URRA, mode: Absolute,    method: R,   ops: EMPTY},
        Instruction {name:  BVS, mode: Relative,    method: R,   ops: EMPTY},
        Instruction {name:  ADC, mode: IndirectY,   method: R,   ops: EMPTY},
        Instruction {name: UJAM, mode: Implied,     method: R,   ops: EMPTY},
        Instruction {name: URRA, mode: IndirectY,   method: R,   ops: EMPTY},
        Instruction {name: UNOP, mode: ZeroPageX,   method: R,   ops: EMPTY},
        Instruction {name:  ADC, mode: ZeroPageX,   method: R,   ops: EMPTY},
        Instruction {name:  ROR, mode: ZeroPageX,   method: RMW, ops: EMPTY},
        Instruction {name: URRA, mode: ZeroPageX,   method: R,   ops: EMPTY},
        Instruction {name:  SEI, mode: Implied,     method: R,   ops: EMPTY},
        Instruction {name:  ADC, mode: AbsoluteY,   method: R,   ops: EMPTY},
        Instruction {name: UNOP, mode: Implied,     method: R,   ops: EMPTY},
        Instruction {name: URRA, mode: AbsoluteY,   method: R,   ops: EMPTY},
        Instruction {name: UNOP, mode: AbsoluteX,   method: R,   ops: EMPTY},
        Instruction {name:  ADC, mode: AbsoluteX,   method: R,   ops: EMPTY},
        Instruction {name:  ROR, mode: AbsoluteX,   method: RMW, ops: EMPTY},
        Instruction {name: URRA, mode: AbsoluteX,   method: R,   ops: EMPTY},
        Instruction {name: UNOP, mode: Immediate,   method: R,   ops: EMPTY},
        Instruction {name:  STA, mode: IndirectX,   method: W,   ops: EMPTY},
        Instruction {name: UNOP, mode: Immediate,   method: R,   ops: EMPTY},
        Instruction {name: USAX, mode: IndirectX,   method: R,   ops: EMPTY},
        Instruction {name:  STY, mode: ZeroPage,    method: W,   ops: EMPTY},
        Instruction {name:  STA, mode: ZeroPage,    method: W,   ops: EMPTY},
        Instruction {name:  STX, mode: ZeroPage,    method: W,   ops: EMPTY},
        Instruction {name: USAX, mode: ZeroPage,    method: R,   ops: EMPTY},
        Instruction {name:  DEY, mode: Implied,     method: R,   ops: EMPTY},
        Instruction {name: UNOP, mode: Immediate,   method: R,   ops: EMPTY},
        Instruction {name:  TXA, mode: Implied,     method: R,   ops: EMPTY},
        Instruction {name: UXAA, mode: Immediate,   method: R,   ops: EMPTY},
        Instruction {name:  STY, mode: Absolute,    method: W,   ops: EMPTY},
        Instruction {name:  STA, mode: Absolute,    method: W,   ops: EMPTY},
        Instruction {name:  STX, mode: Absolute,    method: W,   ops: EMPTY},
        Instruction {name: USAX, mode: Absolute,    method: R,   ops: EMPTY},
        Instruction {name:  BCC, mode: Relative,    method: R,   ops: EMPTY},
        Instruction {name:  STA, mode: IndirectY,   method: W,   ops: EMPTY},
        Instruction {name: UJAM, mode: Implied,     method: R,   ops: EMPTY},
        Instruction {name: USHA, mode: IndirectY,   method: R,   ops: EMPTY},
        Instruction {name:  STY, mode: ZeroPageX,   method: W,   ops: EMPTY},
        Instruction {name:  STA, mode: ZeroPageX,   method: W,   ops: EMPTY},
        Instruction {name:  STX, mode: ZeroPageY,   method: W,   ops: EMPTY},
        Instruction {name: USAX, mode: ZeroPageY,   method: R,   ops: EMPTY},
        Instruction {name:  TYA, mode: Implied,     method: R,   ops: EMPTY},
        Instruction {name:  STA, mode: AbsoluteY,   method: W,   ops: EMPTY},
        Instruction {name:  TXS, mode: Implied,     method: R,   ops: EMPTY},
        Instruction {name: USHS, mode: AbsoluteY,   method: R,   ops: EMPTY},
        Instruction {name: USHY, mode: AbsoluteX,   method: R,   ops: EMPTY},
        Instruction {name:  STA, mode: AbsoluteX,   method: W,   ops: EMPTY},
        Instruction {name: USHX, mode: AbsoluteY,   method: R,   ops: EMPTY},
        Instruction {name: USHA, mode: AbsoluteY,   method: R,   ops: EMPTY},
        Instruction {name:  LDY, mode: Immediate,   method: R,   ops: EMPTY},
        Instruction {name:  LDA, mode: IndirectX,   method: R,   ops: EMPTY},
        Instruction {name:  LDX, mode: Immediate,   method: R,   ops: EMPTY},
        Instruction {name: ULAX, mode: IndirectX,   method: R,   ops: EMPTY},
        Instruction {name:  LDY, mode: ZeroPage,    method: R,   ops: EMPTY},
        Instruction {name:  LDA, mode: ZeroPage,    method: R,   ops: EMPTY},
        Instruction {name:  LDX, mode: ZeroPage,    method: R,   ops: EMPTY},
        Instruction {name: ULAX, mode: ZeroPage,    method: R,   ops: EMPTY},
        Instruction {name:  TAY, mode: Implied,     method: R,   ops: EMPTY},
        Instruction {name:  LDA, mode: Immediate,   method: R,   ops: EMPTY},
        Instruction {name:  TAX, mode: Implied,     method: R,   ops: EMPTY},
        Instruction {name: ULAX, mode: Immediate,   method: R,   ops: EMPTY},
        Instruction {name:  LDY, mode: Absolute,    method: R,   ops: EMPTY},
        Instruction {name:  LDA, mode: Absolute,    method: R,   ops: EMPTY},
        Instruction {name:  LDX, mode: Absolute,    method: R,   ops: EMPTY},
        Instruction {name: ULAX, mode: Absolute,    method: R,   ops: EMPTY},
        Instruction {name:  BCS, mode: Relative,    method: R,   ops: EMPTY},
        Instruction {name:  LDA, mode: IndirectY,   method: R,   ops: EMPTY},
        Instruction {name: UJAM, mode: Implied,     method: R,   ops: EMPTY},
        Instruction {name: ULAX, mode: IndirectY,   method: R,   ops: EMPTY},
        Instruction {name:  LDY, mode: ZeroPageX,   method: R,   ops: EMPTY},
        Instruction {name:  LDA, mode: ZeroPageX,   method: R,   ops: EMPTY},
        Instruction {name:  LDX, mode: ZeroPageY,   method: R,   ops: EMPTY},
        Instruction {name: ULAX, mode: ZeroPageY,   method: R,   ops: EMPTY},
        Instruction {name:  CLV, mode: Implied,     method: R,   ops: EMPTY},
        Instruction {name:  LDA, mode: AbsoluteY,   method: R,   ops: EMPTY},
        Instruction {name:  TSX, mode: Implied,     method: R,   ops: EMPTY},
        Instruction {name: ULAS, mode: AbsoluteY,   method: R,   ops: EMPTY},
        Instruction {name:  LDY, mode: AbsoluteX,   method: R,   ops: EMPTY},
        Instruction {name:  LDA, mode: AbsoluteX,   method: R,   ops: EMPTY},
        Instruction {name:  LDX, mode: AbsoluteY,   method: R,   ops: EMPTY},
        Instruction {name: ULAX, mode: AbsoluteY,   method: R,   ops: EMPTY},
        Instruction {name:  CPY, mode: Immediate,   method: R,   ops: EMPTY},
        Instruction {name:  CMP, mode: IndirectX,   method: R,   ops: EMPTY},
        Instruction {name: UNOP, mode: Immediate,   method: R,   ops: EMPTY},
        Instruction {name: UDCP, mode: IndirectX,   method: R,   ops: EMPTY},
        Instruction {name:  CPY, mode: ZeroPage,    method: R,   ops: EMPTY},
        Instruction {name:  CMP, mode: ZeroPage,    method: R,   ops: EMPTY},
        Instruction {name:  DEC, mode: ZeroPage,    method: RMW, ops: EMPTY},
        Instruction {name: UDCP, mode: ZeroPage,    method: R,   ops: EMPTY},
        Instruction {name:  INY, mode: Implied,     method: R,   ops: EMPTY},
        Instruction {name:  CMP, mode: Immediate,   method: R,   ops: EMPTY},
        Instruction {name:  DEX, mode: Implied,     method: R,   ops: EMPTY},
        Instruction {name: UAXS, mode: Immediate,   method: R,   ops: EMPTY},
        Instruction {name:  CPY, mode: Absolute,    method: R,   ops: EMPTY},
        Instruction {name:  CMP, mode: Absolute,    method: R,   ops: EMPTY},
        Instruction {name:  DEC, mode: Absolute,    method: RMW, ops: EMPTY},
        Instruction {name: UDCP, mode: Absolute,    method: R,   ops: EMPTY},
        Instruction {name:  BNE, mode: Relative,    method: R,   ops: EMPTY},
        Instruction {name:  CMP, mode: IndirectY,   method: R,   ops: EMPTY},
        Instruction {name: UJAM, mode: Implied,     method: R,   ops: EMPTY},
        Instruction {name: UDCP, mode: IndirectY,   method: R,   ops: EMPTY},
        Instruction {name: UNOP, mode: ZeroPageX,   method: R,   ops: EMPTY},
        Instruction {name:  CMP, mode: ZeroPageX,   method: R,   ops: EMPTY},
        Instruction {name:  DEC, mode: ZeroPageX,   method: RMW, ops: EMPTY},
        Instruction {name: UDCP, mode: ZeroPageX,   method: R,   ops: EMPTY},
        Instruction {name:  CLD, mode: Implied,     method: R,   ops: EMPTY},
        Instruction {name:  CMP, mode: AbsoluteY,   method: R,   ops: EMPTY},
        Instruction {name: UNOP, mode: Implied,     method: R,   ops: EMPTY},
        Instruction {name: UDCP, mode: AbsoluteY,   method: R,   ops: EMPTY},
        Instruction {name: UNOP, mode: AbsoluteX,   method: R,   ops: EMPTY},
        Instruction {name:  CMP, mode: AbsoluteX,   method: R,   ops: EMPTY},
        Instruction {name:  DEC, mode: AbsoluteX,   method: RMW, ops: EMPTY},
        Instruction {name: UDCP, mode: AbsoluteX,   method: R,   ops: EMPTY},
        Instruction {name:  CPX, mode: Immediate,   method: R,   ops: EMPTY},
        Instruction {name:  SBC, mode: IndirectX,   method: R,   ops: EMPTY},
        Instruction {name: UNOP, mode: Immediate,   method: R,   ops: EMPTY},
        Instruction {name: UISB, mode: IndirectX,   method: R,   ops: EMPTY},
        Instruction {name:  CPX, mode: ZeroPage,    method: R,   ops: EMPTY},
        Instruction {name:  SBC, mode: ZeroPage,    method: R,   ops: EMPTY},
        Instruction {name:  INC, mode: ZeroPage,    method: RMW, ops: EMPTY},
        Instruction {name: UISB, mode: ZeroPage,    method: R,   ops: EMPTY},
        Instruction {name:  INX, mode: Implied,     method: R,   ops: EMPTY},
        Instruction {name:  SBC, mode: Immediate,   method: R,   ops: EMPTY},
        Instruction {name:  NOP, mode: Implied,     method: R,   ops: EMPTY},
        Instruction {name: USBC, mode: Immediate,   method: R,   ops: EMPTY},
        Instruction {name:  CPX, mode: Absolute,    method: R,   ops: EMPTY},
        Instruction {name:  SBC, mode: Absolute,    method: R,   ops: EMPTY},
        Instruction {name:  INC, mode: Absolute,    method: RMW, ops: EMPTY},
        Instruction {name: UISB, mode: Absolute,    method: R,   ops: EMPTY},
        Instruction {name:  BEQ, mode: Relative,    method: R,   ops: EMPTY},
        Instruction {name:  SBC, mode: IndirectY,   method: R,   ops: EMPTY},
        Instruction {name: UJAM, mode: Implied,     method: R,   ops: EMPTY},
        Instruction {name: UISB, mode: IndirectY,   method: R,   ops: EMPTY},
        Instruction {name: UNOP, mode: ZeroPageX,   method: R,   ops: EMPTY},
        Instruction {name:  SBC, mode: ZeroPageX,   method: R,   ops: EMPTY},
        Instruction {name:  INC, mode: ZeroPageX,   method: RMW, ops: EMPTY},
        Instruction {name: UISB, mode: ZeroPageX,   method: R,   ops: EMPTY},
        Instruction {name:  SED, mode: Implied,     method: R,   ops: EMPTY},
        Instruction {name:  SBC, mode: AbsoluteY,   method: R,   ops: EMPTY},
        Instruction {name: UNOP, mode: Implied,     method: R,   ops: EMPTY},
        Instruction {name: UISB, mode: AbsoluteY,   method: R,   ops: EMPTY},
        Instruction {name: UNOP, mode: AbsoluteX,   method: R,   ops: EMPTY},
        Instruction {name:  SBC, mode: AbsoluteX,   method: R,   ops: EMPTY},
        Instruction {name:  INC, mode: AbsoluteX,   method: RMW, ops: EMPTY},
        Instruction {name: UISB, mode: AbsoluteX,   method: R,   ops: EMPTY}
    ];

    for instr in instrs.iter_mut() {
        initialise_instruction(instr);
    }

    instrs

});




pub fn read_from_pc(nes: &mut Nes) {
    nes.cpu.data = read_mem(nes.cpu.pc, nes);
}


pub fn copy_address_to_pc(nes: &mut Nes) {
    nes.cpu.pc = nes.cpu.get_address();
}






pub fn fetch_lower_pointer_address_from_pc(nes: &mut Nes) {
    nes.cpu.lower_pointer = read_mem(nes.cpu.pc, nes);
}
pub fn fetch_upper_pointer_address_from_pc(nes: &mut Nes) {
    nes.cpu.upper_pointer = read_mem(nes.cpu.pc, nes);
}

pub fn fetch_lower_address_from_pointer(nes: &mut Nes) {
    nes.cpu.lower_address = read_mem(nes.cpu.get_pointer(), nes);
}
pub fn increment_lower_pointer(nes: &mut Nes) {
    nes.cpu.lower_address = nes.cpu.lower_address.wrapping_add(1);
}
pub fn fetch_upper_address_from_pointer(nes: &mut Nes) {
    nes.cpu.upper_address = read_mem(nes.cpu.get_pointer(), nes);
}




pub fn fetch_lower_address_from_pc(nes: &mut Nes) {
    nes.cpu.lower_address = read_mem(nes.cpu.pc, nes);
}

pub fn fetch_upper_address_from_pc(nes: &mut Nes) {
    nes.cpu.upper_address = read_mem(nes.cpu.pc, nes);
}
pub fn fetch_immediate_from_pc(nes: &mut Nes) {
    nes.cpu.data = read_mem(nes.cpu.pc, nes);
}



pub fn read_from_address(nes: &mut Nes) {
    let addr = nes.cpu.get_address();
    nes.cpu.data = read_mem(addr, nes);
}
pub fn read_from_pointer(nes: &mut Nes) {
    let addr = nes.cpu.get_pointer();
    nes.cpu.data = read_mem(addr, nes);
}

pub fn write_to_address(nes: &mut Nes) {
    let addr = nes.cpu.get_address();
    write_mem(addr, nes.cpu.data, nes);
}

pub fn fetch_lower_pc_from_interrupt_vector(nes: &mut Nes) {
    let lower = read_mem(0xFFFE, nes);
    nes.cpu.set_lower_pc(lower);
}
pub fn fetch_upper_pc_from_interrupt_vector(nes: &mut Nes) {
    let upper = read_mem(0xFFFF, nes);
    nes.cpu.set_upper_pc(upper);
}


pub fn fetch_branch_offset_from_pc(nes: &mut Nes) {
    nes.cpu.branch_offset = read_mem(nes.cpu.pc, nes);
}



    
    






pub fn add_index_to_lower_address_and_set_carry(index: u8, nes: &mut Nes) {
    let (new_val, was_overflow) = nes.cpu.lower_address.overflowing_add(index);
    nes.cpu.lower_address = new_val; 
    nes.cpu.addr_low_carry = was_overflow;
}

pub fn push_to_stack(value: u8, nes: &mut Nes) {
    let stack_addr = 0x0100 + nes.cpu.s as u16;
    write_mem(stack_addr, value, nes);
}

pub fn pull_from_stack(nes: &mut Nes) -> u8 {
    let stack_addr = 0x0100 + nes.cpu.s as u16;
    let value = read_mem(stack_addr, nes);
    value
}

pub fn add_lower_address_carry_bit_to_upper_address(nes: &mut Nes) {
    nes.cpu.upper_address = nes.cpu.upper_address.wrapping_add(nes.cpu.lower_address_carry_out as u8);
}
















pub fn push_lower_pc_to_stack(nes: &mut Nes) {
    push_to_stack(nes.cpu.pc as u8, nes);
}
pub fn push_upper_pc_to_stack(nes: &mut Nes) {
    push_to_stack((nes.cpu.pc >> 8) as u8, nes);    
}
pub fn push_p_to_stack_with_brk_flag(nes: &mut Nes) {
    push_to_stack(nes.cpu.get_p() | 0b0001_0000, nes);
}
pub fn push_p_to_stack(nes: &mut Nes) {
    push_to_stack(nes.cpu.get_p(), nes);
}
pub fn push_a_to_stack(nes: &mut Nes) {
    push_to_stack(nes.cpu.a, nes);
}



pub fn pull_lower_pc_from_stack(nes: &mut Nes) {
    let lower_pc = pull_from_stack(nes);
    nes.cpu.set_lower_pc(lower_pc);
}
pub fn pull_upper_pc_from_stack(nes: &mut Nes) {
    let upper_pc = pull_from_stack(nes);
    nes.cpu.set_upper_pc(upper_pc);
}
pub fn pull_p_from_stack(nes: &mut Nes) {
    let status_reg = pull_from_stack(nes);
    nes.cpu.set_p(status_reg);
}
pub fn pull_a_from_stack(nes: &mut Nes) {
    let a_reg = pull_from_stack(nes);
    nes.cpu.a = a_reg;
}






// Register operations

pub fn increment_pc(nes: &mut Nes) {
    nes.cpu.pc = nes.cpu.pc.wrapping_add(1);
}

pub fn increment_s(nes: &mut Nes) {
    nes.cpu.s = nes.cpu.s.wrapping_add(1);
}
pub fn increment_x(nes: &mut Nes) {
    nes.cpu.x = nes.cpu.x.wrapping_add(1);
}
pub fn increment_y(nes: &mut Nes) {
    nes.cpu.y = nes.cpu.y.wrapping_add(1);
}
pub fn increment_data(nes: &mut Nes) {
    nes.cpu.data = nes.cpu.data.wrapping_add(1);
}


pub fn decrement_s(nes: &mut Nes) {
    nes.cpu.s = nes.cpu.s.wrapping_sub(1);
}
pub fn decrement_x(nes: &mut Nes) {
    nes.cpu.x = nes.cpu.x.wrapping_sub(1);
}
pub fn decrement_y(nes: &mut Nes) {
    nes.cpu.y = nes.cpu.y.wrapping_sub(1);
}
pub fn decrement_data(nes: &mut Nes) {
    nes.cpu.data = nes.cpu.data.wrapping_sub(1);
}

pub fn add_x_to_lower_address(nes: &mut Nes) {
    add_index_to_lower_address_and_set_carry(nes.cpu.x, nes);
}

pub fn add_y_to_lower_address(nes: &mut Nes) {
    add_index_to_lower_address_and_set_carry(nes.cpu.y, nes);
}

pub fn add_x_to_lower_pointer(nes: &mut Nes) {
    nes.cpu.lower_pointer = nes.cpu.lower_pointer.wrapping_add(nes.cpu.x);
}
pub fn add_y_to_lower_pointer(nes: &mut Nes) {
    nes.cpu.lower_pointer = nes.cpu.lower_pointer.wrapping_add(nes.cpu.y);
}




pub fn none(nes: &mut Nes) {}


