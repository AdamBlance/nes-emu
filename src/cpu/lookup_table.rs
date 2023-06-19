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

use Mode::*;
use Category::*;
use Name::*;

use super::operation_funcs::*;
use crate::nes::Nes;

#[derive(Copy, Clone)]
pub struct Instruction { 
    pub name: Name,
    pub mode: Mode,
    pub category: Category,
    // pub cycles: u8,
    pub is_unofficial: bool,
    pub operation: fn(&mut Nes),
}

impl Default for Instruction {
    fn default() -> Self {
        Instruction {name: NOP, mode: Implied, category: NoOp, is_unofficial: false, operation: none}    
    }
}

impl Instruction {
    pub fn does_interrupt_poll_early(&self) -> bool {
        self.category == Register || self.mode == Immediate || self.name == NOP || self.name == PLP
    }
    pub fn number_of_operands(&self) -> u8 {
        match self.mode {
            Accumulator | Implied => 0,
            Absolute | AbsoluteX | AbsoluteY | AbsoluteI => 2,
            _ => 1,
        }
    }
}


#[derive(Debug, Copy, Clone, PartialEq)]
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
    LAS, LAX, SAX, SHA, SHX, SHY, SHS, ANC, ARR, ASR, DCP, ISC, 
    RLA, RRA, SBX, SLO, SRE, XAA, JAM, ALR, ISB, AXS,
}

impl Default for Name {
    fn default() -> Name {NOP}
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
    pub fn address_resolution_cycles(self) -> i8 {
        match self {
            Immediate   => 1,
            ZeroPage    => 1,
            ZeroPageX   => 2,
            ZeroPageY   => 2,
            Absolute    => 2,
            AbsoluteX   => 2,
            AbsoluteY   => 2,
            IndirectX   => 4,
            IndirectY   => 3,
            _           => 0,
        }
    }

}

impl Default for Mode {
    fn default() -> Mode {Accumulator}
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Category {
    Control,
    Register,
    Branch,
    Read,
    ReadModifyWrite,
    Write,
    NoOp,
    StatusRegister,
    Unimplemented,
}

// Maybe there doesn't need to be so many addressing modes and categories
// Address modes are really important for some instructions


impl Default for Category {
    fn default() -> Category {Read}
}
// (ORA,.*operation: )none

pub static INSTRUCTIONS: [Instruction; 256] = [
    // 0
    Instruction {name:  BRK, mode: Implied,     category: Control, is_unofficial: false, operation: none},
    Instruction {name:  ORA, mode: IndirectX,   category: Read, is_unofficial: false, operation: or},
    Instruction {name: JAM, mode: Implied,     category: Unimplemented, is_unofficial: true, operation: none},
    Instruction {name: SLO, mode: IndirectX,   category: Unimplemented, is_unofficial: true, operation: none},
    Instruction {name: NOP, mode: ZeroPage,    category: NoOp, is_unofficial: true, operation: none},
    Instruction {name:  ORA, mode: ZeroPage,    category: Read, is_unofficial: false, operation: or},
    Instruction {name:  ASL, mode: ZeroPage,    category: ReadModifyWrite, is_unofficial: false, operation: arithmetic_shift_left},
    Instruction {name: SLO, mode: ZeroPage,    category: Unimplemented, is_unofficial: true, operation: none},
    Instruction {name:  PHP, mode: Implied,     category: Control, is_unofficial: false, operation: none},
    Instruction {name:  ORA, mode: Immediate,   category: Read, is_unofficial: false, operation: or},
    Instruction {name:  ASL, mode: Accumulator, category: Register, is_unofficial: false, operation: arithmetic_shift_left},
    Instruction {name: ANC, mode: Immediate,   category: Unimplemented, is_unofficial: true, operation: none},
    Instruction {name: NOP, mode: Absolute,    category: NoOp, is_unofficial: true, operation: none},
    Instruction {name:  ORA, mode: Absolute,    category: Read, is_unofficial: false, operation: or},
    Instruction {name:  ASL, mode: Absolute,    category: ReadModifyWrite, is_unofficial: false, operation: arithmetic_shift_left},
    Instruction {name: SLO, mode: Absolute,    category: Unimplemented, is_unofficial: true, operation: none},
    // 1
    Instruction {name:  BPL, mode: Relative,    category: Branch, is_unofficial: false, operation: none},
    Instruction {name:  ORA, mode: IndirectY,   category: Read, is_unofficial: false, operation: or},
    Instruction {name: JAM, mode: Implied,     category: Unimplemented, is_unofficial: true, operation: none},
    Instruction {name: SLO, mode: IndirectY,   category: Unimplemented, is_unofficial: true, operation: none},
    Instruction {name: NOP, mode: ZeroPageX,   category: NoOp, is_unofficial: true, operation: none},
    Instruction {name:  ORA, mode: ZeroPageX,   category: Read, is_unofficial: false, operation: or},
    Instruction {name:  ASL, mode: ZeroPageX,   category: ReadModifyWrite, is_unofficial: false, operation: arithmetic_shift_left},
    Instruction {name: SLO, mode: ZeroPageX,   category: Unimplemented, is_unofficial: true, operation: none},
    Instruction {name:  CLC, mode: Implied,     category: StatusRegister, is_unofficial: false, operation: clear_carry_flag},
    Instruction {name:  ORA, mode: AbsoluteY,   category: Read, is_unofficial: false, operation: or},
    Instruction {name: NOP, mode: Implied,     category: NoOp, is_unofficial: true, operation: none},
    Instruction {name: SLO, mode: AbsoluteY,   category: Unimplemented, is_unofficial: true, operation: none},
    Instruction {name: NOP, mode: AbsoluteX,   category: NoOp, is_unofficial: true, operation: none},
    Instruction {name:  ORA, mode: AbsoluteX,   category: Read, is_unofficial: false, operation: or},
    Instruction {name:  ASL, mode: AbsoluteX,   category: ReadModifyWrite, is_unofficial: false, operation: arithmetic_shift_left},
    Instruction {name: SLO, mode: AbsoluteX,   category: Unimplemented, is_unofficial: true, operation: none},
    // 2
    Instruction {name:  JSR, mode: Absolute,    category: Control, is_unofficial: false, operation: none},
    Instruction {name:  AND, mode: IndirectX,   category: Read, is_unofficial: false, operation: and},
    Instruction {name: JAM, mode: Implied,     category: Unimplemented, is_unofficial: true, operation: none},
    Instruction {name: RLA, mode: IndirectX,   category: Unimplemented, is_unofficial: true, operation: none},
    Instruction {name:  BIT, mode: ZeroPage,    category: Read, is_unofficial: false, operation: bit},
    Instruction {name:  AND, mode: ZeroPage,    category: Read, is_unofficial: false, operation: and},
    Instruction {name:  ROL, mode: ZeroPage,    category: ReadModifyWrite, is_unofficial: false, operation: rotate_left},
    Instruction {name: RLA, mode: ZeroPage,    category: Unimplemented, is_unofficial: true, operation: none},
    Instruction {name:  PLP, mode: Implied,     category: Control, is_unofficial: false, operation: none},
    Instruction {name:  AND, mode: Immediate,   category: Read, is_unofficial: false, operation: and},
    Instruction {name:  ROL, mode: Accumulator, category: Register, is_unofficial: false, operation: rotate_left},
    Instruction {name: ANC, mode: Immediate,   category: Unimplemented, is_unofficial: true, operation: none},
    Instruction {name:  BIT, mode: Absolute,    category: Read, is_unofficial: false, operation: bit},
    Instruction {name:  AND, mode: Absolute,    category: Read, is_unofficial: false, operation: and},
    Instruction {name:  ROL, mode: Absolute,    category: ReadModifyWrite, is_unofficial: false, operation: rotate_left},
    Instruction {name: RLA, mode: Absolute,    category: Unimplemented, is_unofficial: true, operation: none},
    // 3
    Instruction {name:  BMI, mode: Relative,    category: Branch, is_unofficial: false, operation: none},
    Instruction {name:  AND, mode: IndirectY,   category: Read, is_unofficial: false, operation: and},
    Instruction {name: JAM, mode: Implied,     category: Unimplemented, is_unofficial: true, operation: none},
    Instruction {name: RLA, mode: IndirectY,   category: Unimplemented, is_unofficial: true, operation: none},
    Instruction {name: NOP, mode: ZeroPageX,   category: NoOp, is_unofficial: true, operation: none},
    Instruction {name:  AND, mode: ZeroPageX,   category: Read, is_unofficial: false, operation: and},
    Instruction {name:  ROL, mode: ZeroPageX,   category: ReadModifyWrite, is_unofficial: false, operation: rotate_left},
    Instruction {name: RLA, mode: ZeroPageX,   category: Unimplemented, is_unofficial: true, operation: none},
    Instruction {name:  SEC, mode: Implied,     category: StatusRegister, is_unofficial: false, operation: set_carry_flag},
    Instruction {name:  AND, mode: AbsoluteY,   category: Read, is_unofficial: false, operation: and},
    Instruction {name: NOP, mode: Implied,     category: NoOp, is_unofficial: true, operation: none},
    Instruction {name: RLA, mode: AbsoluteY,   category: Unimplemented, is_unofficial: true, operation: none},
    Instruction {name: NOP, mode: AbsoluteX,   category: NoOp, is_unofficial: true, operation: none},
    Instruction {name:  AND, mode: AbsoluteX,   category: Read, is_unofficial: false, operation: and},
    Instruction {name:  ROL, mode: AbsoluteX,   category: ReadModifyWrite, is_unofficial: false, operation: rotate_left},
    Instruction {name: RLA, mode: AbsoluteX,   category: Unimplemented, is_unofficial: true, operation: none},
    // 4
    Instruction {name:  RTI, mode: Implied,     category: Control, is_unofficial: false, operation: none},
    Instruction {name:  EOR, mode: IndirectX,   category: Read, is_unofficial: false, operation: xor},
    Instruction {name: JAM, mode: Implied,     category: Unimplemented, is_unofficial: true, operation: none},
    Instruction {name: SRE, mode: IndirectX,   category: Unimplemented, is_unofficial: true, operation: none},
    Instruction {name: NOP, mode: ZeroPage,    category: NoOp, is_unofficial: true, operation: none},
    Instruction {name:  EOR, mode: ZeroPage,    category: Read, is_unofficial: false, operation: xor},
    Instruction {name:  LSR, mode: ZeroPage,    category: ReadModifyWrite, is_unofficial: false, operation: logical_shift_right},
    Instruction {name: SRE, mode: ZeroPage,    category: Unimplemented, is_unofficial: true, operation: none},
    Instruction {name:  PHA, mode: Implied,     category: Control, is_unofficial: false, operation: none},
    Instruction {name:  EOR, mode: Immediate,   category: Read, is_unofficial: false, operation: xor},
    Instruction {name:  LSR, mode: Accumulator, category: Register, is_unofficial: false, operation: logical_shift_right},
    Instruction {name: ALR, mode: Immediate,   category: Unimplemented, is_unofficial: true, operation: none},
    Instruction {name:  JMP, mode: Absolute,    category: Control, is_unofficial: false, operation: none},
    Instruction {name:  EOR, mode: Absolute,    category: Read, is_unofficial: false, operation: xor},
    Instruction {name:  LSR, mode: Absolute,    category: ReadModifyWrite, is_unofficial: false, operation: logical_shift_right},
    Instruction {name: SRE, mode: Absolute,    category: Unimplemented, is_unofficial: true, operation: none},
    // 5
    Instruction {name:  BVC, mode: Relative,    category: Branch, is_unofficial: false, operation: none},
    Instruction {name:  EOR, mode: IndirectY,   category: Read, is_unofficial: false, operation: xor},
    Instruction {name: JAM, mode: Implied,     category: Unimplemented, is_unofficial: true, operation: none},
    Instruction {name: SRE, mode: IndirectY,   category: Unimplemented, is_unofficial: true, operation: none},
    Instruction {name: NOP, mode: ZeroPageX,   category: NoOp, is_unofficial: true, operation: none},
    Instruction {name:  EOR, mode: ZeroPageX,   category: Read, is_unofficial: false, operation: xor},
    Instruction {name:  LSR, mode: ZeroPageX,   category: ReadModifyWrite, is_unofficial: false, operation: logical_shift_right},
    Instruction {name: SRE, mode: ZeroPageX,   category: Unimplemented, is_unofficial: true, operation: none},
    Instruction {name:  CLI, mode: Implied,     category: StatusRegister, is_unofficial: false, operation: clear_interrupt_flag},
    Instruction {name:  EOR, mode: AbsoluteY,   category: Read, is_unofficial: false, operation: xor},
    Instruction {name: NOP, mode: Implied,     category: NoOp, is_unofficial: true, operation: none},
    Instruction {name: SRE, mode: AbsoluteY,   category: Unimplemented, is_unofficial: true, operation: none},
    Instruction {name: NOP, mode: AbsoluteX,   category: NoOp, is_unofficial: true, operation: none},
    Instruction {name:  EOR, mode: AbsoluteX,   category: Read, is_unofficial: false, operation: xor},
    Instruction {name:  LSR, mode: AbsoluteX,   category: ReadModifyWrite, is_unofficial: false, operation: logical_shift_right},
    Instruction {name: SRE, mode: AbsoluteX,   category: Unimplemented, is_unofficial: true, operation: none},
    // 6
    Instruction {name:  RTS, mode: Implied,     category: Control, is_unofficial: false, operation: none},
    Instruction {name:  ADC, mode: IndirectX,   category: Read, is_unofficial: false, operation: add_with_carry},
    Instruction {name: JAM, mode: Implied,     category: Unimplemented, is_unofficial: true, operation: none},
    Instruction {name: RRA, mode: IndirectX,   category: Unimplemented, is_unofficial: true, operation: none},
    Instruction {name: NOP, mode: ZeroPage,    category: NoOp, is_unofficial: true, operation: none},
    Instruction {name:  ADC, mode: ZeroPage,    category: Read, is_unofficial: false, operation: add_with_carry},
    Instruction {name:  ROR, mode: ZeroPage,    category: ReadModifyWrite, is_unofficial: false, operation: rotate_right},
    Instruction {name: RRA, mode: ZeroPage,    category: Unimplemented, is_unofficial: true, operation: none},
    Instruction {name:  PLA, mode: Implied,     category: Control, is_unofficial: false, operation: none},
    Instruction {name:  ADC, mode: Immediate,   category: Read, is_unofficial: false, operation: add_with_carry},
    Instruction {name:  ROR, mode: Accumulator, category: Register, is_unofficial: false, operation: rotate_right},
    Instruction {name: ARR, mode: Immediate,   category: Unimplemented, is_unofficial: true, operation: none},
    Instruction {name:  JMP, mode: AbsoluteI,   category: Control, is_unofficial: false, operation: none},
    Instruction {name:  ADC, mode: Absolute,    category: Read, is_unofficial: false, operation: add_with_carry},
    Instruction {name:  ROR, mode: Absolute,    category: ReadModifyWrite, is_unofficial: false, operation: rotate_right},
    Instruction {name: RRA, mode: Absolute,    category: Unimplemented, is_unofficial: true, operation: none},
    // 7
    Instruction {name:  BVS, mode: Relative,    category: Branch, is_unofficial: false, operation: none},
    Instruction {name:  ADC, mode: IndirectY,   category: Read, is_unofficial: false, operation: add_with_carry},
    Instruction {name: JAM, mode: Implied,     category: Unimplemented, is_unofficial: true, operation: none},
    Instruction {name: RRA, mode: IndirectY,   category: Unimplemented, is_unofficial: true, operation: none},
    Instruction {name: NOP, mode: ZeroPageX,   category: NoOp, is_unofficial: true, operation: none},
    Instruction {name:  ADC, mode: ZeroPageX,   category: Read, is_unofficial: false, operation: add_with_carry},
    Instruction {name:  ROR, mode: ZeroPageX,   category: ReadModifyWrite, is_unofficial: false, operation: rotate_right},
    Instruction {name: RRA, mode: ZeroPageX,   category: Unimplemented, is_unofficial: true, operation: none},
    Instruction {name:  SEI, mode: Implied,     category: StatusRegister, is_unofficial: false, operation: set_interrupt_inhibit_flag},
    Instruction {name:  ADC, mode: AbsoluteY,   category: Read, is_unofficial: false, operation: add_with_carry},
    Instruction {name: NOP, mode: Implied,     category: NoOp, is_unofficial: true, operation: none},
    Instruction {name: RRA, mode: AbsoluteY,   category: Unimplemented, is_unofficial: true, operation: none},
    Instruction {name: NOP, mode: AbsoluteX,   category: NoOp, is_unofficial: true, operation: none},
    Instruction {name:  ADC, mode: AbsoluteX,   category: Read, is_unofficial: false, operation: add_with_carry},
    Instruction {name:  ROR, mode: AbsoluteX,   category: ReadModifyWrite, is_unofficial: false, operation: rotate_right},
    Instruction {name: RRA, mode: AbsoluteX,   category: Unimplemented, is_unofficial: true, operation: none},
    // 8
    Instruction {name: NOP, mode: Immediate,   category: NoOp, is_unofficial: true, operation: none},
    Instruction {name:  STA, mode: IndirectX,   category: Write, is_unofficial: false, operation: store_a},
    Instruction {name: NOP, mode: Immediate,   category: NoOp, is_unofficial: true, operation: none},
    Instruction {name: SAX, mode: IndirectX,   category: Unimplemented, is_unofficial: true, operation: none},
    Instruction {name:  STY, mode: ZeroPage,    category: Write, is_unofficial: false, operation: store_y},
    Instruction {name:  STA, mode: ZeroPage,    category: Write, is_unofficial: false, operation: store_a},
    Instruction {name:  STX, mode: ZeroPage,    category: Write, is_unofficial: false, operation: store_x},
    Instruction {name: SAX, mode: ZeroPage,    category: Unimplemented, is_unofficial: true, operation: none},
    Instruction {name:  DEY, mode: Implied,     category: Register, is_unofficial: false, operation: decrement_y},
    Instruction {name: NOP, mode: Immediate,   category: NoOp, is_unofficial: true, operation: none},
    Instruction {name:  TXA, mode: Implied,     category: Register, is_unofficial: false, operation: transfer_x_to_a},
    Instruction {name: XAA, mode: Immediate,   category: Unimplemented, is_unofficial: true, operation: none},
    Instruction {name:  STY, mode: Absolute,    category: Write, is_unofficial: false, operation: store_y},
    Instruction {name:  STA, mode: Absolute,    category: Write, is_unofficial: false, operation: store_a},
    Instruction {name:  STX, mode: Absolute,    category: Write, is_unofficial: false, operation: store_x},
    Instruction {name: SAX, mode: Absolute,    category: Unimplemented, is_unofficial: true, operation: none},
    // 9
    Instruction {name:  BCC, mode: Relative,    category: Branch, is_unofficial: false, operation: none},
    Instruction {name:  STA, mode: IndirectY,   category: Write, is_unofficial: false, operation: store_a},
    Instruction {name: JAM, mode: Implied,     category: Unimplemented, is_unofficial: true, operation: none},
    Instruction {name: SHA, mode: IndirectY,   category: Unimplemented, is_unofficial: true, operation: none},
    Instruction {name:  STY, mode: ZeroPageX,   category: Write, is_unofficial: false, operation: store_y},
    Instruction {name:  STA, mode: ZeroPageX,   category: Write, is_unofficial: false, operation: store_a},
    Instruction {name:  STX, mode: ZeroPageY,   category: Write, is_unofficial: false, operation: store_x},
    Instruction {name: SAX, mode: ZeroPageY,   category: Unimplemented, is_unofficial: true, operation: none},
    Instruction {name:  TYA, mode: Implied,     category: Register, is_unofficial: false, operation: transfer_y_to_a},
    Instruction {name:  STA, mode: AbsoluteY,   category: Write, is_unofficial: false, operation: store_a},
    Instruction {name:  TXS, mode: Implied,     category: Register, is_unofficial: false, operation: transfer_x_to_s},
    Instruction {name: SHS, mode: AbsoluteY,   category: Unimplemented, is_unofficial: true, operation: none},
    Instruction {name: SHY, mode: AbsoluteX,   category: Unimplemented, is_unofficial: true, operation: none},
    Instruction {name:  STA, mode: AbsoluteX,   category: Write, is_unofficial: false, operation: store_a},
    Instruction {name: SHX, mode: AbsoluteY,   category: Unimplemented, is_unofficial: true, operation: none},
    Instruction {name: SHA, mode: AbsoluteY,   category: Unimplemented, is_unofficial: true, operation: none},
    // A
    Instruction {name:  LDY, mode: Immediate,   category: Read, is_unofficial: false, operation: load_y},
    Instruction {name:  LDA, mode: IndirectX,   category: Read, is_unofficial: false, operation: load_a},
    Instruction {name:  LDX, mode: Immediate,   category: Read, is_unofficial: false, operation: load_x},
    Instruction {name: LAX, mode: IndirectX,   category: Unimplemented, is_unofficial: true, operation: none},
    Instruction {name:  LDY, mode: ZeroPage,    category: Read, is_unofficial: false, operation: load_y},
    Instruction {name:  LDA, mode: ZeroPage,    category: Read, is_unofficial: false, operation: load_a},
    Instruction {name:  LDX, mode: ZeroPage,    category: Read, is_unofficial: false, operation: load_x},
    Instruction {name: LAX, mode: ZeroPage,    category: Unimplemented, is_unofficial: true, operation: none},
    Instruction {name:  TAY, mode: Implied,     category: Register, is_unofficial: false, operation: transfer_a_to_y},
    Instruction {name:  LDA, mode: Immediate,   category: Read, is_unofficial: false, operation: load_a},
    Instruction {name:  TAX, mode: Implied,     category: Register, is_unofficial: false, operation: transfer_a_to_x},
    Instruction {name: LAX, mode: Immediate,   category: Unimplemented, is_unofficial: true, operation: none},
    Instruction {name:  LDY, mode: Absolute,    category: Read, is_unofficial: false, operation: load_y},
    Instruction {name:  LDA, mode: Absolute,    category: Read, is_unofficial: false, operation: load_a},
    Instruction {name:  LDX, mode: Absolute,    category: Read, is_unofficial: false, operation: load_x},
    Instruction {name: LAX, mode: Absolute,    category: Unimplemented, is_unofficial: true, operation: none},
    // B
    Instruction {name:  BCS, mode: Relative,    category: Branch, is_unofficial: false, operation: none},
    Instruction {name:  LDA, mode: IndirectY,   category: Read, is_unofficial: false, operation: load_a},
    Instruction {name: JAM, mode: Implied,     category: Unimplemented, is_unofficial: true, operation: none},
    Instruction {name: LAX, mode: IndirectY,   category: Unimplemented, is_unofficial: true, operation: none},
    Instruction {name:  LDY, mode: ZeroPageX,   category: Read, is_unofficial: false, operation: load_y},
    Instruction {name:  LDA, mode: ZeroPageX,   category: Read, is_unofficial: false, operation: load_a},
    Instruction {name:  LDX, mode: ZeroPageY,   category: Read, is_unofficial: false, operation: load_x},
    Instruction {name: LAX, mode: ZeroPageY,   category: Unimplemented, is_unofficial: true, operation: none},
    Instruction {name:  CLV, mode: Implied,     category: StatusRegister, is_unofficial: false, operation: clear_overflow_flag},
    Instruction {name:  LDA, mode: AbsoluteY,   category: Read, is_unofficial: false, operation: load_a},
    Instruction {name:  TSX, mode: Implied,     category: Register, is_unofficial: false, operation: transfer_s_to_x},
    Instruction {name: LAS, mode: AbsoluteY,   category: Unimplemented, is_unofficial: true, operation: none},
    Instruction {name:  LDY, mode: AbsoluteX,   category: Read, is_unofficial: false, operation: load_y},
    Instruction {name:  LDA, mode: AbsoluteX,   category: Read, is_unofficial: false, operation: load_a},
    Instruction {name:  LDX, mode: AbsoluteY,   category: Read, is_unofficial: false, operation: load_x},
    Instruction {name: LAX, mode: AbsoluteY,   category: Unimplemented, is_unofficial: true, operation: none},
    // C
    Instruction {name:  CPY, mode: Immediate,   category: Read, is_unofficial: false, operation: compare_memory_with_y},
    Instruction {name:  CMP, mode: IndirectX,   category: Read, is_unofficial: false, operation: compare_memory_with_a},
    Instruction {name: NOP, mode: Immediate,   category: NoOp, is_unofficial: true, operation: none},
    Instruction {name: DCP, mode: IndirectX,   category: Unimplemented, is_unofficial: true, operation: none},
    Instruction {name:  CPY, mode: ZeroPage,    category: Read, is_unofficial: false, operation: compare_memory_with_y},
    Instruction {name:  CMP, mode: ZeroPage,    category: Read, is_unofficial: false, operation: compare_memory_with_a},
    Instruction {name:  DEC, mode: ZeroPage,    category: ReadModifyWrite, is_unofficial: false, operation: decrement_memory},
    Instruction {name: DCP, mode: ZeroPage,    category: Unimplemented, is_unofficial: true, operation: none},
    Instruction {name:  INY, mode: Implied,     category: Register, is_unofficial: false, operation: increment_y},
    Instruction {name:  CMP, mode: Immediate,   category: Read, is_unofficial: false, operation: compare_memory_with_a},
    Instruction {name:  DEX, mode: Implied,     category: Register, is_unofficial: false, operation: decrement_x},
    Instruction {name: AXS, mode: Immediate,   category: Unimplemented, is_unofficial: true, operation: none},
    Instruction {name:  CPY, mode: Absolute,    category: Read, is_unofficial: false, operation: compare_memory_with_y},
    Instruction {name:  CMP, mode: Absolute,    category: Read, is_unofficial: false, operation: compare_memory_with_a},
    Instruction {name:  DEC, mode: Absolute,    category: ReadModifyWrite, is_unofficial: false, operation: decrement_memory},
    Instruction {name: DCP, mode: Absolute,    category: Unimplemented, is_unofficial: true, operation: none},
    // D
    Instruction {name:  BNE, mode: Relative,    category: Branch, is_unofficial: false, operation: none},
    Instruction {name:  CMP, mode: IndirectY,   category: Read, is_unofficial: false, operation: compare_memory_with_a},
    Instruction {name: JAM, mode: Implied,     category: Unimplemented, is_unofficial: true, operation: none},
    Instruction {name: DCP, mode: IndirectY,   category: Unimplemented, is_unofficial: true, operation: none},
    Instruction {name: NOP, mode: ZeroPageX,   category: NoOp, is_unofficial: true, operation: none},
    Instruction {name:  CMP, mode: ZeroPageX,   category: Read, is_unofficial: false, operation: compare_memory_with_a},
    Instruction {name:  DEC, mode: ZeroPageX,   category: ReadModifyWrite, is_unofficial: false, operation: decrement_memory},
    Instruction {name: DCP, mode: ZeroPageX,   category: Unimplemented, is_unofficial: true, operation: none},
    Instruction {name:  CLD, mode: Implied,     category: StatusRegister, is_unofficial: false, operation: clear_decimal_flag},
    Instruction {name:  CMP, mode: AbsoluteY,   category: Read, is_unofficial: false, operation: compare_memory_with_a},
    Instruction {name: NOP, mode: Implied,     category: NoOp, is_unofficial: true, operation: none},
    Instruction {name: DCP, mode: AbsoluteY,   category: Unimplemented, is_unofficial: true, operation: none},
    Instruction {name: NOP, mode: AbsoluteX,   category: NoOp, is_unofficial: true, operation: none},
    Instruction {name:  CMP, mode: AbsoluteX,   category: Read, is_unofficial: false, operation: compare_memory_with_a},
    Instruction {name:  DEC, mode: AbsoluteX,   category: ReadModifyWrite, is_unofficial: false, operation: decrement_memory},
    Instruction {name: DCP, mode: AbsoluteX,   category: Unimplemented, is_unofficial: true, operation: none},
    // E
    Instruction {name:  CPX, mode: Immediate,   category: Read, is_unofficial: false, operation: compare_memory_with_x},
    Instruction {name:  SBC, mode: IndirectX,   category: Read, is_unofficial: false, operation: subtract_with_carry},
    Instruction {name: NOP, mode: Immediate,   category: NoOp, is_unofficial: true, operation: none},
    Instruction {name: ISB, mode: IndirectX,   category: Unimplemented, is_unofficial: true, operation: none},
    Instruction {name:  CPX, mode: ZeroPage,    category: Read, is_unofficial: false, operation: compare_memory_with_x},
    Instruction {name:  SBC, mode: ZeroPage,    category: Read, is_unofficial: false, operation: subtract_with_carry},
    Instruction {name:  INC, mode: ZeroPage,    category: ReadModifyWrite, is_unofficial: false, operation: increment_memory},
    Instruction {name: ISB, mode: ZeroPage,    category: Unimplemented, is_unofficial: true, operation: none},
    Instruction {name:  INX, mode: Implied,     category: Register, is_unofficial: false, operation: increment_x},
    Instruction {name:  SBC, mode: Immediate,   category: Read, is_unofficial: false, operation: subtract_with_carry},
    Instruction {name:  NOP, mode: Implied,     category: NoOp, is_unofficial: false, operation: none},
    Instruction {name: SBC, mode: Immediate,   category: Unimplemented, is_unofficial: true, operation: subtract_with_carry},
    Instruction {name:  CPX, mode: Absolute,    category: Read, is_unofficial: false, operation: compare_memory_with_x},
    Instruction {name:  SBC, mode: Absolute,    category: Read, is_unofficial: false, operation: subtract_with_carry},
    Instruction {name:  INC, mode: Absolute,    category: ReadModifyWrite, is_unofficial: false, operation: increment_memory},
    Instruction {name: ISB, mode: Absolute,    category: Unimplemented, is_unofficial: true, operation: none},
    // F
    Instruction {name:  BEQ, mode: Relative,    category: Branch, is_unofficial: false, operation: none},
    Instruction {name:  SBC, mode: IndirectY,   category: Read, is_unofficial: false, operation: subtract_with_carry},
    Instruction {name: JAM, mode: Implied,     category: Unimplemented, is_unofficial: true, operation: none},
    Instruction {name: ISB, mode: IndirectY,   category: Unimplemented, is_unofficial: true, operation: none},
    Instruction {name: NOP, mode: ZeroPageX,   category: NoOp, is_unofficial: true, operation: none},
    Instruction {name:  SBC, mode: ZeroPageX,   category: Read, is_unofficial: false, operation: subtract_with_carry},
    Instruction {name:  INC, mode: ZeroPageX,   category: ReadModifyWrite, is_unofficial: false, operation: increment_memory},
    Instruction {name: ISB, mode: ZeroPageX,   category: Unimplemented, is_unofficial: true, operation: none},
    Instruction {name:  SED, mode: Implied,     category: StatusRegister, is_unofficial: false, operation: set_decimal_flag},
    Instruction {name:  SBC, mode: AbsoluteY,   category: Read, is_unofficial: false, operation: subtract_with_carry},
    Instruction {name: NOP, mode: Implied,     category: NoOp, is_unofficial: true, operation: none},
    Instruction {name: ISB, mode: AbsoluteY,   category: Unimplemented, is_unofficial: true, operation: none},
    Instruction {name: NOP, mode: AbsoluteX,   category: NoOp, is_unofficial: true, operation: none},
    Instruction {name:  SBC, mode: AbsoluteX,   category: Read, is_unofficial: false, operation: subtract_with_carry},
    Instruction {name:  INC, mode: AbsoluteX,   category: ReadModifyWrite, is_unofficial: false, operation: increment_memory},
    Instruction {name: ISB, mode: AbsoluteX,   category: Unimplemented, is_unofficial: true, operation: none},
];
