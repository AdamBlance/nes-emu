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

#[derive(Copy, Clone, Debug)]
pub enum Mode {
    Immediate,
    Accumulator,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
    IndirectX,
    IndirectY,
    Implied,
    Relative,
    AbsoluteI, 
}

#[derive(Copy, Clone)]
pub struct Info {
    pub mode: Mode,
    pub cycles: u8,
}

const UNIMPLEMENTED: Info = Info {
    mode: Mode::Immediate,
    cycles: 0,
};

pub static INSTRUCTION_INFO: [Info; 256] = [
    Info {mode: Mode::Implied,      cycles: 7},  // BRK
    Info {mode: Mode::IndirectX,    cycles: 6},  // ORA
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    Info {mode: Mode::ZeroPage,     cycles: 3},  // ORA
    Info {mode: Mode::ZeroPage,     cycles: 5},  // ASL
    UNIMPLEMENTED,
    Info {mode: Mode::Implied,       cycles: 3},  // PHP
    Info {mode: Mode::Immediate,    cycles: 2},  // ORA
    Info {mode: Mode::Accumulator,  cycles: 2},  // ASL
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    Info {mode: Mode::Absolute,     cycles: 4},  // ORA
    Info {mode: Mode::Absolute,     cycles: 6},  // ASL
    UNIMPLEMENTED,

    Info {mode: Mode::Relative,     cycles: 2},  // BPL
    Info {mode: Mode::IndirectY,    cycles: 5},  // ORA
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    Info {mode: Mode::ZeroPageX,    cycles: 4},  // ORA
    Info {mode: Mode::ZeroPageX,    cycles: 6},  // ASL
    UNIMPLEMENTED,
    Info {mode: Mode::Implied,       cycles: 2},  // CLC
    Info {mode: Mode::AbsoluteY,    cycles: 4},  // ORA
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    Info {mode: Mode::AbsoluteX,    cycles: 4},  // ORA
    Info {mode: Mode::AbsoluteX,    cycles: 7},  // ASL
    UNIMPLEMENTED,

    Info {mode: Mode::Absolute,     cycles: 6},  // JSR
    Info {mode: Mode::IndirectX,    cycles: 6},  // AND
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    Info {mode: Mode::ZeroPage,      cycles: 3},  // BIT
    Info {mode: Mode::ZeroPage,     cycles: 3},  // AND
    Info {mode: Mode::ZeroPage,     cycles: 5},  // ROL
    UNIMPLEMENTED,
    Info {mode: Mode::Implied,       cycles: 4},  // PLP
    Info {mode: Mode::Immediate,    cycles: 2},  // AND
    Info {mode: Mode::Accumulator,  cycles: 2},  // ROL
    UNIMPLEMENTED,
    Info {mode: Mode::Absolute,      cycles: 4},  // BIT
    Info {mode: Mode::Absolute,     cycles: 4},  // AND
    Info {mode: Mode::Absolute,     cycles: 6},  // ROL
    UNIMPLEMENTED,

    Info {mode: Mode::Relative,     cycles: 2},  // BMI
    Info {mode: Mode::IndirectY,    cycles: 5},  // AND
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    Info {mode: Mode::ZeroPageX,    cycles: 4},  // AND
    Info {mode: Mode::ZeroPageX,    cycles: 6},  // ROL
    UNIMPLEMENTED,
    Info {mode: Mode::Implied,       cycles: 2},  // SEC
    Info {mode: Mode::AbsoluteY,    cycles: 4},  // AND
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    Info {mode: Mode::AbsoluteX,    cycles: 4},  // AND
    Info {mode: Mode::AbsoluteX,    cycles: 7},  // ROL
    UNIMPLEMENTED,

    Info {mode: Mode::Implied,       cycles: 6},  // RTI
    Info {mode: Mode::IndirectX,    cycles: 6},  // EOR
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    Info {mode: Mode::ZeroPage,     cycles: 3},  // EOR
    Info {mode: Mode::ZeroPage,     cycles: 5},  // LSR
    UNIMPLEMENTED,
    Info {mode: Mode::Implied,       cycles: 3},  // PHA
    Info {mode: Mode::Immediate,    cycles: 2},  // EOR
    Info {mode: Mode::Accumulator,  cycles: 2},  // LSR
    UNIMPLEMENTED,
    Info {mode: Mode::Absolute,     cycles: 3},  // JMP
    Info {mode: Mode::Absolute,     cycles: 4},  // EOR
    Info {mode: Mode::Absolute,     cycles: 6},  // LSR
    UNIMPLEMENTED,

    Info {mode: Mode::Relative,     cycles: 2},  // BVC
    Info {mode: Mode::IndirectY,    cycles: 5},  // EOR
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    Info {mode: Mode::ZeroPageX,    cycles: 4},  // EOR
    Info {mode: Mode::ZeroPageX,    cycles: 6},  // LSR
    UNIMPLEMENTED,
    Info {mode: Mode::Implied,       cycles: 2},  // CLI
    Info {mode: Mode::AbsoluteY,    cycles: 4},  // EOR
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    Info {mode: Mode::AbsoluteX,    cycles: 4},  // EOR
    Info {mode: Mode::AbsoluteX,    cycles: 7},  // LSR
    UNIMPLEMENTED,

    Info {mode: Mode::Implied,       cycles: 6},  // RTS
    Info {mode: Mode::IndirectX,    cycles: 6},  // ADC
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    Info {mode: Mode::ZeroPageX,    cycles: 4},  // ADC
    Info {mode: Mode::ZeroPageX,    cycles: 6},  // ROR
    UNIMPLEMENTED,
    Info {mode: Mode::Implied,      cycles: 4},  // PLA
    Info {mode: Mode::Immediate,    cycles: 2},  // ADC
    Info {mode: Mode::Accumulator,  cycles: 2},  // ROR
    UNIMPLEMENTED,
    Info {mode: Mode::AbsoluteI,    cycles: 5},  // JMP
    Info {mode: Mode::Absolute,     cycles: 4},  // ADC
    Info {mode: Mode::Absolute,     cycles: 6},  // ROR
    UNIMPLEMENTED,

    Info {mode: Mode::Relative,     cycles: 2},  // BVS
    Info {mode: Mode::IndirectY,    cycles: 5},  // ADC
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    Info {mode: Mode::ZeroPageX,    cycles: 4},  // ADC
    Info {mode: Mode::ZeroPageX,    cycles: 6},  // ROR
    UNIMPLEMENTED,
    Info {mode: Mode::Implied,       cycles: 2},  // SEI
    Info {mode: Mode::AbsoluteY,    cycles: 4},  // ADC
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    Info {mode: Mode::AbsoluteX,    cycles: 4},  // ADC
    Info {mode: Mode::AbsoluteX,    cycles: 7},  // ROR
    UNIMPLEMENTED,

    UNIMPLEMENTED,
    Info {mode: Mode::IndirectX,     cycles: 6},  // STA
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    Info {mode: Mode::ZeroPage,      cycles: 3},  // STY
    Info {mode: Mode::ZeroPage,      cycles: 3},  // STA
    Info {mode: Mode::ZeroPage,      cycles: 3},  // STX
    UNIMPLEMENTED,
    Info {mode: Mode::Implied,      cycles: 2},  // DEY
    UNIMPLEMENTED,
    Info {mode: Mode::Implied,      cycles: 2},  // TXA
    UNIMPLEMENTED,
    Info {mode: Mode::Absolute,      cycles: 4},  // STY
    Info {mode: Mode::Absolute,      cycles: 4},  // STA
    Info {mode: Mode::Absolute,      cycles: 4},  // STX
    UNIMPLEMENTED,

    Info {mode: Mode::Relative,     cycles: 2},  // BCC
    Info {mode: Mode::IndirectY,     cycles: 6},  // STA
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    Info {mode: Mode::ZeroPageX,     cycles: 4},  // STY
    Info {mode: Mode::ZeroPageX,     cycles: 4},  // STA
    Info {mode: Mode::ZeroPageX,     cycles: 4},  // STX
    UNIMPLEMENTED,
    Info {mode: Mode::Implied,      cycles: 2},  // TYA
    Info {mode: Mode::AbsoluteY,     cycles: 5},  // STA
    Info {mode: Mode::Implied,       cycles: 2},  // TXS
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    Info {mode: Mode::AbsoluteX,     cycles: 5},  // STA
    UNIMPLEMENTED,
    UNIMPLEMENTED,

    Info {mode: Mode::Immediate,    cycles: 2},  // LDY
    Info {mode: Mode::IndirectX,    cycles: 6},  // LDA
    Info {mode: Mode::Immediate,    cycles: 2},  // LDX
    UNIMPLEMENTED,
    Info {mode: Mode::ZeroPage,     cycles: 3},  // LDY
    Info {mode: Mode::ZeroPage,     cycles: 3},  // LDA
    Info {mode: Mode::ZeroPage,     cycles: 3},  // LDX
    UNIMPLEMENTED,
    Info {mode: Mode::Implied,      cycles: 2},  // TAY
    Info {mode: Mode::Immediate,    cycles: 2},  // LDA
    Info {mode: Mode::Implied,      cycles: 2},  // TAX
    UNIMPLEMENTED,
    Info {mode: Mode::Absolute,     cycles: 4},  // LDY
    Info {mode: Mode::Absolute,     cycles: 4},  // LDA
    Info {mode: Mode::Absolute,     cycles: 4},  // LDX
    UNIMPLEMENTED,

    Info {mode: Mode::Relative,     cycles: 2},  // BCS
    Info {mode: Mode::IndirectY,    cycles: 5},  // LDA
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    Info {mode: Mode::ZeroPageX,    cycles: 4},  // LDY
    Info {mode: Mode::ZeroPageX,    cycles: 4},  // LDA
    Info {mode: Mode::ZeroPageX,    cycles: 4},  // LDX
    UNIMPLEMENTED,
    Info {mode: Mode::Implied,       cycles: 2},  // CLV
    Info {mode: Mode::AbsoluteY,    cycles: 4},  // LDA
    Info {mode: Mode::Implied,      cycles: 2},  // TSX
    UNIMPLEMENTED,
    Info {mode: Mode::AbsoluteX,    cycles: 4},  // LDY
    Info {mode: Mode::AbsoluteX,    cycles: 4},  // LDA
    Info {mode: Mode::AbsoluteX,    cycles: 4},  // LDX
    UNIMPLEMENTED,
    Info {mode: Mode::Immediate,     cycles: 2},  // CPY
    Info {mode: Mode::IndirectX,     cycles: 6},  // CMP
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    Info {mode: Mode::ZeroPage,      cycles: 3},  // CPY
    Info {mode: Mode::ZeroPage,      cycles: 3},  // CMP
    Info {mode: Mode::ZeroPage,     cycles: 6},  // DEC
    UNIMPLEMENTED,
    Info {mode: Mode::Implied,      cycles: 2},  // INY
    Info {mode: Mode::Immediate,     cycles: 2},  // CMP
    Info {mode: Mode::Implied,      cycles: 2},  // DEX
    UNIMPLEMENTED,
    Info {mode: Mode::Absolute,      cycles: 4},  // CPY
    Info {mode: Mode::Absolute,      cycles: 4},  // CMP
    Info {mode: Mode::Absolute,     cycles: 6},  // DEC
    UNIMPLEMENTED,
    Info {mode: Mode::Relative,     cycles: 2},  // BNE
    Info {mode: Mode::IndirectY,     cycles: 5},  // CMP
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    Info {mode: Mode::ZeroPageX,     cycles: 4},  // CMP
    Info {mode: Mode::ZeroPageX,    cycles: 6},  // DEC
    UNIMPLEMENTED,
    Info {mode: Mode::Implied,       cycles: 2},  // CLD
    Info {mode: Mode::AbsoluteY,     cycles: 4},  // CMP
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    Info {mode: Mode::AbsoluteX,     cycles: 4},  // CMP
    Info {mode: Mode::AbsoluteX,    cycles: 7},  // DEC
    UNIMPLEMENTED,
    Info {mode: Mode::Immediate,     cycles: 2},  // CPX
    Info {mode: Mode::IndirectX,    cycles: 6},  // SBC
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    Info {mode: Mode::ZeroPage,      cycles: 3},  // CPX
    Info {mode: Mode::ZeroPage,     cycles: 3},  // SBC
    Info {mode: Mode::ZeroPage,     cycles: 5},  // INC
    UNIMPLEMENTED,
    Info {mode: Mode::Implied,      cycles: 2},  // INX
    Info {mode: Mode::Immediate,    cycles: 2},  // SBC
    Info {mode: Mode::Implied,       cycles: 2},  // NOP
    UNIMPLEMENTED,
    Info {mode: Mode::Absolute,      cycles: 4},  // CPX
    Info {mode: Mode::Absolute,     cycles: 4},  // SBC
    Info {mode: Mode::Absolute,     cycles: 6},  // INC
    UNIMPLEMENTED,
    Info {mode: Mode::Relative,     cycles: 2},  // BEQ
    Info {mode: Mode::IndirectY,    cycles: 5},  // SBC
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    Info {mode: Mode::ZeroPageX,    cycles: 4},  // SBC
    Info {mode: Mode::ZeroPageX,    cycles: 6},  // INC
    UNIMPLEMENTED,
    Info {mode: Mode::Implied,       cycles: 2},  // SED
    Info {mode: Mode::AbsoluteY,     cycles: 4},  // SBC
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    Info {mode: Mode::AbsoluteX,    cycles: 4},  // SBC
    Info {mode: Mode::AbsoluteX,    cycles: 7},  // INC
    UNIMPLEMENTED
];
