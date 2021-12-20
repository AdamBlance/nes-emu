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

enum Mode {
    Immediate,
    Accumulator,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
    IndirectX
    IndirectY,
    Implied,
    Relative,
    AbsoluteI, 
}

enum Dest {
    A,
    X,
    Y,
    M,
    PC,
    U,
}
struct Instruction {
    mode: Mode,
    cycles: u8,
}

const UNIMPLEMENTED: Instruction = Instruction {
    mode: Mode::Immediate,
    n_z: false,
    cycles: 0,
}

pub static INSTRUCTION_INFO: [Instruction; 256] = [
    // 0
    Instruction {mode: Mode::Implied,     n_z: false, cycles: 7},  // BRK
    Instruction {mode: Mode::IndirectX,   n_z: true,  cycles: 6},  // ORA
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    Instruction {mode: Mode::ZeroPage,    n_z: true,  cycles: 3},  // ORA
    Instruction {mode: Mode::ZeroPage,    n_z: true,  cycles: 5},  // ASL
    UNIMPLEMENTED,
    Instruction {mode: Mode::Implied,     n_z: false,  cycles: 3},  // PHP
    Instruction {mode: Mode::Immediate,   n_z: true,  cycles: 2},  // ORA
    Instruction {mode: Mode::Accumulator, n_z: true,  cycles: 2},  // ASL
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    Instruction {mode: Mode::Absolute,    n_z: true,  cycles: 4},  // ORA
    Instruction {mode: Mode::Absolute,    n_z: true,  cycles: 6},  // ASL
    UNIMPLEMENTED,

    // 1
    Instruction {mode: Mode::Relative,    n_z: false, cycles: 2},  // BPL
    Instruction {mode: Mode::IndirectY,   n_z: true,  cycles: 5},  // ORA
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    Instruction {mode: Mode::ZeroPageX,   n_z: true,  cycles: 4},  // ORA
    Instruction {mode: Mode::ZeroPageX,   n_z: true,  cycles: 6},  // ASL
    UNIMPLEMENTED,
    Instruction {mode: Mode::Implied,     n_z: false,  cycles: 2},  // CLC
    Instruction {mode: Mode::AbsoluteY,   n_z: true,  cycles: 4},  // ORA
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    Instruction {mode: Mode::AbsoluteX,   n_z: true,  cycles: 4},  // ORA
    Instruction {mode: Mode::AbsoluteX,   n_z: true,  cycles: 7},  // ASL
    UNIMPLEMENTED,

    // 2
    Instruction {mode: Mode::Absolute,    n_z: false, cycles: 6},  // JSR
    Instruction {mode: Mode::IndirectX,   n_z: true,  cycles: 6},  // AND
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    Instruction {mode: Mode::ZeroPage,    n_z: false,  cycles: 3},  // BIT
    Instruction {mode: Mode::ZeroPage,    n_z: true,  cycles: 3},  // AND
    Instruction {mode: Mode::ZeroPage,    n_z: true,  cycles: 5},  // ROL
    UNIMPLEMENTED,
    Instruction {mode: Mode::Implied,     n_z: false,  cycles: 4},  // PLP
    Instruction {mode: Mode::Immediate,   n_z: true,  cycles: 2},  // AND
    Instruction {mode: Mode::Accumulator, n_z: true,  cycles: 2},  // ROL
    UNIMPLEMENTED,
    Instruction {mode: Mode::Absolute,    n_z: false,  cycles: 4},  // BIT
    Instruction {mode: Mode::Absolute,    n_z: true,  cycles: 4},  // AND
    Instruction {mode: Mode::Absolute,    n_z: true,  cycles: 6},  // ROL
    UNIMPLEMENTED,

    // 3
    Instruction {mode: Mode::Relative,    n_z: false, cycles: 2},  // BMI
    Instruction {mode: Mode::IndirectY,   n_z: true,  cycles: 5},  // AND
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    Instruction {mode: Mode::ZeroPageX,   n_z: true,  cycles: 4},  // AND
    Instruction {mode: Mode::ZeroPageX,   n_z: true,  cycles: 6},  // ROL
    UNIMPLEMENTED,
    Instruction {mode: Mode::Implied,     n_z: false,  cycles: 2},  // SEC
    Instruction {mode: Mode::AbsoluteY,   n_z: true,  cycles: 4},  // AND
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    Instruction {mode: Mode::AbsoluteX,   n_z: true,  cycles: 4},  // AND
    Instruction {mode: Mode::AbsoluteX,   n_z: true,  cycles: 7},  // ROL
    UNIMPLEMENTED,
    
    // 4
    Instruction {mode: Mode::Implied,     n_z: false,  cycles: 6},  // RTI
    Instruction {mode: Mode::IndirectX,   n_z: true,  cycles: 6},  // EOR
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    Instruction {mode: Mode::ZeroPage,    n_z: true,  cycles: 3},  // EOR
    Instruction {mode: Mode::ZeroPage,    n_z: true,  cycles: 5},  // LSR
    UNIMPLEMENTED,
    Instruction {mode: Mode::Implied,     n_z: false,  cycles: 3},  // PHA
    Instruction {mode: Mode::Immediate,   n_z: true,  cycles: 2},  // EOR
    Instruction {mode: Mode::Accumulator, n_z: true,  cycles: 2},  // LSR
    UNIMPLEMENTED,
    Instruction {mode: Mode::Absolute,    n_z: false, cycles: 3},  // JMP
    Instruction {mode: Mode::Absolute,    n_z: true,  cycles: 4},  // EOR
    Instruction {mode: Mode::Absolute,    n_z: true,  cycles: 6},  // LSR
    UNIMPLEMENTED,

    // 5
    Instruction {mode: Mode::Relative,    n_z: false, cycles: 2},  // BVC
    Instruction {mode: Mode::IndirectY,   n_z: true,  cycles: 5},  // EOR
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    Instruction {mode: Mode::ZeroPageX,   n_z: true,  cycles: 4},  // EOR
    Instruction {mode: Mode::ZeroPageX,   n_z: true,  cycles: 6},  // LSR
    UNIMPLEMENTED,
    Instruction {mode: Mode::Implied,     n_z: false,  cycles: 2},  // CLI
    Instruction {mode: Mode::AbsoluteY,   n_z: true,  cycles: 4},  // EOR
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    Instruction {mode: Mode::AbsoluteX,   n_z: true,  cycles: 4},  // EOR
    Instruction {mode: Mode::AbsoluteX,   n_z: true,  cycles: 7},  // LSR
    UNIMPLEMENTED,

    // 6
    Instruction {mode: Mode::Implied,     n_z: false,  cycles: 6},  // RTS
    Instruction {mode: Mode::IndirectX,   n_z: true,  cycles: 6},  // ADC
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    Instruction {mode: Mode::ZeroPageX,   n_z: true,  cycles: 4},  // ADC
    Instruction {mode: Mode::ZeroPageX,   n_z: true,  cycles: 6},  // ROR
    UNIMPLEMENTED,
    Instruction {mode: Mode::Implied,     n_z: true,  cycles: 4},  // PLA
    Instruction {mode: Mode::Immediate,   n_z: true,  cycles: 2},  // ADC
    Instruction {mode: Mode::Accumulator, n_z: true,  cycles: 2},  // ROR
    UNIMPLEMENTED,
    Instruction {mode: Mode::AbsoluteI,   n_z: false, cycles: 5},  // JMP
    Instruction {mode: Mode::Absolute,    n_z: true,  cycles: 4},  // ADC
    Instruction {mode: Mode::Absolute,    n_z: true,  cycles: 6},  // ROR
    UNIMPLEMENTED,

    // 7
    Instruction {mode: Mode::Relative,    n_z: false, cycles: 2},  // BVS
    Instruction {mode: Mode::IndirectY,   n_z: true,  cycles: 5},  // ADC
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    Instruction {mode: Mode::ZeroPageX,   n_z: true,  cycles: 4},  // ADC
    Instruction {mode: Mode::ZeroPageX,   n_z: true,  cycles: 6},  // ROR
    Instruction {mode: Mode::Implied,     n_z: false,  cycles: 2},  // SEI
    Instruction {mode: Mode::AbsoluteY,   n_z: true,  cycles: 4},  // ADC
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    Instruction {mode: Mode::AbsoluteX,   n_z: true,  cycles: 4},  // ADC
    Instruction {mode: Mode::AbsoluteX,   n_z: true,  cycles: 7},  // ROR
    UNIMPLEMENTED,

    // 8
    UNIMPLEMENTED,
    Instruction {mode: Mode::IndirectX,   n_z: false,  cycles: 6},  // STA
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    Instruction {mode: Mode::ZeroPage,    n_z: false,  cycles: 3},  // STY
    Instruction {mode: Mode::ZeroPage,    n_z: false,  cycles: 3},  // STA
    Instruction {mode: Mode::ZeroPage,    n_z: false,  cycles: 3},  // STX
    UNIMPLEMENTED,
    Instruction {mode: Mode::Implied,     n_z: true,  cycles: 2},  // DEY
    UNIMPLEMENTED,
    Instruction {mode: Mode::Implied,     n_z: true,  cycles: 2},  // TXA
    UNIMPLEMENTED,
    Instruction {mode: Mode::Absolute,    n_z: false,  cycles: 4},  // STY
    Instruction {mode: Mode::Absolute,    n_z: false,  cycles: 4},  // STA
    Instruction {mode: Mode::Absolute,    n_z: false,  cycles: 4},  // STX
    UNIMPLEMENTED,

    // 9
    Instruction {mode: Mode::Relative,    n_z: false, cycles: 2},  // BCC
    Instruction {mode: Mode::IndirectY,   n_z: false,  cycles: 6},  // STA
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    Instruction {mode: Mode::ZeroPageX,   n_z: false,  cycles: 4},  // STY
    Instruction {mode: Mode::ZeroPageX,   n_z: false,  cycles: 4},  // STA
    Instruction {mode: Mode::ZeroPageX,   n_z: false,  cycles: 4},  // STX
    UNIMPLEMENTED,
    Instruction {mode: Mode::Implied,     n_z: true,  cycles: 2},  // TYA
    Instruction {mode: Mode::AbsoluteY,   n_z: false,  cycles: 5},  // STA
    Instruction {mode: Mode::Implied,     n_z: false,  cycles: 2},  // TXS
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    Instruction {mode: Mode::AbsoluteX,   n_z: false,  cycles: 5},  // STA
    UNIMPLEMENTED,
    UNIMPLEMENTED,

    // A
    Instruction {mode: Mode::Immediate,   n_z: true,  cycles: 2},  // LDY
    Instruction {mode: Mode::IndirectX,   n_z: true,  cycles: 6},  // LDA
    Instruction {mode: Mode::Immediate,   n_z: true,  cycles: 2},  // LDX
    UNIMPLEMENTED,
    Instruction {mode: Mode::ZeroPage,    n_z: true,  cycles: 3},  // LDY
    Instruction {mode: Mode::ZeroPage,    n_z: true,  cycles: 3},  // LDA
    Instruction {mode: Mode::ZeroPage,    n_z: true,  cycles: 3},  // LDX
    UNIMPLEMENTED,
    Instruction {mode: Mode::Implied,     n_z: true,  cycles: 2},  // TAY
    Instruction {mode: Mode::Immediate,   n_z: true,  cycles: 2},  // LDA
    Instruction {mode: Mode::Implied,     n_z: true,  cycles: 2},  // TAX
    UNIMPLEMENTED,
    Instruction {mode: Mode::Absolute,    n_z: true,  cycles: 4},  // LDY
    Instruction {mode: Mode::Absolute,    n_z: true,  cycles: 4},  // LDA
    Instruction {mode: Mode::Absolute,    n_z: true,  cycles: 4},  // LDX
    UNIMPLEMENTED,

    // B
    Instruction {mode: Mode::Relative,    n_z: false, cycles: 2},  // BCS
    Instruction {mode: Mode::IndirectY,   n_z: true,  cycles: 5},  // LDA
    Instruction {mode: Mode::Immediate,   n_z: true,  cycles: 2},  // LDX
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    Instruction {mode: Mode::ZeroPageX,   n_z: true,  cycles: 4},  // LDY
    Instruction {mode: Mode::ZeroPageX,   n_z: true,  cycles: 4},  // LDA
    Instruction {mode: Mode::ZeroPageX,   n_z: true,  cycles: 4},  // LDX
    UNIMPLEMENTED,
    Instruction {mode: Mode::Implied,     n_z: false,  cycles: 2},  // CLV
    Instruction {mode: Mode::AbsoluteY,   n_z: true,  cycles: 4},  // LDA
    Instruction {mode: Mode::Implied,     n_z: true,  cycles: 2},  // TSX
    UNIMPLEMENTED,
    Instruction {mode: Mode::AbsoluteX,   n_z: true,  cycles: 4},  // LDY
    Instruction {mode: Mode::AbsoluteX,   n_z: true,  cycles: 4},  // LDA
    Instruction {mode: Mode::AbsoluteX,   n_z: true,  cycles: 4},  // LDX
    UNIMPLEMENTED,
    
    // C
    Instruction {mode: Mode::Immediate,   n_z: false,  cycles: 2},  // CPY
    Instruction {mode: Mode::IndirectX,   n_z: false,  cycles: 6},  // CMP
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    Instruction {mode: Mode::ZeroPage,    n_z: false,  cycles: 3},  // CPY
    Instruction {mode: Mode::ZeroPage,    n_z: false,  cycles: 3},  // CMP
    Instruction {mode: Mode::ZeroPage,    n_z: true,  cycles: 6},  // DEC
    UNIMPLEMENTED,
    Instruction {mode: Mode::Implied,     n_z: true,  cycles: 2},  // INY
    Instruction {mode: Mode::Immediate,   n_z: false,  cycles: 2},  // CMP
    Instruction {mode: Mode::Implied,     n_z: true,  cycles: 2},  // DEX
    UNIMPLEMENTED,
    Instruction {mode: Mode::Absolute,    n_z: false,  cycles: 4},  // CPY
    Instruction {mode: Mode::Absolute,    n_z: false,  cycles: 4},  // CMP
    Instruction {mode: Mode::Absolute,    n_z: true,  cycles: 6},  // DEC
    UNIMPLEMENTED,

    // D
    Instruction {mode: Mode::Relative,    n_z: false, cycles: 2},  // BNE
    Instruction {mode: Mode::IndirectY,   n_z: false,  cycles: 5},  // CMP
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    Instruction {mode: Mode::ZeroPageX,   n_z: false,  cycles: 4},  // CMP
    Instruction {mode: Mode::ZeroPageX,   n_z: true,  cycles: 6},  // DEC
    UNIMPLEMENTED,
    Instruction {mode: Mode::Implied,     n_z: false,  cycles: 2},  // CLD
    Instruction {mode: Mode::AbsoluteY,   n_z: false,  cycles: 4},  // CMP
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    Instruction {mode: Mode::AbsoluteX,   n_z: false,  cycles: 4},  // CMP
    Instruction {mode: Mode::AbsoluteX,   n_z: true,  cycles: 7},  // DEC
    UNIMPLEMENTED,

    // E
    Instruction {mode: Mode::Immediate,   n_z: false,  cycles: 2},  // CPX
    Instruction {mode: Mode::IndirectX,   n_z: true,  cycles: 6},  // SBC
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    Instruction {mode: Mode::ZeroPage,    n_z: false,  cycles: 3},  // CPX
    Instruction {mode: Mode::ZeroPage,    n_z: true,  cycles: 3},  // SBC
    Instruction {mode: Mode::ZeroPage,    n_z: true,  cycles: 5},  // INC
    UNIMPLEMENTED,
    Instruction {mode: Mode::Implied,     n_z: true,  cycles: 2},  // INX
    Instruction {mode: Mode::Immediate,   n_z: true,  cycles: 2},  // SBC
    Instruction {mode: Mode::Implied,     n_z: false,  cycles: 2},  // NOP
    UNIMPLEMENTED,
    Instruction {mode: Mode::Absolute,    n_z: false,  cycles: 4},  // CPX
    Instruction {mode: Mode::Absolute,    n_z: true,  cycles: 4},  // SBC
    Instruction {mode: Mode::Absolute,    n_z: true,  cycles: 6},  // INC
    UNIMPLEMENTED,

    // F
    Instruction {mode: Mode::Relative,    n_z: false, cycles: 2},  // BEQ
    Instruction {mode: Mode::IndirectY,   n_z: true,  cycles: 5},  // SBC
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    Instruction {mode: Mode::ZeroPageX,   n_z: true,  cycles: 4},  // SBC
    Instruction {mode: Mode::ZeroPageX,   n_z: true,  cycles: 6},  // INC
    UNIMPLEMENTED,
    Instruction {mode: Mode::Implied,     n_z: false,  cycles: 2},  // SED
    Instruction {mode: Mode::AbsoluteY,   n_z: false,  cycles: 4},  // SBC
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    Instruction {mode: Mode::AbsoluteX,   n_z: true,  cycles: 4},  // SBC
    Instruction {mode: Mode::AbsoluteX,   n_z: true,  cycles: 7},  // INC
    UNIMPLEMENTED
];
