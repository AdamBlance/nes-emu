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

use super::operation_funcs;
use crate::nes::Nes;

#[derive(Copy, Clone, Default)]
pub struct Instruction { 
    pub name: Name,
    pub mode: Mode,
    pub category: Category,
    pub is_unofficial: bool,
}

impl Instruction {
    pub fn get_associated_function(&self) -> fn(&mut Nes) {
        // All of these instructions are R/W/RMW
        // This might as well be added to the lookup table instead
        // Not sure of the performance of a big match likes this, like I'm guessing it becomes a 
        // lookup table anyway? 
        match self.name {
            LDA => operation_funcs::load_a,
            LDX => operation_funcs::load_x,
            LDY => operation_funcs::load_y,

            STA => operation_funcs::store_a,
            STX => operation_funcs::store_x,
            STY => operation_funcs::store_y,
            
            TAX => operation_funcs::transfer_a_to_x,
            TAY => operation_funcs::transfer_a_to_y,
            TSX => operation_funcs::transfer_s_to_x,
            TXA => operation_funcs::transfer_x_to_a,
            TXS => operation_funcs::transfer_x_to_s,
            TYA => operation_funcs::transfer_y_to_a,
            
            ASL => operation_funcs::arithmetic_shift_left,
            LSR => operation_funcs::logical_shift_right,
            ROL => operation_funcs::rotate_left,
            ROR => operation_funcs::rotate_right,
            
            AND => operation_funcs::and,
            BIT => operation_funcs::bit,
            EOR => operation_funcs::xor,
            ORA => operation_funcs::or,
            
            ADC => operation_funcs::add_with_carry,
            SBC => operation_funcs::subtract_with_carry,
            
            DEC => operation_funcs::decrement_memory,
            DEX => operation_funcs::decrement_x,
            DEY => operation_funcs::decrement_y,
            
            INC => operation_funcs::increment_memory,
            INX => operation_funcs::increment_x,
            INY => operation_funcs::increment_y,
            
            CMP => operation_funcs::compare_memory_with_a,
            CPX => operation_funcs::compare_memory_with_x,
            CPY => operation_funcs::compare_memory_with_y,

            CLC => operation_funcs::clear_carry_flag,
            CLD => operation_funcs::clear_decimal_flag,
            CLI => operation_funcs::clear_interrupt_flag,
            CLV => operation_funcs::clear_overflow_flag,
            
            SEC => operation_funcs::set_carry_flag,
            SED => operation_funcs::set_decimal_flag,
            SEI => operation_funcs::set_interrupt_inhibit_flag,
            
            NOP => operation_funcs::nop,
            _   => operation_funcs::nop,
        }
    }
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
    Unimplemented,
}

impl Default for Category {
    fn default() -> Category {Read}
}


pub static INSTRUCTIONS: [Instruction; 256] = [
    // 0
    Instruction {name:  BRK, mode: Implied,     category: Control, is_unofficial: false},
    Instruction {name:  ORA, mode: IndirectX,   category: Read, is_unofficial: false},
    Instruction {name: JAM, mode: Implied,     category: Unimplemented, is_unofficial: true},
    Instruction {name: SLO, mode: IndirectX,   category: Unimplemented, is_unofficial: true},
    Instruction {name: NOP, mode: ZeroPage,    category: Control, is_unofficial: true},
    Instruction {name:  ORA, mode: ZeroPage,    category: Read, is_unofficial: false},
    Instruction {name:  ASL, mode: ZeroPage,    category: ReadModifyWrite, is_unofficial: false},
    Instruction {name: SLO, mode: ZeroPage,    category: Unimplemented, is_unofficial: true},
    Instruction {name:  PHP, mode: Implied,     category: Control, is_unofficial: false},
    Instruction {name:  ORA, mode: Immediate,   category: Read, is_unofficial: false},
    Instruction {name:  ASL, mode: Accumulator, category: Register, is_unofficial: false},
    Instruction {name: ANC, mode: Immediate,   category: Unimplemented, is_unofficial: true},
    Instruction {name: NOP, mode: Absolute,    category: Control, is_unofficial: true},
    Instruction {name:  ORA, mode: Absolute,    category: Read, is_unofficial: false},
    Instruction {name:  ASL, mode: Absolute,    category: ReadModifyWrite, is_unofficial: false},
    Instruction {name: SLO, mode: Absolute,    category: Unimplemented, is_unofficial: true},
    // 1
    Instruction {name:  BPL, mode: Relative,    category: Branch, is_unofficial: false},
    Instruction {name:  ORA, mode: IndirectY,   category: Read, is_unofficial: false},
    Instruction {name: JAM, mode: Implied,     category: Unimplemented, is_unofficial: true},
    Instruction {name: SLO, mode: IndirectY,   category: Unimplemented, is_unofficial: true},
    Instruction {name: NOP, mode: ZeroPageX,   category: Control, is_unofficial: true},
    Instruction {name:  ORA, mode: ZeroPageX,   category: Read, is_unofficial: false},
    Instruction {name:  ASL, mode: ZeroPageX,   category: ReadModifyWrite, is_unofficial: false},
    Instruction {name: SLO, mode: ZeroPageX,   category: Unimplemented, is_unofficial: true},
    Instruction {name:  CLC, mode: Implied,     category: Register, is_unofficial: false},
    Instruction {name:  ORA, mode: AbsoluteY,   category: Read, is_unofficial: false},
    Instruction {name: NOP, mode: Implied,     category: Control, is_unofficial: true},
    Instruction {name: SLO, mode: AbsoluteY,   category: Unimplemented, is_unofficial: true},
    Instruction {name: NOP, mode: AbsoluteX,   category: Control, is_unofficial: true},
    Instruction {name:  ORA, mode: AbsoluteX,   category: Read, is_unofficial: false},
    Instruction {name:  ASL, mode: AbsoluteX,   category: ReadModifyWrite, is_unofficial: false},
    Instruction {name: SLO, mode: AbsoluteX,   category: Unimplemented, is_unofficial: true},
    // 2
    Instruction {name:  JSR, mode: Absolute,    category: Control, is_unofficial: false},
    Instruction {name:  AND, mode: IndirectX,   category: Read, is_unofficial: false},
    Instruction {name: JAM, mode: Implied,     category: Unimplemented, is_unofficial: true},
    Instruction {name: RLA, mode: IndirectX,   category: Unimplemented, is_unofficial: true},
    Instruction {name:  BIT, mode: ZeroPage,    category: Read, is_unofficial: false},
    Instruction {name:  AND, mode: ZeroPage,    category: Read, is_unofficial: false},
    Instruction {name:  ROL, mode: ZeroPage,    category: ReadModifyWrite, is_unofficial: false},
    Instruction {name: RLA, mode: ZeroPage,    category: Unimplemented, is_unofficial: true},
    Instruction {name:  PLP, mode: Implied,     category: Control, is_unofficial: false},
    Instruction {name:  AND, mode: Immediate,   category: Read, is_unofficial: false},
    Instruction {name:  ROL, mode: Accumulator, category: Register, is_unofficial: false},
    Instruction {name: ANC, mode: Immediate,   category: Unimplemented, is_unofficial: true},
    Instruction {name:  BIT, mode: Absolute,    category: Read, is_unofficial: false},
    Instruction {name:  AND, mode: Absolute,    category: Read, is_unofficial: false},
    Instruction {name:  ROL, mode: Absolute,    category: ReadModifyWrite, is_unofficial: false},
    Instruction {name: RLA, mode: Absolute,    category: Unimplemented, is_unofficial: true},
    // 3
    Instruction {name:  BMI, mode: Relative,    category: Branch, is_unofficial: false},
    Instruction {name:  AND, mode: IndirectY,   category: Read, is_unofficial: false},
    Instruction {name: JAM, mode: Implied,     category: Unimplemented, is_unofficial: true},
    Instruction {name: RLA, mode: IndirectY,   category: Unimplemented, is_unofficial: true},
    Instruction {name: NOP, mode: ZeroPageX,   category: Control, is_unofficial: true},
    Instruction {name:  AND, mode: ZeroPageX,   category: Read, is_unofficial: false},
    Instruction {name:  ROL, mode: ZeroPageX,   category: ReadModifyWrite, is_unofficial: false},
    Instruction {name: RLA, mode: ZeroPageX,   category: Unimplemented, is_unofficial: true},
    Instruction {name:  SEC, mode: Implied,     category: Register, is_unofficial: false},
    Instruction {name:  AND, mode: AbsoluteY,   category: Read, is_unofficial: false},
    Instruction {name: NOP, mode: Implied,     category: Control, is_unofficial: true},
    Instruction {name: RLA, mode: AbsoluteY,   category: Unimplemented, is_unofficial: true},
    Instruction {name: NOP, mode: AbsoluteX,   category: Control, is_unofficial: true},
    Instruction {name:  AND, mode: AbsoluteX,   category: Read, is_unofficial: false},
    Instruction {name:  ROL, mode: AbsoluteX,   category: ReadModifyWrite, is_unofficial: false},
    Instruction {name: RLA, mode: AbsoluteX,   category: Unimplemented, is_unofficial: true},
    // 4
    Instruction {name:  RTI, mode: Implied,     category: Control, is_unofficial: false},
    Instruction {name:  EOR, mode: IndirectX,   category: Read, is_unofficial: false},
    Instruction {name: JAM, mode: Implied,     category: Unimplemented, is_unofficial: true},
    Instruction {name: SRE, mode: IndirectX,   category: Unimplemented, is_unofficial: true},
    Instruction {name: NOP, mode: ZeroPage,    category: Control, is_unofficial: true},
    Instruction {name:  EOR, mode: ZeroPage,    category: Read, is_unofficial: false},
    Instruction {name:  LSR, mode: ZeroPage,    category: ReadModifyWrite, is_unofficial: false},
    Instruction {name: SRE, mode: ZeroPage,    category: Unimplemented, is_unofficial: true},
    Instruction {name:  PHA, mode: Implied,     category: Control, is_unofficial: false},
    Instruction {name:  EOR, mode: Immediate,   category: Read, is_unofficial: false},
    Instruction {name:  LSR, mode: Accumulator, category: Register, is_unofficial: false},
    Instruction {name: ALR, mode: Immediate,   category: Unimplemented, is_unofficial: true},
    Instruction {name:  JMP, mode: Absolute,    category: Control, is_unofficial: false},
    Instruction {name:  EOR, mode: Absolute,    category: Read, is_unofficial: false},
    Instruction {name:  LSR, mode: Absolute,    category: ReadModifyWrite, is_unofficial: false},
    Instruction {name: SRE, mode: Absolute,    category: Unimplemented, is_unofficial: true},
    // 5
    Instruction {name:  BVC, mode: Relative,    category: Branch, is_unofficial: false},
    Instruction {name:  EOR, mode: IndirectY,   category: Read, is_unofficial: false},
    Instruction {name: JAM, mode: Implied,     category: Unimplemented, is_unofficial: true},
    Instruction {name: SRE, mode: IndirectY,   category: Unimplemented, is_unofficial: true},
    Instruction {name: NOP, mode: ZeroPageX,   category: Control, is_unofficial: true},
    Instruction {name:  EOR, mode: ZeroPageX,   category: Read, is_unofficial: false},
    Instruction {name:  LSR, mode: ZeroPageX,   category: ReadModifyWrite, is_unofficial: false},
    Instruction {name: SRE, mode: ZeroPageX,   category: Unimplemented, is_unofficial: true},
    Instruction {name:  CLI, mode: Implied,     category: Register, is_unofficial: false},
    Instruction {name:  EOR, mode: AbsoluteY,   category: Read, is_unofficial: false},
    Instruction {name: NOP, mode: Implied,     category: Control, is_unofficial: true},
    Instruction {name: SRE, mode: AbsoluteY,   category: Unimplemented, is_unofficial: true},
    Instruction {name: NOP, mode: AbsoluteX,   category: Control, is_unofficial: true},
    Instruction {name:  EOR, mode: AbsoluteX,   category: Read, is_unofficial: false},
    Instruction {name:  LSR, mode: AbsoluteX,   category: ReadModifyWrite, is_unofficial: false},
    Instruction {name: SRE, mode: AbsoluteX,   category: Unimplemented, is_unofficial: true},
    // 6
    Instruction {name:  RTS, mode: Implied,     category: Control, is_unofficial: false},
    Instruction {name:  ADC, mode: IndirectX,   category: Read, is_unofficial: false},
    Instruction {name: JAM, mode: Implied,     category: Unimplemented, is_unofficial: true},
    Instruction {name: RRA, mode: IndirectX,   category: Unimplemented, is_unofficial: true},
    Instruction {name: NOP, mode: ZeroPage,    category: Control, is_unofficial: true},
    Instruction {name:  ADC, mode: ZeroPage,    category: Read, is_unofficial: false},
    Instruction {name:  ROR, mode: ZeroPage,    category: ReadModifyWrite, is_unofficial: false},
    Instruction {name: RRA, mode: ZeroPage,    category: Unimplemented, is_unofficial: true},
    Instruction {name:  PLA, mode: Implied,     category: Control, is_unofficial: false},
    Instruction {name:  ADC, mode: Immediate,   category: Read, is_unofficial: false},
    Instruction {name:  ROR, mode: Accumulator, category: Register, is_unofficial: false},
    Instruction {name: ARR, mode: Immediate,   category: Unimplemented, is_unofficial: true},
    Instruction {name:  JMP, mode: AbsoluteI,   category: Control, is_unofficial: false},
    Instruction {name:  ADC, mode: Absolute,    category: Read, is_unofficial: false},
    Instruction {name:  ROR, mode: Absolute,    category: ReadModifyWrite, is_unofficial: false},
    Instruction {name: RRA, mode: Absolute,    category: Unimplemented, is_unofficial: true},
    // 7
    Instruction {name:  BVS, mode: Relative,    category: Branch, is_unofficial: false},
    Instruction {name:  ADC, mode: IndirectY,   category: Read, is_unofficial: false},
    Instruction {name: JAM, mode: Implied,     category: Unimplemented, is_unofficial: true},
    Instruction {name: RRA, mode: IndirectY,   category: Unimplemented, is_unofficial: true},
    Instruction {name: NOP, mode: ZeroPageX,   category: Control, is_unofficial: true},
    Instruction {name:  ADC, mode: ZeroPageX,   category: Read, is_unofficial: false},
    Instruction {name:  ROR, mode: ZeroPageX,   category: ReadModifyWrite, is_unofficial: false},
    Instruction {name: RRA, mode: ZeroPageX,   category: Unimplemented, is_unofficial: true},
    Instruction {name:  SEI, mode: Implied,     category: Register, is_unofficial: false},
    Instruction {name:  ADC, mode: AbsoluteY,   category: Read, is_unofficial: false},
    Instruction {name: NOP, mode: Implied,     category: Control, is_unofficial: true},
    Instruction {name: RRA, mode: AbsoluteY,   category: Unimplemented, is_unofficial: true},
    Instruction {name: NOP, mode: AbsoluteX,   category: Control, is_unofficial: true},
    Instruction {name:  ADC, mode: AbsoluteX,   category: Read, is_unofficial: false},
    Instruction {name:  ROR, mode: AbsoluteX,   category: ReadModifyWrite, is_unofficial: false},
    Instruction {name: RRA, mode: AbsoluteX,   category: Unimplemented, is_unofficial: true},
    // 8
    Instruction {name: NOP, mode: Immediate,   category: Control, is_unofficial: true},
    Instruction {name:  STA, mode: IndirectX,   category: Write, is_unofficial: false},
    Instruction {name: NOP, mode: Immediate,   category: Control, is_unofficial: true},
    Instruction {name: SAX, mode: IndirectX,   category: Unimplemented, is_unofficial: true},
    Instruction {name:  STY, mode: ZeroPage,    category: Write, is_unofficial: false},
    Instruction {name:  STA, mode: ZeroPage,    category: Write, is_unofficial: false},
    Instruction {name:  STX, mode: ZeroPage,    category: Write, is_unofficial: false},
    Instruction {name: SAX, mode: ZeroPage,    category: Unimplemented, is_unofficial: true},
    Instruction {name:  DEY, mode: Implied,     category: Register, is_unofficial: false},
    Instruction {name: NOP, mode: Immediate,   category: Control, is_unofficial: true},
    Instruction {name:  TXA, mode: Implied,     category: Register, is_unofficial: false},
    Instruction {name: XAA, mode: Immediate,   category: Unimplemented, is_unofficial: true},
    Instruction {name:  STY, mode: Absolute,    category: Write, is_unofficial: false},
    Instruction {name:  STA, mode: Absolute,    category: Write, is_unofficial: false},
    Instruction {name:  STX, mode: Absolute,    category: Write, is_unofficial: false},
    Instruction {name: SAX, mode: Absolute,    category: Unimplemented, is_unofficial: true},
    // 9
    Instruction {name:  BCC, mode: Relative,    category: Branch, is_unofficial: false},
    Instruction {name:  STA, mode: IndirectY,   category: Write, is_unofficial: false},
    Instruction {name: JAM, mode: Implied,     category: Unimplemented, is_unofficial: true},
    Instruction {name: SHA, mode: IndirectY,   category: Unimplemented, is_unofficial: true},
    Instruction {name:  STY, mode: ZeroPageX,   category: Write, is_unofficial: false},
    Instruction {name:  STA, mode: ZeroPageX,   category: Write, is_unofficial: false},
    Instruction {name:  STX, mode: ZeroPageY,   category: Write, is_unofficial: false},
    Instruction {name: SAX, mode: ZeroPageY,   category: Unimplemented, is_unofficial: true},
    Instruction {name:  TYA, mode: Implied,     category: Register, is_unofficial: false},
    Instruction {name:  STA, mode: AbsoluteY,   category: Write, is_unofficial: false},
    Instruction {name:  TXS, mode: Implied,     category: Register, is_unofficial: false},
    Instruction {name: SHS, mode: AbsoluteY,   category: Unimplemented, is_unofficial: true},
    Instruction {name: SHY, mode: AbsoluteX,   category: Unimplemented, is_unofficial: true},
    Instruction {name:  STA, mode: AbsoluteX,   category: Write, is_unofficial: false},
    Instruction {name: SHX, mode: AbsoluteY,   category: Unimplemented, is_unofficial: true},
    Instruction {name: SHA, mode: AbsoluteY,   category: Unimplemented, is_unofficial: true},
    // A
    Instruction {name:  LDY, mode: Immediate,   category: Read, is_unofficial: false},
    Instruction {name:  LDA, mode: IndirectX,   category: Read, is_unofficial: false},
    Instruction {name:  LDX, mode: Immediate,   category: Read, is_unofficial: false},
    Instruction {name: LAX, mode: IndirectX,   category: Unimplemented, is_unofficial: true},
    Instruction {name:  LDY, mode: ZeroPage,    category: Read, is_unofficial: false},
    Instruction {name:  LDA, mode: ZeroPage,    category: Read, is_unofficial: false},
    Instruction {name:  LDX, mode: ZeroPage,    category: Read, is_unofficial: false},
    Instruction {name: LAX, mode: ZeroPage,    category: Unimplemented, is_unofficial: true},
    Instruction {name:  TAY, mode: Implied,     category: Register, is_unofficial: false},
    Instruction {name:  LDA, mode: Immediate,   category: Read, is_unofficial: false},
    Instruction {name:  TAX, mode: Implied,     category: Register, is_unofficial: false},
    Instruction {name: LAX, mode: Immediate,   category: Unimplemented, is_unofficial: true},
    Instruction {name:  LDY, mode: Absolute,    category: Read, is_unofficial: false},
    Instruction {name:  LDA, mode: Absolute,    category: Read, is_unofficial: false},
    Instruction {name:  LDX, mode: Absolute,    category: Read, is_unofficial: false},
    Instruction {name: LAX, mode: Absolute,    category: Unimplemented, is_unofficial: true},
    // B
    Instruction {name:  BCS, mode: Relative,    category: Branch, is_unofficial: false},
    Instruction {name:  LDA, mode: IndirectY,   category: Read, is_unofficial: false},
    Instruction {name: JAM, mode: Implied,     category: Unimplemented, is_unofficial: true},
    Instruction {name: LAX, mode: IndirectY,   category: Unimplemented, is_unofficial: true},
    Instruction {name:  LDY, mode: ZeroPageX,   category: Read, is_unofficial: false},
    Instruction {name:  LDA, mode: ZeroPageX,   category: Read, is_unofficial: false},
    Instruction {name:  LDX, mode: ZeroPageY,   category: Read, is_unofficial: false},
    Instruction {name: LAX, mode: ZeroPageY,   category: Unimplemented, is_unofficial: true},
    Instruction {name:  CLV, mode: Implied,     category: Register, is_unofficial: false},
    Instruction {name:  LDA, mode: AbsoluteY,   category: Read, is_unofficial: false},
    Instruction {name:  TSX, mode: Implied,     category: Register, is_unofficial: false},
    Instruction {name: LAS, mode: AbsoluteY,   category: Unimplemented, is_unofficial: true},
    Instruction {name:  LDY, mode: AbsoluteX,   category: Read, is_unofficial: false},
    Instruction {name:  LDA, mode: AbsoluteX,   category: Read, is_unofficial: false},
    Instruction {name:  LDX, mode: AbsoluteY,   category: Read, is_unofficial: false},
    Instruction {name: LAX, mode: AbsoluteY,   category: Unimplemented, is_unofficial: true},
    // C
    Instruction {name:  CPY, mode: Immediate,   category: Read, is_unofficial: false},
    Instruction {name:  CMP, mode: IndirectX,   category: Read, is_unofficial: false},
    Instruction {name: NOP, mode: Immediate,   category: Control, is_unofficial: true},
    Instruction {name: DCP, mode: IndirectX,   category: Unimplemented, is_unofficial: true},
    Instruction {name:  CPY, mode: ZeroPage,    category: Read, is_unofficial: false},
    Instruction {name:  CMP, mode: ZeroPage,    category: Read, is_unofficial: false},
    Instruction {name:  DEC, mode: ZeroPage,    category: ReadModifyWrite, is_unofficial: false},
    Instruction {name: DCP, mode: ZeroPage,    category: Unimplemented, is_unofficial: true},
    Instruction {name:  INY, mode: Implied,     category: Register, is_unofficial: false},
    Instruction {name:  CMP, mode: Immediate,   category: Read, is_unofficial: false},
    Instruction {name:  DEX, mode: Implied,     category: Register, is_unofficial: false},
    Instruction {name: AXS, mode: Immediate,   category: Unimplemented, is_unofficial: true},
    Instruction {name:  CPY, mode: Absolute,    category: Read, is_unofficial: false},
    Instruction {name:  CMP, mode: Absolute,    category: Read, is_unofficial: false},
    Instruction {name:  DEC, mode: Absolute,    category: ReadModifyWrite, is_unofficial: false},
    Instruction {name: DCP, mode: Absolute,    category: Unimplemented, is_unofficial: true},
    // D
    Instruction {name:  BNE, mode: Relative,    category: Branch, is_unofficial: false},
    Instruction {name:  CMP, mode: IndirectY,   category: Read, is_unofficial: false},
    Instruction {name: JAM, mode: Implied,     category: Unimplemented, is_unofficial: true},
    Instruction {name: DCP, mode: IndirectY,   category: Unimplemented, is_unofficial: true},
    Instruction {name: NOP, mode: ZeroPageX,   category: Control, is_unofficial: true},
    Instruction {name:  CMP, mode: ZeroPageX,   category: Read, is_unofficial: false},
    Instruction {name:  DEC, mode: ZeroPageX,   category: ReadModifyWrite, is_unofficial: false},
    Instruction {name: DCP, mode: ZeroPageX,   category: Unimplemented, is_unofficial: true},
    Instruction {name:  CLD, mode: Implied,     category: Register, is_unofficial: false},
    Instruction {name:  CMP, mode: AbsoluteY,   category: Read, is_unofficial: false},
    Instruction {name: NOP, mode: Implied,     category: Control, is_unofficial: true},
    Instruction {name: DCP, mode: AbsoluteY,   category: Unimplemented, is_unofficial: true},
    Instruction {name: NOP, mode: AbsoluteX,   category: Control, is_unofficial: true},
    Instruction {name:  CMP, mode: AbsoluteX,   category: Read, is_unofficial: false},
    Instruction {name:  DEC, mode: AbsoluteX,   category: ReadModifyWrite, is_unofficial: false},
    Instruction {name: DCP, mode: AbsoluteX,   category: Unimplemented, is_unofficial: true},
    // E
    Instruction {name:  CPX, mode: Immediate,   category: Read, is_unofficial: false},
    Instruction {name:  SBC, mode: IndirectX,   category: Read, is_unofficial: false},
    Instruction {name: NOP, mode: Immediate,   category: Control, is_unofficial: true},
    Instruction {name: ISB, mode: IndirectX,   category: Unimplemented, is_unofficial: true},
    Instruction {name:  CPX, mode: ZeroPage,    category: Read, is_unofficial: false},
    Instruction {name:  SBC, mode: ZeroPage,    category: Read, is_unofficial: false},
    Instruction {name:  INC, mode: ZeroPage,    category: ReadModifyWrite, is_unofficial: false},
    Instruction {name: ISB, mode: ZeroPage,    category: Unimplemented, is_unofficial: true},
    Instruction {name:  INX, mode: Implied,     category: Register, is_unofficial: false},
    Instruction {name:  SBC, mode: Immediate,   category: Read, is_unofficial: false},
    Instruction {name:  NOP, mode: Implied,     category: Control, is_unofficial: false},
    Instruction {name: SBC, mode: Immediate,   category: Unimplemented, is_unofficial: true},
    Instruction {name:  CPX, mode: Absolute,    category: Read, is_unofficial: false},
    Instruction {name:  SBC, mode: Absolute,    category: Read, is_unofficial: false},
    Instruction {name:  INC, mode: Absolute,    category: ReadModifyWrite, is_unofficial: false},
    Instruction {name: ISB, mode: Absolute,    category: Unimplemented, is_unofficial: true},
    // F
    Instruction {name:  BEQ, mode: Relative,    category: Branch, is_unofficial: false},
    Instruction {name:  SBC, mode: IndirectY,   category: Read, is_unofficial: false},
    Instruction {name: JAM, mode: Implied,     category: Unimplemented, is_unofficial: true},
    Instruction {name: ISB, mode: IndirectY,   category: Unimplemented, is_unofficial: true},
    Instruction {name: NOP, mode: ZeroPageX,   category: Control, is_unofficial: true},
    Instruction {name:  SBC, mode: ZeroPageX,   category: Read, is_unofficial: false},
    Instruction {name:  INC, mode: ZeroPageX,   category: ReadModifyWrite, is_unofficial: false},
    Instruction {name: ISB, mode: ZeroPageX,   category: Unimplemented, is_unofficial: true},
    Instruction {name:  SED, mode: Implied,     category: Register, is_unofficial: false},
    Instruction {name:  SBC, mode: AbsoluteY,   category: Read, is_unofficial: false},
    Instruction {name: NOP, mode: Implied,     category: Control, is_unofficial: true},
    Instruction {name: ISB, mode: AbsoluteY,   category: Unimplemented, is_unofficial: true},
    Instruction {name: NOP, mode: AbsoluteX,   category: Control, is_unofficial: true},
    Instruction {name:  SBC, mode: AbsoluteX,   category: Read, is_unofficial: false},
    Instruction {name:  INC, mode: AbsoluteX,   category: ReadModifyWrite, is_unofficial: false},
    Instruction {name: ISB, mode: AbsoluteX,   category: Unimplemented, is_unofficial: true},
];
