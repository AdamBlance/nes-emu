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
    ULAS, ULAX, USAX, USHA, USHX, USHY, USHS, UANC, UARR, UASR, UDCP, UISC, 
    URLA, URRA, USBC, USBX, USLO, USRE, UXAA, UJAM, UNOP, UALR, UISB, UAXS,
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
    Instruction {name:  BRK, mode: Implied,     category: Control},
    Instruction {name:  ORA, mode: IndirectX,   category: Read},
    Instruction {name: UJAM, mode: Implied,     category: Unimplemented},
    Instruction {name: USLO, mode: IndirectX,   category: Unimplemented},
    Instruction {name: UNOP, mode: ZeroPage,    category: Unimplemented},
    Instruction {name:  ORA, mode: ZeroPage,    category: Read},
    Instruction {name:  ASL, mode: ZeroPage,    category: ReadModifyWrite},
    Instruction {name: USLO, mode: ZeroPage,    category: Unimplemented},
    Instruction {name:  PHP, mode: Implied,     category: Control},
    Instruction {name:  ORA, mode: Immediate,   category: Read},
    Instruction {name:  ASL, mode: Accumulator, category: Register},
    Instruction {name: UANC, mode: Immediate,   category: Unimplemented},
    Instruction {name: UNOP, mode: Absolute,    category: Unimplemented},
    Instruction {name:  ORA, mode: Absolute,    category: Read},
    Instruction {name:  ASL, mode: Absolute,    category: ReadModifyWrite},
    Instruction {name: USLO, mode: Absolute,    category: Unimplemented},
    // 1
    Instruction {name:  BPL, mode: Relative,    category: Branch},
    Instruction {name:  ORA, mode: IndirectY,   category: Read},
    Instruction {name: UJAM, mode: Implied,     category: Unimplemented},
    Instruction {name: USLO, mode: IndirectY,   category: Unimplemented},
    Instruction {name: UNOP, mode: ZeroPageX,   category: Unimplemented},
    Instruction {name:  ORA, mode: ZeroPageX,   category: Read},
    Instruction {name:  ASL, mode: ZeroPageX,   category: ReadModifyWrite},
    Instruction {name: USLO, mode: ZeroPageX,   category: Unimplemented},
    Instruction {name:  CLC, mode: Implied,     category: Register},
    Instruction {name:  ORA, mode: AbsoluteY,   category: Read},
    Instruction {name: UNOP, mode: Implied,     category: Unimplemented},
    Instruction {name: USLO, mode: AbsoluteY,   category: Unimplemented},
    Instruction {name: UNOP, mode: AbsoluteX,   category: Unimplemented},
    Instruction {name:  ORA, mode: AbsoluteX,   category: Read},
    Instruction {name:  ASL, mode: AbsoluteX,   category: ReadModifyWrite},
    Instruction {name: USLO, mode: AbsoluteX,   category: Unimplemented},

    Instruction {name:  JSR, mode: Absolute,    category: Control},
    Instruction {name:  AND, mode: IndirectX,   category: Read},
    Instruction {name: UJAM, mode: Implied,     category: Unimplemented},
    Instruction {name: URLA, mode: IndirectX,   category: Unimplemented},
    Instruction {name:  BIT, mode: ZeroPage,    category: Read},
    Instruction {name:  AND, mode: ZeroPage,    category: Read},
    Instruction {name:  ROL, mode: ZeroPage,    category: ReadModifyWrite},
    Instruction {name: URLA, mode: ZeroPage,    category: Unimplemented},
    Instruction {name:  PLP, mode: Implied,     category: Control},
    Instruction {name:  AND, mode: Immediate,   category: Read},
    Instruction {name:  ROL, mode: Accumulator, category: Register},
    Instruction {name: UANC, mode: Immediate,   category: Unimplemented},
    Instruction {name:  BIT, mode: Absolute,    category: Read},
    Instruction {name:  AND, mode: Absolute,    category: Read},
    Instruction {name:  ROL, mode: Absolute,    category: ReadModifyWrite},
    Instruction {name: URLA, mode: Absolute,    category: Unimplemented},

    Instruction {name:  BMI, mode: Relative,    category: Branch},
    Instruction {name:  AND, mode: IndirectY,   category: Read},
    Instruction {name: UJAM, mode: Implied,     category: Unimplemented},
    Instruction {name: URLA, mode: IndirectY,   category: Unimplemented},
    Instruction {name: UNOP, mode: ZeroPageX,   category: Unimplemented},
    Instruction {name:  AND, mode: ZeroPageX,   category: Read},
    Instruction {name:  ROL, mode: ZeroPageX,   category: ReadModifyWrite},
    Instruction {name: URLA, mode: ZeroPageX,   category: Unimplemented},
    Instruction {name:  SEC, mode: Implied,     category: Register},
    Instruction {name:  AND, mode: AbsoluteY,   category: Read},
    Instruction {name: UNOP, mode: Implied,     category: Unimplemented},
    Instruction {name: URLA, mode: AbsoluteY,   category: Unimplemented},
    Instruction {name: UNOP, mode: AbsoluteX,   category: Unimplemented},
    Instruction {name:  AND, mode: AbsoluteX,   category: Read},
    Instruction {name:  ROL, mode: AbsoluteX,   category: ReadModifyWrite},
    Instruction {name: URLA, mode: AbsoluteX,   category: Unimplemented},

    Instruction {name:  RTI, mode: Implied,     category: Control},
    Instruction {name:  EOR, mode: IndirectX,   category: Read},
    Instruction {name: UJAM, mode: Implied,     category: Unimplemented},
    Instruction {name: USRE, mode: IndirectX,   category: Unimplemented},
    Instruction {name: UNOP, mode: ZeroPage,    category: Unimplemented},
    Instruction {name:  EOR, mode: ZeroPage,    category: Read},
    Instruction {name:  LSR, mode: ZeroPage,    category: ReadModifyWrite},
    Instruction {name: USRE, mode: ZeroPage,    category: Unimplemented},
    Instruction {name:  PHA, mode: Implied,     category: Control},
    Instruction {name:  EOR, mode: Immediate,   category: Read},
    Instruction {name:  LSR, mode: Accumulator, category: Register},
    Instruction {name: UALR, mode: Immediate,   category: Unimplemented},
    Instruction {name:  JMP, mode: Absolute,    category: Control},
    Instruction {name:  EOR, mode: Absolute,    category: Read},
    Instruction {name:  LSR, mode: Absolute,    category: ReadModifyWrite},
    Instruction {name: USRE, mode: Absolute,    category: Unimplemented},

    Instruction {name:  BVC, mode: Relative,    category: Branch},
    Instruction {name:  EOR, mode: IndirectY,   category: Read},
    Instruction {name: UJAM, mode: Implied,     category: Unimplemented},
    Instruction {name: USRE, mode: IndirectY,   category: Unimplemented},
    Instruction {name: UNOP, mode: ZeroPageX,   category: Unimplemented},
    Instruction {name:  EOR, mode: ZeroPageX,   category: Read},
    Instruction {name:  LSR, mode: ZeroPageX,   category: ReadModifyWrite},
    Instruction {name: USRE, mode: ZeroPageX,   category: Unimplemented},
    Instruction {name:  CLI, mode: Implied,     category: Register},
    Instruction {name:  EOR, mode: AbsoluteY,   category: Read},
    Instruction {name: UNOP, mode: Implied,     category: Unimplemented},
    Instruction {name: USRE, mode: AbsoluteY,   category: Unimplemented},
    Instruction {name: UNOP, mode: AbsoluteX,   category: Unimplemented},
    Instruction {name:  EOR, mode: AbsoluteX,   category: Read},
    Instruction {name:  LSR, mode: AbsoluteX,   category: ReadModifyWrite},
    Instruction {name: USRE, mode: AbsoluteX,   category: Unimplemented},

    Instruction {name:  RTS, mode: Implied,     category: Control},
    Instruction {name:  ADC, mode: IndirectX,   category: Read},
    Instruction {name: UJAM, mode: Implied,     category: Unimplemented},
    Instruction {name: URRA, mode: IndirectX,   category: Unimplemented},
    Instruction {name: UNOP, mode: ZeroPage,    category: Unimplemented},
    Instruction {name:  ADC, mode: ZeroPage,    category: Read},
    Instruction {name:  ROR, mode: ZeroPage,    category: ReadModifyWrite},
    Instruction {name: URRA, mode: ZeroPage,    category: Unimplemented},
    Instruction {name:  PLA, mode: Implied,     category: Control},
    Instruction {name:  ADC, mode: Immediate,   category: Read},
    Instruction {name:  ROR, mode: Accumulator, category: Register},
    Instruction {name: UARR, mode: Immediate,   category: Unimplemented},
    Instruction {name:  JMP, mode: AbsoluteI,   category: Control},
    Instruction {name:  ADC, mode: Absolute,    category: Read},
    Instruction {name:  ROR, mode: Absolute,    category: ReadModifyWrite},
    Instruction {name: URRA, mode: Absolute,    category: Unimplemented},

    Instruction {name:  BVS, mode: Relative,    category: Branch},
    Instruction {name:  ADC, mode: IndirectY,   category: Read},
    Instruction {name: UJAM, mode: Implied,     category: Unimplemented},
    Instruction {name: URRA, mode: IndirectY,   category: Unimplemented},
    Instruction {name: UNOP, mode: ZeroPageX,   category: Unimplemented},
    Instruction {name:  ADC, mode: ZeroPageX,   category: Read},
    Instruction {name:  ROR, mode: ZeroPageX,   category: ReadModifyWrite},
    Instruction {name: URRA, mode: ZeroPageX,   category: Unimplemented},
    Instruction {name:  SEI, mode: Implied,     category: Register},
    Instruction {name:  ADC, mode: AbsoluteY,   category: Read},
    Instruction {name: UNOP, mode: Implied,     category: Unimplemented},
    Instruction {name: URRA, mode: AbsoluteY,   category: Unimplemented},
    Instruction {name: UNOP, mode: AbsoluteX,   category: Unimplemented},
    Instruction {name:  ADC, mode: AbsoluteX,   category: Read},
    Instruction {name:  ROR, mode: AbsoluteX,   category: ReadModifyWrite},
    Instruction {name: URRA, mode: AbsoluteX,   category: Unimplemented},

    Instruction {name: UNOP, mode: Immediate,   category: Unimplemented},
    Instruction {name:  STA, mode: IndirectX,   category: Write},
    Instruction {name: UNOP, mode: Immediate,   category: Unimplemented},
    Instruction {name: USAX, mode: IndirectX,   category: Unimplemented},
    Instruction {name:  STY, mode: ZeroPage,    category: Write},
    Instruction {name:  STA, mode: ZeroPage,    category: Write},
    Instruction {name:  STX, mode: ZeroPage,    category: Write},
    Instruction {name: USAX, mode: ZeroPage,    category: Unimplemented},
    Instruction {name:  DEY, mode: Implied,     category: Register},
    Instruction {name: UNOP, mode: Immediate,   category: Unimplemented},
    Instruction {name:  TXA, mode: Implied,     category: Register},
    Instruction {name: UXAA, mode: Immediate,   category: Unimplemented},
    Instruction {name:  STY, mode: Absolute,    category: Write},
    Instruction {name:  STA, mode: Absolute,    category: Write},
    Instruction {name:  STX, mode: Absolute,    category: Write},
    Instruction {name: USAX, mode: Absolute,    category: Unimplemented},
    Instruction {name:  BCC, mode: Relative,    category: Branch},
    Instruction {name:  STA, mode: IndirectY,   category: Write},
    Instruction {name: UJAM, mode: Implied,     category: Unimplemented},
    Instruction {name: USHA, mode: IndirectY,   category: Unimplemented},
    Instruction {name:  STY, mode: ZeroPageX,   category: Write},
    Instruction {name:  STA, mode: ZeroPageX,   category: Write},
    Instruction {name:  STX, mode: ZeroPageY,   category: Write},
    Instruction {name: USAX, mode: ZeroPageY,   category: Unimplemented},
    Instruction {name:  TYA, mode: Implied,     category: Register},
    Instruction {name:  STA, mode: AbsoluteY,   category: Write},
    Instruction {name:  TXS, mode: Implied,     category: Register},
    Instruction {name: USHS, mode: AbsoluteY,   category: Unimplemented},
    Instruction {name: USHY, mode: AbsoluteX,   category: Unimplemented},
    Instruction {name:  STA, mode: AbsoluteX,   category: Write},
    Instruction {name: USHX, mode: AbsoluteY,   category: Unimplemented},
    Instruction {name: USHA, mode: AbsoluteY,   category: Unimplemented},
    Instruction {name:  LDY, mode: Immediate,   category: Read},
    Instruction {name:  LDA, mode: IndirectX,   category: Read},
    Instruction {name:  LDX, mode: Immediate,   category: Read},
    Instruction {name: ULAX, mode: IndirectX,   category: Unimplemented},
    Instruction {name:  LDY, mode: ZeroPage,    category: Read},
    Instruction {name:  LDA, mode: ZeroPage,    category: Read},
    Instruction {name:  LDX, mode: ZeroPage,    category: Read},
    Instruction {name: ULAX, mode: ZeroPage,    category: Unimplemented},
    Instruction {name:  TAY, mode: Implied,     category: Register},
    Instruction {name:  LDA, mode: Immediate,   category: Read},
    Instruction {name:  TAX, mode: Implied,     category: Register},
    Instruction {name: ULAX, mode: Immediate,   category: Unimplemented},
    Instruction {name:  LDY, mode: Absolute,    category: Read},
    Instruction {name:  LDA, mode: Absolute,    category: Read},
    Instruction {name:  LDX, mode: Absolute,    category: Read},
    Instruction {name: ULAX, mode: Absolute,    category: Unimplemented},
    Instruction {name:  BCS, mode: Relative,    category: Branch},
    Instruction {name:  LDA, mode: IndirectY,   category: Read},
    Instruction {name: UJAM, mode: Implied,     category: Unimplemented},
    Instruction {name: ULAX, mode: IndirectY,   category: Unimplemented},
    Instruction {name:  LDY, mode: ZeroPageX,   category: Read},
    Instruction {name:  LDA, mode: ZeroPageX,   category: Read},
    Instruction {name:  LDX, mode: ZeroPageY,   category: Read},
    Instruction {name: ULAX, mode: ZeroPageY,   category: Unimplemented},
    Instruction {name:  CLV, mode: Implied,     category: Register},
    Instruction {name:  LDA, mode: AbsoluteY,   category: Read},
    Instruction {name:  TSX, mode: Implied,     category: Register},
    Instruction {name: ULAS, mode: AbsoluteY,   category: Unimplemented},
    Instruction {name:  LDY, mode: AbsoluteX,   category: Read},
    Instruction {name:  LDA, mode: AbsoluteX,   category: Read},
    Instruction {name:  LDX, mode: AbsoluteY,   category: Read},
    Instruction {name: ULAX, mode: AbsoluteY,   category: Unimplemented},
    Instruction {name:  CPY, mode: Immediate,   category: Read},
    Instruction {name:  CMP, mode: IndirectX,   category: Read},
    Instruction {name: UNOP, mode: Immediate,   category: Unimplemented},
    Instruction {name: UDCP, mode: IndirectX,   category: Unimplemented},
    Instruction {name:  CPY, mode: ZeroPage,    category: Read},
    Instruction {name:  CMP, mode: ZeroPage,    category: Read},
    Instruction {name:  DEC, mode: ZeroPage,    category: ReadModifyWrite},
    Instruction {name: UDCP, mode: ZeroPage,    category: Unimplemented},
    Instruction {name:  INY, mode: Implied,     category: Register},
    Instruction {name:  CMP, mode: Immediate,   category: Read},
    Instruction {name:  DEX, mode: Implied,     category: Register},
    Instruction {name: UAXS, mode: Immediate,   category: Unimplemented},
    Instruction {name:  CPY, mode: Absolute,    category: Read},
    Instruction {name:  CMP, mode: Absolute,    category: Read},
    Instruction {name:  DEC, mode: Absolute,    category: ReadModifyWrite},
    Instruction {name: UDCP, mode: Absolute,    category: Unimplemented},
    Instruction {name:  BNE, mode: Relative,    category: Branch},
    Instruction {name:  CMP, mode: IndirectY,   category: Read},
    Instruction {name: UJAM, mode: Implied,     category: Unimplemented},
    Instruction {name: UDCP, mode: IndirectY,   category: Unimplemented},
    Instruction {name: UNOP, mode: ZeroPageX,   category: Unimplemented},
    Instruction {name:  CMP, mode: ZeroPageX,   category: Read},
    Instruction {name:  DEC, mode: ZeroPageX,   category: ReadModifyWrite},
    Instruction {name: UDCP, mode: ZeroPageX,   category: Unimplemented},
    Instruction {name:  CLD, mode: Implied,     category: Register},
    Instruction {name:  CMP, mode: AbsoluteY,   category: Read},
    Instruction {name: UNOP, mode: Implied,     category: Unimplemented},
    Instruction {name: UDCP, mode: AbsoluteY,   category: Unimplemented},
    Instruction {name: UNOP, mode: AbsoluteX,   category: Unimplemented},
    Instruction {name:  CMP, mode: AbsoluteX,   category: Read},
    Instruction {name:  DEC, mode: AbsoluteX,   category: ReadModifyWrite},
    Instruction {name: UDCP, mode: AbsoluteX,   category: Unimplemented},
    Instruction {name:  CPX, mode: Immediate,   category: Read},
    Instruction {name:  SBC, mode: IndirectX,   category: Read},
    Instruction {name: UNOP, mode: Immediate,   category: Unimplemented},
    Instruction {name: UISB, mode: IndirectX,   category: Unimplemented},
    Instruction {name:  CPX, mode: ZeroPage,    category: Read},
    Instruction {name:  SBC, mode: ZeroPage,    category: Read},
    Instruction {name:  INC, mode: ZeroPage,    category: ReadModifyWrite},
    Instruction {name: UISB, mode: ZeroPage,    category: Unimplemented},
    Instruction {name:  INX, mode: Implied,     category: Register},
    Instruction {name:  SBC, mode: Immediate,   category: Read},
    Instruction {name:  NOP, mode: Implied,     category: Control},
    Instruction {name: USBC, mode: Immediate,   category: Unimplemented},
    Instruction {name:  CPX, mode: Absolute,    category: Read},
    Instruction {name:  SBC, mode: Absolute,    category: Read},
    Instruction {name:  INC, mode: Absolute,    category: ReadModifyWrite},
    Instruction {name: UISB, mode: Absolute,    category: Unimplemented},

    Instruction {name:  BEQ, mode: Relative,    category: Branch},
    Instruction {name:  SBC, mode: IndirectY,   category: Read},
    Instruction {name: UJAM, mode: Implied,     category: Unimplemented},
    Instruction {name: UISB, mode: IndirectY,   category: Unimplemented},
    Instruction {name: UNOP, mode: ZeroPageX,   category: Unimplemented},
    Instruction {name:  SBC, mode: ZeroPageX,   category: Read},
    Instruction {name:  INC, mode: ZeroPageX,   category: ReadModifyWrite},
    Instruction {name: UISB, mode: ZeroPageX,   category: Unimplemented},
    Instruction {name:  SED, mode: Implied,     category: Register},
    Instruction {name:  SBC, mode: AbsoluteY,   category: Read},
    Instruction {name: UNOP, mode: Implied,     category: Unimplemented},
    Instruction {name: UISB, mode: AbsoluteY,   category: Unimplemented},
    Instruction {name: UNOP, mode: AbsoluteX,   category: Unimplemented},
    Instruction {name:  SBC, mode: AbsoluteX,   category: Read},
    Instruction {name:  INC, mode: AbsoluteX,   category: ReadModifyWrite},
    Instruction {name: UISB, mode: AbsoluteX,   category: Unimplemented},
];
