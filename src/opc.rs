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
use crate::util::*;
use crate::mem::*;
use Mode::*;
use Method::*;
use Name::*;
use once_cell::sync::Lazy;


#[derive(Clone)]
pub struct Instruction { 
    pub name: Name,
    pub mode: Mode,
    pub method: Method,
    pub ops: Vec<Vec<fn(&mut Nes)>>,
}
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
    pub fn num_bytes(&self) -> u16 {
        match self {
            Accumulator => 1,
            Implied     => 1,
            Immediate   => 2,
            Relative    => 2,
            IndirectX   => 2,
            IndirectY   => 2,
            ZeroPage    => 2,
            ZeroPageX   => 2,
            ZeroPageY   => 2,
            Absolute    => 3,
            AbsoluteX   => 3,
            AbsoluteY   => 3,
            AbsoluteI   => 3,
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


// #[derive(Copy, Clone)]
// pub struct MicroOp {
//     pub op:      fn(&mut Nes),
//     pub r:      fn(&mut Nes),
//     pub inc_pc: bool,
// }


// #[derive(Copy, Clone)]
// pub struct MicroOp {
//     pub op:     fn(&mut Nes),
// }

const EMPTY: Vec<Vec<fn(&mut Nes)>> = Vec::new();
const DUMMY_READ_FROM_PC: fn(&mut Nes) = read_from_pc;


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

    // https://stackoverflow.com/questions/70549985/what-enables-a-closure-type-to-be-used-where-a-function-pointer-type-is-expected
    // https://rust-lang.github.io/rfcs/1558-closure-to-fn-coercion.html


    for i in 0..256 {


        /*
        right, needs to be said that lower_address and upper_address do not represent the address bus!
        the address bus does all sorts of crazy things
        like, to read any value from memory, the address bus needs to have the right location
        
        in absolute addressing, the lower address is read from pc+1, then the upper address is read from pc+2

        to read the upper address, the address bus registers have to contain the PC, so the previously read
        address lsb would just get erased

        basically, the lsb gets stored in some other place while the high byte is read

        who knows where. It seems to be the ADD register? The one after the accumulator? 
        Idk honestly

        I'm just going to do the reads at the right times and try to make things intuitive without 
        following every single step that the cpu does because it literally doesn't matter to the 
        emulator at all


        */



        // These instructions don't follow the standard pattern because they do stuff
        // with the stack or pc or whatever, deal with them first


        // Compiler / macro gets confused with functions / function pointers
        // Have to give it a hint to coerce the functions into pointers instead of function types

        
        match instrs[i].name {
            BRK => {
                instrs[i].ops = vec![
                    vec![DUMMY_READ_FROM_PC, increment_pc],
                    vec![push_upper_pc_to_stack, decrement_s],
                    vec![push_lower_pc_to_stack, decrement_s],
                    vec![push_p_to_stack_with_brk_flag, decrement_s],
                    vec![fetch_lower_pc_from_interrupt_vector],
                    vec![fetch_upper_pc_from_interrupt_vector],
                ];
                continue;
            }
            RTI => {
                instrs[i].ops = vec![
                    vec![DUMMY_READ_FROM_PC],
                    vec![increment_s],
                    vec![pull_p_from_stack, increment_s],
                    vec![pull_lower_pc_from_stack, increment_s],
                    vec![pull_upper_pc_from_stack],
                ];
                continue;
            }
            RTS => {
                instrs[i].ops = vec![
                    vec![DUMMY_READ_FROM_PC],
                    vec![increment_s],
                    vec![pull_lower_pc_from_stack, increment_s],
                    vec![pull_upper_pc_from_stack],
                    vec![increment_pc],
                ];
                continue;
            }
            JSR => {
                instrs[i].ops = vec![
                    vec![fetch_lower_address_from_pc, increment_pc],
                    vec![none],
                    vec![push_upper_pc_to_stack, decrement_s],
                    vec![push_lower_pc_to_stack, decrement_s],
                    vec![fetch_upper_address_from_pc, copy_address_to_pc], 
                ];
                continue;
            }
            PHA => {
                instrs[i].ops = vec![
                    vec![DUMMY_READ_FROM_PC],
                    vec![push_a_to_stack, decrement_s],
                ];
                continue;
            }
            PHP => {
                instrs[i].ops = vec![
                    vec![DUMMY_READ_FROM_PC],
                    vec![push_p_to_stack, decrement_s],
                ];
                continue;
            }
            PLA => {
                instrs[i].ops = vec![
                    vec![DUMMY_READ_FROM_PC],
                    vec![increment_s],
                    vec![pull_a_from_stack],
                ];
                continue;
            }
            PLP => {
                instrs[i].ops = vec![
                    vec![DUMMY_READ_FROM_PC],
                    vec![increment_s],
                    vec![pull_p_from_stack],
                ];
                continue;
            }
            JMP => {
                match instrs[i].mode {
                    Absolute => {
                        instrs[i].ops = vec![
                            vec![fetch_lower_address_from_pc, increment_pc],
                            vec![fetch_upper_address_from_pc, copy_address_to_pc],
                        ];
                        continue;
                    }
                    AbsoluteI => {
                        instrs[i].ops = vec![
                            vec![fetch_lower_pointer_address_from_pc, increment_pc],
                            vec![fetch_upper_pointer_address_from_pc, increment_pc],
                            vec![fetch_lower_address_from_pointer],
                            vec![fetch_upper_address_from_pointer, copy_address_to_pc]
                        ];
                        continue;
                    }
                    _ => panic!(),
                }
            }
            _ => (),
        };
        


        // Didn't match with any of the previous instructions



        let addressing_mode_cycles = match instrs[i].mode {
            Accumulator | Implied => {
                let ops: Vec<Vec<fn(&mut Nes)>> = vec![
                    vec![DUMMY_READ_FROM_PC],
                ];
                ops
            }
            Immediate => {
                let ops: Vec<Vec<fn(&mut Nes)>> = vec![
                    vec![fetch_immediate_from_pc, increment_pc],
                ];
                ops
            }
            ZeroPage => {
                let ops: Vec<Vec<fn(&mut Nes)>> = vec![
                    vec![fetch_lower_address_from_pc, increment_pc],
                ];
                ops
            }
            // need to remember to zero out the address register between instructions
            ZeroPageX => {
                let ops: Vec<Vec<fn(&mut Nes)>> = vec![
                    vec![fetch_lower_address, increment_pc],
                    vec![dummy_fetch, add_x_to_lower_address],
                ];
            }
            ZeroPageY => {
                let ops: Vec<Vec<fn(&mut Nes)>> = vec![
                    vec![fetch_lower_address, increment_pc],
                    vec![dummy_fetch, add_y_to_lower_address],
                ];
            }
            Absolute => {
                let ops: Vec<Vec<fn(&mut Nes)>> = vec![
                    vec![fetch_lower_address, increment_pc],
                    vec![fetch_upper_address, increment_pc],
                ];
            }
            AbsoluteX => {
                let ops: Vec<Vec<fn(&mut Nes)>> = vec![
                    vec![fetch_lower_address, increment_pc],
                    vec![fetch_upper_address, add_x_to_lower_address, increment_pc],
                    vec![read_from_address, fix_upper_address],
                ];
            }
            AbsoluteY => {
                let ops: Vec<Vec<fn(&mut Nes)>> = vec![
                    vec![fetch_lower_address, increment_pc],
                    vec![fetch_upper_address, add_y_to_lower_address, increment_pc],
                    vec![read_from_address, fix_upper_address],
                ];
            }
            IndirectX => {
                let ops: Vec<Vec<fn(&mut Nes)>> = vec![
                    vec![fetch_lower_address, increment_pc],
                    vec![read_from_address, add_x_to_lower_address], // dummy read!
                    vec![fetch_lower_pointer_address],
                    vec![fetch_upper_pointer_address],
                ];
            }
            IndirectY => {
                let ops: Vec<Vec<fn(&mut Nes)>> = vec![
                    vec![fetch_lower_address, increment_pc],
                    vec![fetch_lower_pointer_address],
                    vec![fetch_upper_pointer_address, add_y_to_lower_address],
                    vec![read_from_address, fix_upper_address],
                ];
            }
            _ => {}
        }





        // These are unique and don't conform to the read/write/rmw things
        // Do these first, then continue if the instruction isn't one of these

        // once this is working, turn slightly different functions into closures
        // if it's readable
        // if it's not readable, duplicate code is fine






        // accumulator addressing is just the two fetch cycles and then the third cycle 
        // which does the operation
        // third cycle happens at the same time as the next cycle fetch

        // Immediate is exactly the same 

        let instruction = match instrs[i].name {
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
            ASL => shift_left,
            EOR => xor,
            AND => and,
            ORA => or,
            ADC => add_data_to_a,
            SBC => sub_data_from_a,
            DEX => decrement_x,
            DEY => decrement_y,
            DEC => decrement_data,
            INX => increment_x, 
            INY => increment_y,
            INC => increment_data,

            _ => none

        };

    }
    instrs

});



// write a comment next to a function once it's been used in a micro op, then delete the ones
// that haven't been used 
    





// weird memory operations

// please decide on naming conventions 

// upper/lower, low/high, lsb/msb
// read/fetch/set/get

// byte1/2, pc fetch, pointer fetch


fn read_from_pc(nes: &mut Nes) {
    nes.cpu.data = read_mem(nes.cpu.pc, nes);
}



fn copy_address_to_pc(nes: &mut Nes) {
    nes.cpu.pc = nes.cpu.get_address();
}






fn fetch_lower_pointer_address_from_pc(nes: &mut Nes) {
    nes.cpu.lower_pointer = read_mem(nes.cpu.pc, nes);
}
fn fetch_upper_pointer_address_from_pc(nes: &mut Nes) {
    nes.cpu.upper_pointer = read_mem(nes.cpu.pc, nes);
}

fn fetch_lower_address_from_pointer(nes: &mut Nes) {
    nes.cpu.lower_address = read_mem(nes.cpu.get_pointer(), nes);
    nes.cpu.lower_address = nes.cpu.lower_address.wrapping_add(1); // hello
}
fn fetch_upper_address_from_pointer(nes: &mut Nes) {
    nes.cpu.upper_address = read_mem(nes.cpu.get_pointer(), nes);
}

fn write_to_address(value: u8, nes: &mut Nes) {
    write_mem(nes.cpu.get_address(), value, nes);
}

fn fetch_lower_address_from_pc(nes: &mut Nes) {
    nes.cpu.lower_address = read_mem(nes.cpu.pc, nes);
}

fn fetch_upper_address_from_pc(nes: &mut Nes) {
    nes.cpu.upper_address = read_mem(nes.cpu.pc, nes);
}
fn fetch_immediate_from_pc(nes: &mut Nes) {
    nes.cpu.data = read_mem(nes.cpu.pc, nes);
}



fn read_from_address(nes: &mut Nes) {
    let addr = nes.cpu.get_address();
    nes.cpu.data = read_mem(addr, nes);
}


fn fetch_lower_pc_from_interrupt_vector(nes: &mut Nes) {
    let lower = read_mem(IRQ_VECTOR, nes);
    nes.cpu.set_lower_pc(lower);
}
fn fetch_upper_pc_from_interrupt_vector(nes: &mut Nes) {
    let upper = read_mem(IRQ_VECTOR+1, nes);
    nes.cpu.set_upper_pc(upper);
}




fn fetch_byte2_to_upper_pc(nes: &mut Nes) {
    let upper = read_mem(nes.cpu.pc, nes);
    nes.cpu.set_upper_pc(upper);
}



    
    
const IRQ_VECTOR: u16 = 0xFFFE;
const BREAK_FLAG: u8  = 0b0001_0000;



// Convenience functions 



pub fn was_signed_overflow(a: u8, b: u8, a_plus_b: u8) -> bool {
    // If the sign bits of A and B are the same
    // and the sign bits of A and A+B are different,
    // sign bit was corrupted (there was signed overflow)
    ((!(a ^ b) & (a ^ a_plus_b)) >> 7) == 1
}



fn update_p_nz(val: u8, nes: &mut Nes) {
    nes.cpu.p_n = val > 0x7F;
    nes.cpu.p_z = val == 0;
}

fn shift_left(val: u8, rotate: bool, nes: &mut Nes) -> u8 {
    let prev_carry = nes.cpu.p_c;
    nes.cpu.p_c = get_bit(val, 7);
    (val << 1) | ((prev_carry && rotate) as u8)
}
fn shift_right(val: u8, rotate: bool, nes: &mut Nes) -> u8 {
    let prev_carry = nes.cpu.p_c;
    nes.cpu.p_c = get_bit(val, 0);
    (val >> 1) | (((prev_carry && rotate) as u8) << 7)
}

fn add_with_carry(val: u8, nes: &mut Nes) {
    let (result, carry) = nes.cpu.a.carrying_add(val, nes.cpu.p_c);
    nes.cpu.p_v = was_signed_overflow(nes.cpu.a, val, result);
    nes.cpu.p_c = carry;
    nes.cpu.a = result;  
}
fn add_index_to_lower_address_and_set_carry(index: u8, nes: &mut Nes) {
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

fn fix_upper_address(nes: &mut Nes) {
    if nes.cpu.addr_low_carry {
        nes.cpu.upper_address = nes.cpu.upper_address.wrapping_add(1);
    }
    nes.cpu.addr_low_carry = false;
}
















fn push_lower_pc_to_stack(nes: &mut Nes) {
    push_to_stack(nes.cpu.pc as u8, nes);
}
fn push_upper_pc_to_stack(nes: &mut Nes) {
    push_to_stack((nes.cpu.pc >> 8) as u8, nes);    
}
fn push_p_to_stack_with_brk_flag(nes: &mut Nes) {
    push_to_stack(nes.cpu.get_p() | BREAK_FLAG, nes);
}
fn push_p_to_stack(nes: &mut Nes) {
    push_to_stack(nes.cpu.get_p(), nes);
}
fn push_a_to_stack(nes: &mut Nes) {
    push_to_stack(nes.cpu.a, nes);
}



fn pull_lower_pc_from_stack(nes: &mut Nes) {
    let lower_pc = pull_from_stack(nes);
    nes.cpu.set_lower_pc(lower_pc);
}
fn pull_upper_pc_from_stack(nes: &mut Nes) {
    let upper_pc = pull_from_stack(nes);
    nes.cpu.set_upper_pc(upper_pc);
}
fn pull_p_from_stack(nes: &mut Nes) {
    let status_reg = pull_from_stack(nes);
    nes.cpu.set_p(status_reg);
}
fn pull_a_from_stack(nes: &mut Nes) {
    let a_reg = pull_from_stack(nes);
    nes.cpu.a = a_reg;
}






// Register operations

fn increment_pc(nes: &mut Nes) {
    nes.cpu.pc = nes.cpu.pc.wrapping_add(1);
}

fn increment_s(nes: &mut Nes) {
    nes.cpu.s = nes.cpu.s.wrapping_add(1);
}
fn increment_x(nes: &mut Nes) {
    nes.cpu.x = nes.cpu.x.wrapping_add(1);
}
fn increment_y(nes: &mut Nes) {
    nes.cpu.y = nes.cpu.y.wrapping_add(1);
}
fn increment_data(nes: &mut Nes) {
    nes.cpu.data = nes.cpu.data.wrapping_add(1);
}


fn decrement_s(nes: &mut Nes) {
    nes.cpu.s = nes.cpu.s.wrapping_sub(1);
}
fn decrement_x(nes: &mut Nes) {
    nes.cpu.x = nes.cpu.x.wrapping_sub(1);
}
fn decrement_y(nes: &mut Nes) {
    nes.cpu.y = nes.cpu.y.wrapping_sub(1);
}
fn decrement_data(nes: &mut Nes) {
    nes.cpu.data = nes.cpu.data.wrapping_sub(1);
}

fn add_x_to_lower_address(nes: &mut Nes) {
    add_index_to_lower_address_and_set_carry(nes.cpu.x, nes);
}

fn add_y_to_lower_address(nes: &mut Nes) {
    add_index_to_lower_address_and_set_carry(nes.cpu.y, nes);
}

fn copy_data_to_a(nes: &mut Nes) {
    nes.cpu.a = nes.cpu.data;
}
fn copy_data_to_x(nes: &mut Nes) {
    nes.cpu.x = nes.cpu.data;
}
fn copy_data_to_y(nes: &mut Nes) {
    nes.cpu.y = nes.cpu.data;
}

fn xor_data_with_a(nes: &mut Nes) {
    nes.cpu.a ^= nes.cpu.data;
}

fn and_data_with_a(nes: &mut Nes) {
    nes.cpu.a &= nes.cpu.data;
}

fn or_data_with_a(nes: &mut Nes) {
    nes.cpu.a |= nes.cpu.data;
}

fn add_data_to_a(nes: &mut Nes) {
    add_with_carry(nes.cpu.data, nes);
}

fn sub_data_from_a(nes: &mut Nes) {
    add_with_carry(!nes.cpu.data, nes);
}

fn none(nes: &mut Nes) {
    // I could have used Option and None, but would have just made things too wordy
}









// INSTRUCTION IMPLEMENTATIONS


fn load_a(nes: &mut Nes) {
    nes.cpu.a = nes.cpu.data;
    update_p_nz(nes.cpu.a, nes);
}
fn load_x(nes: &mut Nes) {
    nes.cpu.x = nes.cpu.data;
    update_p_nz(nes.cpu.x, nes);
}
fn load_y(nes: &mut Nes) {
    nes.cpu.y = nes.cpu.data;
    update_p_nz(nes.cpu.y, nes);
}


fn store_a(nes: &mut Nes) {
    write_to_address(nes.cpu.a, nes);
}
fn store_x(nes: &mut Nes) {
    write_to_address(nes.cpu.x, nes);
}
fn store_y(nes: &mut Nes) {
    write_to_address(nes.cpu.y, nes);
}


fn xor(nes: &mut Nes) {
    nes.cpu.a ^= nes.cpu.data;
}
fn or(nes: &mut Nes) {
    nes.cpu.a |= nes.cpu.data;
}
fn and(nes: &mut Nes) {
    nes.cpu.a &= nes.cpu.data;
}


fn transfer_a_to_x(nes: &mut Nes) {
    nes.cpu.x = nes.cpu.a;
    update_p_nz(nes.cpu.x, nes);
}
fn transfer_a_to_y(nes: &mut Nes) {
    nes.cpu.y = nes.cpu.a;
    update_p_nz(nes.cpu.y, nes);
}
fn transfer_s_to_x(nes: &mut Nes) {
    nes.cpu.x = nes.cpu.s;
    update_p_nz(nes.cpu.x, nes);
}
fn transfer_x_to_a(nes: &mut Nes) {
    nes.cpu.a = nes.cpu.x;
    update_p_nz(nes.cpu.a, nes);
}
fn transfer_y_to_a(nes: &mut Nes) {
    nes.cpu.a = nes.cpu.y;
    update_p_nz(nes.cpu.a, nes);
}
fn transfer_x_to_s(nes: &mut Nes) {
    nes.cpu.s = nes.cpu.x;
}
    
    

/*
Visual6502 can tell you everything! But not necessarily in digestible form.

I think it's only barely true to speak of the 6502 as pipelined. The only overlap which saves cycles is when the final cycle of an instruction is an internal operation, which will then overlap with the next instruction's fetch. But there is no buffer for previously fetched information, so there's never a gain from the processor already having read the next opcode byte, which it sometimes will have done. And if the final cycle of an instruction is a write, it can't overlap with the subsequent fetch anyway.

For example,
INX
INY
will take four cycles, not six. There's a gain of two cycles, over an even simpler fetch-decode-execute machine(*), from the fact that the internal operation of writing back the modified register value overlaps with the fetch of the following instruction, which happens for both the INX and the INY.

But there is no gain from the fact that the INY byte was already read during the second cycle of the INX - the INY is read again and then decoded. Every instruction reads a subsequent byte during the decode cycle, which is used as an operand if it is needed, and which therefore constitutes a gain, but the byte is never fed into the instruction decoder even if it turns out that the present instruction was a single byte.

(*) I think the Z80, which is clocked faster at any given technology, does use a clock cycle for each step of an instruction and, if that's right, is even less pipelined than the 6502. But because it's clocked faster, that's not a noticeable net loss. In the Z80, a memory access takes several clock cycles. (We don't have a visual Z80 to investigate, although Ken Shirriff has done some detailed analyses: see http://www.righto.com/search/label/Z-80)
I think later reimplementations of the Z80 gained more performance than the slightly improved later implementations of the 6502 which saved a cycle here and there - there was more slack to be taken up.
As we know, later descendants of the 8080 put in successively more elaborate mechanisms to become much much more productive per clock cycle. Having multiple instruction decoders working in parallel on the available pre-fetched instruction bytes is just the start of it. A quick search indicates 3 instructions per cycle is an achievable peak value, with half of that being a more likely best case.

Cheers
Ed

*/

// https://retrocomputing.stackexchange.com/questions/145/why-does-6502-indexed-lda-take-an-extra-cycle-at-page-boundaries
// 6502 only has 8-bit adder

// I totally need to try this, sounds really cool
// https://doc.rust-lang.org/rustc/profile-guided-optimization.html
