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

/*
Immediate
LDA F4

Zero page
LDA MEM[AB]
LDA MEM[AB] = F4

Zero page X/Y
LDA MEM[B1]
LDA MEM[AB + Y]
LDA MEM[AB + Y] = F4
LDA MEM[AB + Y(06)] = F4
LDA MEM[AB + Y(06) = B1] = F4
LDA MEM[AB + Y(06) mod 100 = B1] = F4

Absolute
LDA MEM[89AB]
LDA MEM[89AB] = F4

Absolute X/Y
LDA MEM[89AB + Y]
LDA MEM[89AB + Y] = F4
LDA MEM[89AB + Y(06)] = F4
LDA MEM[89AB + Y(06) = 89B1] = F4

Indirect X
LDA MEM[ ZP[AB+X,+1] = 89B1 ] = F4


Indirect Y
LDA MEM[ MEM[AB+1]::MEM[AB] + Y ]
LDA MEM[ MEM[AB+1]::MEM[AB] + Y(06) ]
LDA MEM[ ZP[AB,+1] + Y(06) = 89AB ]

Attributes:
 - What's in memory there (right at the end)
 - What's the value of that register


Columns should be like this

Address of instruction
Instruction
Value being used (either immediate or read from memory)

Address value was retrieved from
So maybe this could be shown like
89B1 = 89AB + Y(06)





 */

use serde::{Deserialize, Serialize};
use std::{
    fmt::Error,
    fmt::{self, Formatter},
};

use Category::*;
use Mode::*;
use Name::*;

use super::operation_funcs::*;
use crate::nes::Nes;

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct Instruction {
    pub name: Name,
    pub mode: Mode,
    pub category: Category,
}

impl Default for Instruction {
    fn default() -> Self {
        Instruction {
            name: NOP,
            mode: Implied,
            category: Unimplemented,
        }
    }
}

impl fmt::Debug for Instruction {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        f.debug_struct("Instruction")
            .field("name", &self.name)
            .field("mode", &self.mode)
            .field("category", &self.category)
            .finish()
    }
}

impl Instruction {
    pub fn does_interrupt_poll_early(&self) -> bool {
        self.category == NonMemory || self.mode == Immediate || self.name == NOP || self.name == PLP
    }
    pub fn number_of_operands(&self) -> u8 {
        match self.mode {
            Accumulator | Implied => 0,
            Absolute | AbsoluteX | AbsoluteY | AbsoluteI => 2,
            _ => 1,
        }
    }

    pub fn address_resolution_cycles(&self) -> i8 {
        match self.mode {
            Immediate => 1,
            ZeroPage => 1,
            ZeroPageX => 2,
            ZeroPageY => 2,
            Absolute => 2,
            AbsoluteX => 2,
            AbsoluteY => 2,
            IndirectX => 4,
            IndirectY => 3,
            _ => 0,
        }
    }

    pub fn is_unofficial(&self) -> bool {
        matches!(
            self.name,
            LAS | LAX
                | SAX
                | SHA
                | SHX
                | SHY
                | SHS
                | ANC
                | ARR
                | ASR
                | DCP
                | RLA
                | RRA
                | SLO
                | SRE
                | XAA
                | JAM
                | ISB
                | SBX
        )
    }
    pub fn func(&self) -> fn(&mut Nes) {
        match (self.mode, self.name) {
            (_, LDA) => load_a,
            (_, LDX) => load_x,
            (_, LDY) => load_y,
            (_, STA) => store_a,
            (_, STX) => store_x,
            (_, STY) => store_y,
            (_, TAX) => transfer_a_to_x,
            (_, TAY) => transfer_a_to_y,
            (_, TSX) => transfer_s_to_x,
            (_, TXA) => transfer_x_to_a,
            (_, TXS) => transfer_x_to_s,
            (_, TYA) => transfer_y_to_a,
            (Accumulator, ASL) => arithmetic_shift_left_accumulator,
            (_, ASL) => arithmetic_shift_left,
            (Accumulator, LSR) => logical_shift_right_accumulator,
            (_, LSR) => logical_shift_right,
            (Accumulator, ROL) => rotate_left_accumulator,
            (_, ROL) => rotate_left,
            (Accumulator, ROR) => rotate_right_accumulator,
            (_, ROR) => rotate_right,
            (_, AND) => and,
            (_, ORA) => or,
            (_, EOR) => xor,
            (_, BIT) => bit,
            (_, ADC) => add_with_carry,
            (_, SBC) => subtract_with_carry,
            (_, DEC) => decrement_memory,
            (_, DEX) => decrement_x,
            (_, DEY) => decrement_y,
            (_, INC) => increment_memory,
            (_, INX) => increment_x,
            (_, INY) => increment_y,
            (_, CMP) => compare_memory_with_a,
            (_, CPX) => compare_memory_with_x,
            (_, CPY) => compare_memory_with_y,
            (_, SEC) => set_carry_flag,
            (_, SED) => set_decimal_flag,
            (_, SEI) => set_interrupt_inhibit_flag,
            (_, CLC) => clear_carry_flag,
            (_, CLD) => clear_decimal_flag,
            (_, CLI) => clear_interrupt_flag,
            (_, CLV) => clear_overflow_flag,
            (_, LAS) => las,
            (_, LAX) => load_a_and_x,
            (_, SAX) => store_a_and_x,
            (_, SHA) => sha,
            (_, SHX) => shx,
            (_, SHY) => shy,
            (_, SHS) => shs,
            (_, ANC) => anc,
            (_, ARR) => arr,
            (_, ASR) => asr,
            (_, DCP) => dec_then_compare,
            (_, RLA) => rla,
            (_, RRA) => rra,
            (_, SLO) => slo,
            (_, SRE) => sre,
            (_, JAM) => jam,
            (_, ISB) => isb,
            (_, SBX) => sbx,
            (_, XAA) => xaa,
            (_, _) => none,
        }
    }
}

#[derive(Default, Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub enum Name {
    // Official opcodes
    LDA,
    LDX,
    LDY,
    STA,
    STX,
    STY,
    TAX,
    TAY,
    TSX,
    TXA,
    TXS,
    TYA,
    PHA,
    PHP,
    PLA,
    PLP,
    ASL,
    LSR,
    ROL,
    ROR,
    AND,
    ORA,
    EOR,
    BIT,
    ADC,
    SBC,
    DEC,
    DEX,
    DEY,
    INC,
    INX,
    INY,
    CMP,
    CPX,
    CPY,
    BCC,
    BCS,
    BEQ,
    BMI,
    BNE,
    BPL,
    BVC,
    BVS,
    SEC,
    SED,
    SEI,
    CLC,
    CLD,
    CLI,
    CLV,
    #[default]
    NOP,
    BRK,
    JMP,
    JSR,
    RTI,
    RTS,
    // Unofficial opcodes
    LAS,
    LAX,
    SAX,
    SHA,
    SHX,
    SHY,
    SHS,
    ANC,
    ARR,
    ASR,
    DCP,
    RLA,
    RRA,
    SLO,
    SRE,
    XAA,
    JAM,
    ISB,
    SBX,
}

#[derive(Default, Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Mode {
    #[default]
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

#[derive(Default, Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Category {
    Control,
    NonMemory,
    Branch,
    #[default]
    Read,
    ReadModifyWrite,
    Write,
    Imm,
    Unimplemented,
}

pub static INSTRUCTIONS: [Instruction; 256] = [
    // 0
    Instruction {
        name: BRK,
        mode: Implied,
        category: Control,
    },
    Instruction {
        name: ORA,
        mode: IndirectX,
        category: Read,
    },
    Instruction {
        name: JAM,
        mode: Implied,
        category: NonMemory,
    },
    Instruction {
        name: SLO,
        mode: IndirectX,
        category: ReadModifyWrite,
    },
    Instruction {
        name: NOP,
        mode: ZeroPage,
        category: Read,
    },
    Instruction {
        name: ORA,
        mode: ZeroPage,
        category: Read,
    },
    Instruction {
        name: ASL,
        mode: ZeroPage,
        category: ReadModifyWrite,
    },
    Instruction {
        name: SLO,
        mode: ZeroPage,
        category: ReadModifyWrite,
    },
    Instruction {
        name: PHP,
        mode: Implied,
        category: Control,
    },
    Instruction {
        name: ORA,
        mode: Immediate,
        category: Imm,
    },
    Instruction {
        name: ASL,
        mode: Accumulator,
        category: NonMemory,
    },
    Instruction {
        name: ANC,
        mode: Immediate,
        category: Imm,
    },
    Instruction {
        name: NOP,
        mode: Absolute,
        category: Read,
    },
    Instruction {
        name: ORA,
        mode: Absolute,
        category: Read,
    },
    Instruction {
        name: ASL,
        mode: Absolute,
        category: ReadModifyWrite,
    },
    Instruction {
        name: SLO,
        mode: Absolute,
        category: ReadModifyWrite,
    },
    // 1
    Instruction {
        name: BPL,
        mode: Relative,
        category: Branch,
    },
    Instruction {
        name: ORA,
        mode: IndirectY,
        category: Read,
    },
    Instruction {
        name: JAM,
        mode: Implied,
        category: NonMemory,
    },
    Instruction {
        name: SLO,
        mode: IndirectY,
        category: ReadModifyWrite,
    },
    Instruction {
        name: NOP,
        mode: ZeroPageX,
        category: Read,
    },
    Instruction {
        name: ORA,
        mode: ZeroPageX,
        category: Read,
    },
    Instruction {
        name: ASL,
        mode: ZeroPageX,
        category: ReadModifyWrite,
    },
    Instruction {
        name: SLO,
        mode: ZeroPageX,
        category: ReadModifyWrite,
    },
    Instruction {
        name: CLC,
        mode: Implied,
        category: NonMemory,
    },
    Instruction {
        name: ORA,
        mode: AbsoluteY,
        category: Read,
    },
    Instruction {
        name: NOP,
        mode: Implied,
        category: NonMemory,
    },
    Instruction {
        name: SLO,
        mode: AbsoluteY,
        category: ReadModifyWrite,
    },
    Instruction {
        name: NOP,
        mode: AbsoluteX,
        category: Read,
    },
    Instruction {
        name: ORA,
        mode: AbsoluteX,
        category: Read,
    },
    Instruction {
        name: ASL,
        mode: AbsoluteX,
        category: ReadModifyWrite,
    },
    Instruction {
        name: SLO,
        mode: AbsoluteX,
        category: ReadModifyWrite,
    },
    // 2
    Instruction {
        name: JSR,
        mode: Absolute,
        category: Control,
    },
    Instruction {
        name: AND,
        mode: IndirectX,
        category: Read,
    },
    Instruction {
        name: JAM,
        mode: Implied,
        category: NonMemory,
    },
    Instruction {
        name: RLA,
        mode: IndirectX,
        category: ReadModifyWrite,
    },
    Instruction {
        name: BIT,
        mode: ZeroPage,
        category: Read,
    },
    Instruction {
        name: AND,
        mode: ZeroPage,
        category: Read,
    },
    Instruction {
        name: ROL,
        mode: ZeroPage,
        category: ReadModifyWrite,
    },
    Instruction {
        name: RLA,
        mode: ZeroPage,
        category: ReadModifyWrite,
    },
    Instruction {
        name: PLP,
        mode: Implied,
        category: Control,
    },
    Instruction {
        name: AND,
        mode: Immediate,
        category: Imm,
    },
    Instruction {
        name: ROL,
        mode: Accumulator,
        category: NonMemory,
    },
    Instruction {
        name: ANC,
        mode: Immediate,
        category: Imm,
    },
    Instruction {
        name: BIT,
        mode: Absolute,
        category: Read,
    },
    Instruction {
        name: AND,
        mode: Absolute,
        category: Read,
    },
    Instruction {
        name: ROL,
        mode: Absolute,
        category: ReadModifyWrite,
    },
    Instruction {
        name: RLA,
        mode: Absolute,
        category: ReadModifyWrite,
    },
    // 3
    Instruction {
        name: BMI,
        mode: Relative,
        category: Branch,
    },
    Instruction {
        name: AND,
        mode: IndirectY,
        category: Read,
    },
    Instruction {
        name: JAM,
        mode: Implied,
        category: NonMemory,
    },
    Instruction {
        name: RLA,
        mode: IndirectY,
        category: ReadModifyWrite,
    },
    Instruction {
        name: NOP,
        mode: ZeroPageX,
        category: Read,
    },
    Instruction {
        name: AND,
        mode: ZeroPageX,
        category: Read,
    },
    Instruction {
        name: ROL,
        mode: ZeroPageX,
        category: ReadModifyWrite,
    },
    Instruction {
        name: RLA,
        mode: ZeroPageX,
        category: ReadModifyWrite,
    },
    Instruction {
        name: SEC,
        mode: Implied,
        category: NonMemory,
    },
    Instruction {
        name: AND,
        mode: AbsoluteY,
        category: Read,
    },
    Instruction {
        name: NOP,
        mode: Implied,
        category: NonMemory,
    },
    Instruction {
        name: RLA,
        mode: AbsoluteY,
        category: ReadModifyWrite,
    },
    Instruction {
        name: NOP,
        mode: AbsoluteX,
        category: Read,
    },
    Instruction {
        name: AND,
        mode: AbsoluteX,
        category: Read,
    },
    Instruction {
        name: ROL,
        mode: AbsoluteX,
        category: ReadModifyWrite,
    },
    Instruction {
        name: RLA,
        mode: AbsoluteX,
        category: ReadModifyWrite,
    },
    // 4
    Instruction {
        name: RTI,
        mode: Implied,
        category: Control,
    },
    Instruction {
        name: EOR,
        mode: IndirectX,
        category: Read,
    },
    Instruction {
        name: JAM,
        mode: Implied,
        category: NonMemory,
    },
    Instruction {
        name: SRE,
        mode: IndirectX,
        category: ReadModifyWrite,
    },
    Instruction {
        name: NOP,
        mode: ZeroPage,
        category: Read,
    },
    Instruction {
        name: EOR,
        mode: ZeroPage,
        category: Read,
    },
    Instruction {
        name: LSR,
        mode: ZeroPage,
        category: ReadModifyWrite,
    },
    Instruction {
        name: SRE,
        mode: ZeroPage,
        category: ReadModifyWrite,
    },
    Instruction {
        name: PHA,
        mode: Implied,
        category: Control,
    },
    Instruction {
        name: EOR,
        mode: Immediate,
        category: Imm,
    },
    Instruction {
        name: LSR,
        mode: Accumulator,
        category: NonMemory,
    },
    Instruction {
        name: ASR,
        mode: Immediate,
        category: Imm,
    },
    Instruction {
        name: JMP,
        mode: Absolute,
        category: Control,
    },
    Instruction {
        name: EOR,
        mode: Absolute,
        category: Read,
    },
    Instruction {
        name: LSR,
        mode: Absolute,
        category: ReadModifyWrite,
    },
    Instruction {
        name: SRE,
        mode: Absolute,
        category: ReadModifyWrite,
    },
    // 5
    Instruction {
        name: BVC,
        mode: Relative,
        category: Branch,
    },
    Instruction {
        name: EOR,
        mode: IndirectY,
        category: Read,
    },
    Instruction {
        name: JAM,
        mode: Implied,
        category: NonMemory,
    },
    Instruction {
        name: SRE,
        mode: IndirectY,
        category: ReadModifyWrite,
    },
    Instruction {
        name: NOP,
        mode: ZeroPageX,
        category: Read,
    },
    Instruction {
        name: EOR,
        mode: ZeroPageX,
        category: Read,
    },
    Instruction {
        name: LSR,
        mode: ZeroPageX,
        category: ReadModifyWrite,
    },
    Instruction {
        name: SRE,
        mode: ZeroPageX,
        category: ReadModifyWrite,
    },
    Instruction {
        name: CLI,
        mode: Implied,
        category: NonMemory,
    },
    Instruction {
        name: EOR,
        mode: AbsoluteY,
        category: Read,
    },
    Instruction {
        name: NOP,
        mode: Implied,
        category: NonMemory,
    },
    Instruction {
        name: SRE,
        mode: AbsoluteY,
        category: ReadModifyWrite,
    },
    Instruction {
        name: NOP,
        mode: AbsoluteX,
        category: Read,
    },
    Instruction {
        name: EOR,
        mode: AbsoluteX,
        category: Read,
    },
    Instruction {
        name: LSR,
        mode: AbsoluteX,
        category: ReadModifyWrite,
    },
    Instruction {
        name: SRE,
        mode: AbsoluteX,
        category: ReadModifyWrite,
    },
    // 6
    Instruction {
        name: RTS,
        mode: Implied,
        category: Control,
    },
    Instruction {
        name: ADC,
        mode: IndirectX,
        category: Read,
    },
    Instruction {
        name: JAM,
        mode: Implied,
        category: NonMemory,
    },
    Instruction {
        name: RRA,
        mode: IndirectX,
        category: ReadModifyWrite,
    },
    Instruction {
        name: NOP,
        mode: ZeroPage,
        category: Read,
    },
    Instruction {
        name: ADC,
        mode: ZeroPage,
        category: Read,
    },
    Instruction {
        name: ROR,
        mode: ZeroPage,
        category: ReadModifyWrite,
    },
    Instruction {
        name: RRA,
        mode: ZeroPage,
        category: ReadModifyWrite,
    },
    Instruction {
        name: PLA,
        mode: Implied,
        category: Control,
    },
    Instruction {
        name: ADC,
        mode: Immediate,
        category: Imm,
    },
    Instruction {
        name: ROR,
        mode: Accumulator,
        category: NonMemory,
    },
    Instruction {
        name: ARR,
        mode: Immediate,
        category: Imm,
    },
    Instruction {
        name: JMP,
        mode: AbsoluteI,
        category: Control,
    },
    Instruction {
        name: ADC,
        mode: Absolute,
        category: Read,
    },
    Instruction {
        name: ROR,
        mode: Absolute,
        category: ReadModifyWrite,
    },
    Instruction {
        name: RRA,
        mode: Absolute,
        category: ReadModifyWrite,
    },
    // 7
    Instruction {
        name: BVS,
        mode: Relative,
        category: Branch,
    },
    Instruction {
        name: ADC,
        mode: IndirectY,
        category: Read,
    },
    Instruction {
        name: JAM,
        mode: Implied,
        category: NonMemory,
    },
    Instruction {
        name: RRA,
        mode: IndirectY,
        category: ReadModifyWrite,
    },
    Instruction {
        name: NOP,
        mode: ZeroPageX,
        category: Read,
    },
    Instruction {
        name: ADC,
        mode: ZeroPageX,
        category: Read,
    },
    Instruction {
        name: ROR,
        mode: ZeroPageX,
        category: ReadModifyWrite,
    },
    Instruction {
        name: RRA,
        mode: ZeroPageX,
        category: ReadModifyWrite,
    },
    Instruction {
        name: SEI,
        mode: Implied,
        category: NonMemory,
    },
    Instruction {
        name: ADC,
        mode: AbsoluteY,
        category: Read,
    },
    Instruction {
        name: NOP,
        mode: Implied,
        category: NonMemory,
    },
    Instruction {
        name: RRA,
        mode: AbsoluteY,
        category: ReadModifyWrite,
    },
    Instruction {
        name: NOP,
        mode: AbsoluteX,
        category: Read,
    },
    Instruction {
        name: ADC,
        mode: AbsoluteX,
        category: Read,
    },
    Instruction {
        name: ROR,
        mode: AbsoluteX,
        category: ReadModifyWrite,
    },
    Instruction {
        name: RRA,
        mode: AbsoluteX,
        category: ReadModifyWrite,
    },
    // 8
    Instruction {
        name: NOP,
        mode: Immediate,
        category: Imm,
    },
    Instruction {
        name: STA,
        mode: IndirectX,
        category: Write,
    },
    Instruction {
        name: NOP,
        mode: Immediate,
        category: Imm,
    },
    Instruction {
        name: SAX,
        mode: IndirectX,
        category: Write,
    },
    Instruction {
        name: STY,
        mode: ZeroPage,
        category: Write,
    },
    Instruction {
        name: STA,
        mode: ZeroPage,
        category: Write,
    },
    Instruction {
        name: STX,
        mode: ZeroPage,
        category: Write,
    },
    Instruction {
        name: SAX,
        mode: ZeroPage,
        category: Write,
    },
    Instruction {
        name: DEY,
        mode: Implied,
        category: NonMemory,
    },
    Instruction {
        name: NOP,
        mode: Immediate,
        category: Imm,
    },
    Instruction {
        name: TXA,
        mode: Implied,
        category: NonMemory,
    },
    Instruction {
        name: XAA,
        mode: Immediate,
        category: Imm,
    },
    Instruction {
        name: STY,
        mode: Absolute,
        category: Write,
    },
    Instruction {
        name: STA,
        mode: Absolute,
        category: Write,
    },
    Instruction {
        name: STX,
        mode: Absolute,
        category: Write,
    },
    Instruction {
        name: SAX,
        mode: Absolute,
        category: Write,
    },
    // 9
    Instruction {
        name: BCC,
        mode: Relative,
        category: Branch,
    },
    Instruction {
        name: STA,
        mode: IndirectY,
        category: Write,
    },
    Instruction {
        name: JAM,
        mode: Implied,
        category: NonMemory,
    },
    Instruction {
        name: SHA,
        mode: IndirectY,
        category: Write,
    },
    Instruction {
        name: STY,
        mode: ZeroPageX,
        category: Write,
    },
    Instruction {
        name: STA,
        mode: ZeroPageX,
        category: Write,
    },
    Instruction {
        name: STX,
        mode: ZeroPageY,
        category: Write,
    },
    Instruction {
        name: SAX,
        mode: ZeroPageY,
        category: Write,
    },
    Instruction {
        name: TYA,
        mode: Implied,
        category: NonMemory,
    },
    Instruction {
        name: STA,
        mode: AbsoluteY,
        category: Write,
    },
    Instruction {
        name: TXS,
        mode: Implied,
        category: NonMemory,
    },
    Instruction {
        name: SHS,
        mode: AbsoluteY,
        category: Write,
    },
    Instruction {
        name: SHY,
        mode: AbsoluteX,
        category: Write,
    },
    Instruction {
        name: STA,
        mode: AbsoluteX,
        category: Write,
    },
    Instruction {
        name: SHX,
        mode: AbsoluteY,
        category: Write,
    },
    Instruction {
        name: SHA,
        mode: AbsoluteY,
        category: Write,
    },
    // A
    Instruction {
        name: LDY,
        mode: Immediate,
        category: Imm,
    },
    Instruction {
        name: LDA,
        mode: IndirectX,
        category: Read,
    },
    Instruction {
        name: LDX,
        mode: Immediate,
        category: Imm,
    },
    Instruction {
        name: LAX,
        mode: IndirectX,
        category: Read,
    },
    Instruction {
        name: LDY,
        mode: ZeroPage,
        category: Read,
    },
    Instruction {
        name: LDA,
        mode: ZeroPage,
        category: Read,
    },
    Instruction {
        name: LDX,
        mode: ZeroPage,
        category: Read,
    },
    Instruction {
        name: LAX,
        mode: ZeroPage,
        category: Read,
    },
    Instruction {
        name: TAY,
        mode: Implied,
        category: NonMemory,
    },
    Instruction {
        name: LDA,
        mode: Immediate,
        category: Imm,
    },
    Instruction {
        name: TAX,
        mode: Implied,
        category: NonMemory,
    },
    Instruction {
        name: LAX,
        mode: Immediate,
        category: Imm,
    },
    Instruction {
        name: LDY,
        mode: Absolute,
        category: Read,
    },
    Instruction {
        name: LDA,
        mode: Absolute,
        category: Read,
    },
    Instruction {
        name: LDX,
        mode: Absolute,
        category: Read,
    },
    Instruction {
        name: LAX,
        mode: Absolute,
        category: Read,
    },
    // B
    Instruction {
        name: BCS,
        mode: Relative,
        category: Branch,
    },
    Instruction {
        name: LDA,
        mode: IndirectY,
        category: Read,
    },
    Instruction {
        name: JAM,
        mode: Implied,
        category: NonMemory,
    },
    Instruction {
        name: LAX,
        mode: IndirectY,
        category: Read,
    },
    Instruction {
        name: LDY,
        mode: ZeroPageX,
        category: Read,
    },
    Instruction {
        name: LDA,
        mode: ZeroPageX,
        category: Read,
    },
    Instruction {
        name: LDX,
        mode: ZeroPageY,
        category: Read,
    },
    Instruction {
        name: LAX,
        mode: ZeroPageY,
        category: Read,
    },
    Instruction {
        name: CLV,
        mode: Implied,
        category: NonMemory,
    },
    Instruction {
        name: LDA,
        mode: AbsoluteY,
        category: Read,
    },
    Instruction {
        name: TSX,
        mode: Implied,
        category: NonMemory,
    },
    Instruction {
        name: LAS,
        mode: AbsoluteY,
        category: Read,
    },
    Instruction {
        name: LDY,
        mode: AbsoluteX,
        category: Read,
    },
    Instruction {
        name: LDA,
        mode: AbsoluteX,
        category: Read,
    },
    Instruction {
        name: LDX,
        mode: AbsoluteY,
        category: Read,
    },
    Instruction {
        name: LAX,
        mode: AbsoluteY,
        category: Read,
    },
    // C
    Instruction {
        name: CPY,
        mode: Immediate,
        category: Imm,
    },
    Instruction {
        name: CMP,
        mode: IndirectX,
        category: Read,
    },
    Instruction {
        name: NOP,
        mode: Immediate,
        category: Imm,
    },
    Instruction {
        name: DCP,
        mode: IndirectX,
        category: ReadModifyWrite,
    },
    Instruction {
        name: CPY,
        mode: ZeroPage,
        category: Read,
    },
    Instruction {
        name: CMP,
        mode: ZeroPage,
        category: Read,
    },
    Instruction {
        name: DEC,
        mode: ZeroPage,
        category: ReadModifyWrite,
    },
    Instruction {
        name: DCP,
        mode: ZeroPage,
        category: ReadModifyWrite,
    },
    Instruction {
        name: INY,
        mode: Implied,
        category: NonMemory,
    },
    Instruction {
        name: CMP,
        mode: Immediate,
        category: Imm,
    },
    Instruction {
        name: DEX,
        mode: Implied,
        category: NonMemory,
    },
    Instruction {
        name: SBX,
        mode: Immediate,
        category: Imm,
    },
    Instruction {
        name: CPY,
        mode: Absolute,
        category: Read,
    },
    Instruction {
        name: CMP,
        mode: Absolute,
        category: Read,
    },
    Instruction {
        name: DEC,
        mode: Absolute,
        category: ReadModifyWrite,
    },
    Instruction {
        name: DCP,
        mode: Absolute,
        category: ReadModifyWrite,
    },
    // D
    Instruction {
        name: BNE,
        mode: Relative,
        category: Branch,
    },
    Instruction {
        name: CMP,
        mode: IndirectY,
        category: Read,
    },
    Instruction {
        name: JAM,
        mode: Implied,
        category: NonMemory,
    },
    Instruction {
        name: DCP,
        mode: IndirectY,
        category: ReadModifyWrite,
    },
    Instruction {
        name: NOP,
        mode: ZeroPageX,
        category: Read,
    },
    Instruction {
        name: CMP,
        mode: ZeroPageX,
        category: Read,
    },
    Instruction {
        name: DEC,
        mode: ZeroPageX,
        category: ReadModifyWrite,
    },
    Instruction {
        name: DCP,
        mode: ZeroPageX,
        category: ReadModifyWrite,
    },
    Instruction {
        name: CLD,
        mode: Implied,
        category: NonMemory,
    },
    Instruction {
        name: CMP,
        mode: AbsoluteY,
        category: Read,
    },
    Instruction {
        name: NOP,
        mode: Implied,
        category: NonMemory,
    },
    Instruction {
        name: DCP,
        mode: AbsoluteY,
        category: ReadModifyWrite,
    },
    Instruction {
        name: NOP,
        mode: AbsoluteX,
        category: Read,
    },
    Instruction {
        name: CMP,
        mode: AbsoluteX,
        category: Read,
    },
    Instruction {
        name: DEC,
        mode: AbsoluteX,
        category: ReadModifyWrite,
    },
    Instruction {
        name: DCP,
        mode: AbsoluteX,
        category: ReadModifyWrite,
    },
    // E
    Instruction {
        name: CPX,
        mode: Immediate,
        category: Imm,
    },
    Instruction {
        name: SBC,
        mode: IndirectX,
        category: Read,
    },
    Instruction {
        name: NOP,
        mode: Immediate,
        category: Imm,
    },
    Instruction {
        name: ISB,
        mode: IndirectX,
        category: ReadModifyWrite,
    },
    Instruction {
        name: CPX,
        mode: ZeroPage,
        category: Read,
    },
    Instruction {
        name: SBC,
        mode: ZeroPage,
        category: Read,
    },
    Instruction {
        name: INC,
        mode: ZeroPage,
        category: ReadModifyWrite,
    },
    Instruction {
        name: ISB,
        mode: ZeroPage,
        category: ReadModifyWrite,
    },
    Instruction {
        name: INX,
        mode: Implied,
        category: NonMemory,
    },
    Instruction {
        name: SBC,
        mode: Immediate,
        category: Imm,
    },
    Instruction {
        name: NOP,
        mode: Implied,
        category: NonMemory,
    },
    Instruction {
        name: SBC,
        mode: Immediate,
        category: Imm,
    },
    Instruction {
        name: CPX,
        mode: Absolute,
        category: Read,
    },
    Instruction {
        name: SBC,
        mode: Absolute,
        category: Read,
    },
    Instruction {
        name: INC,
        mode: Absolute,
        category: ReadModifyWrite,
    },
    Instruction {
        name: ISB,
        mode: Absolute,
        category: ReadModifyWrite,
    },
    // F
    Instruction {
        name: BEQ,
        mode: Relative,
        category: Branch,
    },
    Instruction {
        name: SBC,
        mode: IndirectY,
        category: Read,
    },
    Instruction {
        name: JAM,
        mode: Implied,
        category: NonMemory,
    },
    Instruction {
        name: ISB,
        mode: IndirectY,
        category: ReadModifyWrite,
    },
    Instruction {
        name: NOP,
        mode: ZeroPageX,
        category: Read,
    },
    Instruction {
        name: SBC,
        mode: ZeroPageX,
        category: Read,
    },
    Instruction {
        name: INC,
        mode: ZeroPageX,
        category: ReadModifyWrite,
    },
    Instruction {
        name: ISB,
        mode: ZeroPageX,
        category: ReadModifyWrite,
    },
    Instruction {
        name: SED,
        mode: Implied,
        category: NonMemory,
    },
    Instruction {
        name: SBC,
        mode: AbsoluteY,
        category: Read,
    },
    Instruction {
        name: NOP,
        mode: Implied,
        category: NonMemory,
    },
    Instruction {
        name: ISB,
        mode: AbsoluteY,
        category: ReadModifyWrite,
    },
    Instruction {
        name: NOP,
        mode: AbsoluteX,
        category: Read,
    },
    Instruction {
        name: SBC,
        mode: AbsoluteX,
        category: Read,
    },
    Instruction {
        name: INC,
        mode: AbsoluteX,
        category: ReadModifyWrite,
    },
    Instruction {
        name: ISB,
        mode: AbsoluteX,
        category: ReadModifyWrite,
    },
];
