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

// https://wiki.nesdev.org/w/index.php?title=Programming_with_unofficial_opcodes

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
    // 0
    Info {mode: Mode::Implied,     cycles: 7},  //  BRK
    Info {mode: Mode::IndirectX,   cycles: 6},  //  ORA
    Info {mode: Mode::Implied,     cycles: 0},  // *JAM
    Info {mode: Mode::IndirectX,   cycles: 8},  // *SLO
    Info {mode: Mode::ZeroPage,    cycles: 3},  // *IGN
    Info {mode: Mode::ZeroPage,    cycles: 3},  //  ORA
    Info {mode: Mode::ZeroPage,    cycles: 5},  //  ASL
    Info {mode: Mode::ZeroPage,    cycles: 5},  // *SLO
    Info {mode: Mode::Implied,     cycles: 3},  //  PHP
    Info {mode: Mode::Immediate,   cycles: 2},  //  ORA
    Info {mode: Mode::Accumulator, cycles: 2},  //  ASL
    Info {mode: Mode::Immediate,   cycles: 2},  // *ANC
    Info {mode: Mode::Absolute,    cycles: 4},  // *IGN
    Info {mode: Mode::Absolute,    cycles: 4},  //  ORA
    Info {mode: Mode::Absolute,    cycles: 6},  //  ASL
    Info {mode: Mode::Absolute,    cycles: 6},  // *SLO

    // 1
    Info {mode: Mode::Relative,    cycles: 2},  //  BPL
    Info {mode: Mode::IndirectY,   cycles: 5},  //  ORA
    Info {mode: Mode::Implied,     cycles: 0},  // *JAM
    Info {mode: Mode::IndirectY,   cycles: 8},  // *SLO
    Info {mode: Mode::ZeroPageX,   cycles: 4},  // *IGN
    Info {mode: Mode::ZeroPageX,   cycles: 4},  //  ORA
    Info {mode: Mode::ZeroPageX,   cycles: 6},  //  ASL
    Info {mode: Mode::ZeroPageX,   cycles: 6},  // *SLO
    Info {mode: Mode::Implied,     cycles: 2},  //  CLC
    Info {mode: Mode::AbsoluteY,   cycles: 4},  //  ORA
    Info {mode: Mode::Implied,     cycles: 2},  // *NOP
    Info {mode: Mode::AbsoluteY,   cycles: 7},  // *SLO
    Info {mode: Mode::AbsoluteX,   cycles: 4},  // *IGN
    Info {mode: Mode::AbsoluteX,   cycles: 4},  //  ORA
    Info {mode: Mode::AbsoluteX,   cycles: 7},  //  ASL
    Info {mode: Mode::AbsoluteX,   cycles: 7},  // *SLO

    // 2
    Info {mode: Mode::Absolute,    cycles: 6},  //  JSR
    Info {mode: Mode::IndirectX,   cycles: 6},  //  AND
    Info {mode: Mode::Implied,     cycles: 0},  // *JAM
    Info {mode: Mode::IndirectX,   cycles: 8},  // *RLA
    Info {mode: Mode::ZeroPage,    cycles: 3},  //  BIT
    Info {mode: Mode::ZeroPage,    cycles: 3},  //  AND
    Info {mode: Mode::ZeroPage,    cycles: 5},  //  ROL
    Info {mode: Mode::ZeroPage,    cycles: 5},  // *RLA
    Info {mode: Mode::Implied,     cycles: 4},  //  PLP
    Info {mode: Mode::Immediate,   cycles: 2},  //  AND
    Info {mode: Mode::Accumulator, cycles: 2},  //  ROL
    Info {mode: Mode::Immediate,   cycles: 2},  // *ANC
    Info {mode: Mode::Absolute,    cycles: 4},  //  BIT
    Info {mode: Mode::Absolute,    cycles: 4},  //  AND
    Info {mode: Mode::Absolute,    cycles: 6},  //  ROL
    Info {mode: Mode::Absolute,    cycles: 6},  // *RLA

    // 3
    Info {mode: Mode::Relative,    cycles: 2},  //  BMI
    Info {mode: Mode::IndirectY,   cycles: 5},  //  AND
    Info {mode: Mode::Implied,     cycles: 0},  // *JAM
    Info {mode: Mode::IndirectY,   cycles: 8},  // *RLA
    Info {mode: Mode::ZeroPageX,   cycles: 4},  // *IGN
    Info {mode: Mode::ZeroPageX,   cycles: 4},  //  AND
    Info {mode: Mode::ZeroPageX,   cycles: 6},  //  ROL
    Info {mode: Mode::ZeroPageX,   cycles: 6},  // *RLA
    Info {mode: Mode::Implied,     cycles: 2},  //  SEC
    Info {mode: Mode::AbsoluteY,   cycles: 4},  //  AND
    Info {mode: Mode::Implied,     cycles: 2},  // *NOP
    Info {mode: Mode::AbsoluteY,   cycles: 8},  // *RLA
    Info {mode: Mode::AbsoluteX,   cycles: 4},  // *IGN
    Info {mode: Mode::AbsoluteX,   cycles: 4},  //  AND
    Info {mode: Mode::AbsoluteX,   cycles: 7},  //  ROL
    Info {mode: Mode::AbsoluteX,   cycles: 7},  // *RLA

    // 4
    Info {mode: Mode::Implied,     cycles: 6},  //  RTI
    Info {mode: Mode::IndirectX,   cycles: 6},  //  EOR
    Info {mode: Mode::Implied,     cycles: 0},  // *JAM
    Info {mode: Mode::IndirectX,   cycles: 8},  // *SRE
    Info {mode: Mode::ZeroPage,    cycles: 3},  // *IGN
    Info {mode: Mode::ZeroPage,    cycles: 3},  //  EOR
    Info {mode: Mode::ZeroPage,    cycles: 5},  //  LSR
    Info {mode: Mode::ZeroPage,    cycles: 5},  // *SRE
    Info {mode: Mode::Implied,     cycles: 3},  //  PHA
    Info {mode: Mode::Immediate,   cycles: 2},  //  EOR
    Info {mode: Mode::Accumulator, cycles: 2},  //  LSR
    Info {mode: Mode::Immediate,   cycles: 2},  // *ALR
    Info {mode: Mode::Absolute,    cycles: 3},  //  JMP
    Info {mode: Mode::Absolute,    cycles: 4},  //  EOR
    Info {mode: Mode::Absolute,    cycles: 6},  //  LSR
    Info {mode: Mode::Absolute,    cycles: 6},  // *SRE

    // 5
    Info {mode: Mode::Relative,    cycles: 2},  //  BVC
    Info {mode: Mode::IndirectY,   cycles: 5},  //  EOR
    Info {mode: Mode::Implied,     cycles: 0},  // *JAM
    Info {mode: Mode::IndirectY,   cycles: 8},  // *SRE
    Info {mode: Mode::ZeroPageX,   cycles: 4},  // *IGN
    Info {mode: Mode::ZeroPageX,   cycles: 4},  //  EOR
    Info {mode: Mode::ZeroPageX,   cycles: 6},  //  LSR
    Info {mode: Mode::ZeroPageX,   cycles: 6},  // *SRE
    Info {mode: Mode::Implied,     cycles: 2},  //  CLI
    Info {mode: Mode::AbsoluteY,   cycles: 4},  //  EOR
    Info {mode: Mode::Implied,     cycles: 2},  // *NOP
    Info {mode: Mode::AbsoluteY,   cycles: 7},  // *SRE
    Info {mode: Mode::AbsoluteX,   cycles: 4},  // *IGN
    Info {mode: Mode::AbsoluteX,   cycles: 4},  //  EOR
    Info {mode: Mode::AbsoluteX,   cycles: 7},  //  LSR
    Info {mode: Mode::AbsoluteX,   cycles: 7},  // *SRE

    // 6
    Info {mode: Mode::Implied,     cycles: 6},  //  RTS
    Info {mode: Mode::IndirectX,   cycles: 6},  //  ADC
    Info {mode: Mode::Implied,     cycles: 0},  // *JAM
    Info {mode: Mode::IndirectX,   cycles: 8},  // *RRA
    Info {mode: Mode::ZeroPage,    cycles: 3},  // *IGN
    Info {mode: Mode::ZeroPage,    cycles: 4},  //  ADC
    Info {mode: Mode::ZeroPage,    cycles: 6},  //  ROR
    Info {mode: Mode::ZeroPage,    cycles: 5},  // *RRA
    Info {mode: Mode::Implied,     cycles: 4},  //  PLA
    Info {mode: Mode::Immediate,   cycles: 2},  //  ADC
    Info {mode: Mode::Accumulator, cycles: 2},  //  ROR
    Info {mode: Mode::Immediate,   cycles: 2},  // *ARR
    Info {mode: Mode::AbsoluteI,   cycles: 5},  //  JMP
    Info {mode: Mode::Absolute,    cycles: 4},  //  ADC
    Info {mode: Mode::Absolute,    cycles: 6},  //  ROR
    Info {mode: Mode::Absolute,    cycles: 6},  // *RRA

    // 7
    Info {mode: Mode::Relative,    cycles: 2},  //  BVS
    Info {mode: Mode::IndirectY,   cycles: 5},  //  ADC
    Info {mode: Mode::Implied,     cycles: 0},  // *JAM
    Info {mode: Mode::IndirectY,   cycles: 8},  // *RRA
    Info {mode: Mode::ZeroPageX,   cycles: 4},  // *IGN
    Info {mode: Mode::ZeroPageX,   cycles: 4},  //  ADC
    Info {mode: Mode::ZeroPageX,   cycles: 6},  //  ROR
    Info {mode: Mode::ZeroPageX,   cycles: 6},  // *RRA
    Info {mode: Mode::Implied,     cycles: 2},  //  SEI
    Info {mode: Mode::AbsoluteY,   cycles: 4},  //  ADC
    Info {mode: Mode::Implied,     cycles: 2},  // *NOP
    Info {mode: Mode::AbsoluteY,   cycles: 7},  // *RRA
    Info {mode: Mode::AbsoluteX,   cycles: 4},  // *IGN
    Info {mode: Mode::AbsoluteX,   cycles: 4},  //  ADC
    Info {mode: Mode::AbsoluteX,   cycles: 7},  //  ROR
    Info {mode: Mode::AbsoluteX,   cycles: 7},  // *RRA

    // 8
    Info {mode: Mode::Immediate,   cycles: 2},  // *SKB
    Info {mode: Mode::IndirectX,   cycles: 6},  //  STA
    Info {mode: Mode::Immediate,   cycles: 2},  // *SKB
    Info {mode: Mode::IndirectX,   cycles: 6},  // *SAX
    Info {mode: Mode::ZeroPage,    cycles: 3},  //  STY
    Info {mode: Mode::ZeroPage,    cycles: 3},  //  STA
    Info {mode: Mode::ZeroPage,    cycles: 3},  //  STX
    Info {mode: Mode::ZeroPage,    cycles: 3},  // *SAX
    Info {mode: Mode::Implied,     cycles: 2},  //  DEY
    Info {mode: Mode::Immediate,   cycles: 2},  // *SKB
    Info {mode: Mode::Implied,     cycles: 2},  //  TXA
    Info {mode: Mode::Immediate,   cycles: 2},  // *XAA
    Info {mode: Mode::Absolute,    cycles: 4},  //  STY
    Info {mode: Mode::Absolute,    cycles: 4},  //  STA
    Info {mode: Mode::Absolute,    cycles: 4},  //  STX
    Info {mode: Mode::Absolute,    cycles: 4},  // *SAX

    // 9
    Info {mode: Mode::Relative,    cycles: 2},  //  BCC
    Info {mode: Mode::IndirectY,   cycles: 6},  //  STA
    Info {mode: Mode::Implied,     cycles: 0},  // *JAM
    Info {mode: Mode::IndirectY,   cycles: 6},  // *SHA
    Info {mode: Mode::ZeroPageX,   cycles: 4},  //  STY
    Info {mode: Mode::ZeroPageX,   cycles: 4},  //  STA
    Info {mode: Mode::ZeroPageY,   cycles: 4},  //  STX
    Info {mode: Mode::ZeroPageY,   cycles: 4},  // *SAX
    Info {mode: Mode::Implied,     cycles: 2},  //  TYA
    Info {mode: Mode::AbsoluteY,   cycles: 5},  //  STA
    Info {mode: Mode::Implied,     cycles: 2},  //  TXS
    Info {mode: Mode::AbsoluteY,   cycles: 5},  // *SHS
    Info {mode: Mode::AbsoluteX,   cycles: 5},  // *SHY
    Info {mode: Mode::AbsoluteX,   cycles: 5},  //  STA
    Info {mode: Mode::AbsoluteY,   cycles: 5},  // *SHX
    Info {mode: Mode::AbsoluteY,   cycles: 5},  // *SHA

    // A
    Info {mode: Mode::Immediate,   cycles: 2},  //  LDY
    Info {mode: Mode::IndirectX,   cycles: 6},  //  LDA
    Info {mode: Mode::Immediate,   cycles: 2},  //  LDX
    Info {mode: Mode::IndirectX,   cycles: 6},  // *LAX
    Info {mode: Mode::ZeroPage,    cycles: 3},  //  LDY
    Info {mode: Mode::ZeroPage,    cycles: 3},  //  LDA
    Info {mode: Mode::ZeroPage,    cycles: 3},  //  LDX
    Info {mode: Mode::ZeroPage,    cycles: 3},  // *LAX
    Info {mode: Mode::Implied,     cycles: 2},  //  TAY
    Info {mode: Mode::Immediate,   cycles: 2},  //  LDA
    Info {mode: Mode::Implied,     cycles: 2},  //  TAX
    Info {mode: Mode::Immediate,   cycles: 2},  // *LAX
    Info {mode: Mode::Absolute,    cycles: 4},  //  LDY
    Info {mode: Mode::Absolute,    cycles: 4},  //  LDA
    Info {mode: Mode::Absolute,    cycles: 4},  //  LDX
    Info {mode: Mode::Absolute,    cycles: 4},  // *LAX

    // B
    Info {mode: Mode::Relative,    cycles: 2},  //  BCS
    Info {mode: Mode::IndirectY,   cycles: 5},  //  LDA
    Info {mode: Mode::Implied,     cycles: 0},  // *JAM
    Info {mode: Mode::IndirectY,   cycles: 4},  // *LAX
    Info {mode: Mode::ZeroPageX,   cycles: 4},  //  LDY
    Info {mode: Mode::ZeroPageX,   cycles: 4},  //  LDA
    Info {mode: Mode::ZeroPageY,   cycles: 4},  //  LDX
    Info {mode: Mode::ZeroPageY,   cycles: 4},  // *LAX
    Info {mode: Mode::Implied,     cycles: 2},  //  CLV
    Info {mode: Mode::AbsoluteY,   cycles: 4},  //  LDA
    Info {mode: Mode::Implied,     cycles: 2},  //  TSX
    Info {mode: Mode::AbsoluteY,   cycles: 4},  // *LAS
    Info {mode: Mode::AbsoluteX,   cycles: 4},  //  LDY
    Info {mode: Mode::AbsoluteX,   cycles: 4},  //  LDA
    Info {mode: Mode::AbsoluteY,   cycles: 4},  //  LDX
    Info {mode: Mode::AbsoluteY,   cycles: 4},  // *LAX

    // C
    Info {mode: Mode::Immediate,   cycles: 2},  //  CPY
    Info {mode: Mode::IndirectX,   cycles: 6},  //  CMP
    Info {mode: Mode::Immediate,   cycles: 2},  // *SKB
    Info {mode: Mode::IndirectX,   cycles: 8},  // *DCP
    Info {mode: Mode::ZeroPage,    cycles: 3},  //  CPY
    Info {mode: Mode::ZeroPage,    cycles: 3},  //  CMP
    Info {mode: Mode::ZeroPage,    cycles: 6},  //  DEC
    Info {mode: Mode::ZeroPage,    cycles: 5},  // *DCP
    Info {mode: Mode::Implied,     cycles: 2},  //  INY
    Info {mode: Mode::Immediate,   cycles: 2},  //  CMP
    Info {mode: Mode::Implied,     cycles: 2},  //  DEX
    Info {mode: Mode::Immediate,   cycles: 2},  // *AXS
    Info {mode: Mode::Absolute,    cycles: 4},  //  CPY
    Info {mode: Mode::Absolute,    cycles: 4},  //  CMP
    Info {mode: Mode::Absolute,    cycles: 6},  //  DEC
    Info {mode: Mode::Absolute,    cycles: 6},  // *DCP

    // D
    Info {mode: Mode::Relative,    cycles: 2},  //  BNE
    Info {mode: Mode::IndirectY,   cycles: 5},  //  CMP
    Info {mode: Mode::Implied,     cycles: 0},  // *JAM
    Info {mode: Mode::IndirectY,   cycles: 8},  // *DCP
    Info {mode: Mode::ZeroPageX,   cycles: 4},  // *IGN
    Info {mode: Mode::ZeroPageX,   cycles: 4},  //  CMP
    Info {mode: Mode::ZeroPageX,   cycles: 6},  //  DEC
    Info {mode: Mode::ZeroPageX,   cycles: 6},  // *DCP
    Info {mode: Mode::Implied,     cycles: 2},  //  CLD
    Info {mode: Mode::AbsoluteY,   cycles: 4},  //  CMP
    Info {mode: Mode::Implied,     cycles: 2},  // *NOP
    Info {mode: Mode::AbsoluteY,   cycles: 7},  // *DCP
    Info {mode: Mode::AbsoluteX,   cycles: 4},  // *IGN
    Info {mode: Mode::AbsoluteX,   cycles: 4},  //  CMP
    Info {mode: Mode::AbsoluteX,   cycles: 7},  //  DEC
    Info {mode: Mode::AbsoluteX,   cycles: 7},  // *DCP

    // E 
    Info {mode: Mode::Immediate,   cycles: 2},  //  CPX
    Info {mode: Mode::IndirectX,   cycles: 6},  //  SBC
    Info {mode: Mode::Immediate,   cycles: 2},  // *SKB
    Info {mode: Mode::IndirectX,   cycles: 8},  // *ISC
    Info {mode: Mode::ZeroPage,    cycles: 3},  //  CPX
    Info {mode: Mode::ZeroPage,    cycles: 3},  //  SBC
    Info {mode: Mode::ZeroPage,    cycles: 5},  //  INC
    Info {mode: Mode::ZeroPage,    cycles: 5},  // *ISC
    Info {mode: Mode::Implied,     cycles: 2},  //  INX
    Info {mode: Mode::Immediate,   cycles: 2},  //  SBC
    Info {mode: Mode::Implied,     cycles: 2},  //  NOP
    Info {mode: Mode::Immediate,   cycles: 2},  //  SBC
    Info {mode: Mode::Absolute,    cycles: 4},  //  CPX
    Info {mode: Mode::Absolute,    cycles: 4},  //  SBC
    Info {mode: Mode::Absolute,    cycles: 6},  //  INC
    Info {mode: Mode::Absolute,    cycles: 6},  // *ISC
    
    // F
    Info {mode: Mode::Relative,    cycles: 2},  //  BEQ
    Info {mode: Mode::IndirectY,   cycles: 5},  //  SBC
    Info {mode: Mode::Implied,     cycles: 0},  // *JAM
    Info {mode: Mode::IndirectY,   cycles: 8},  // *ISC
    Info {mode: Mode::ZeroPageX,   cycles: 4},  // *IGN
    Info {mode: Mode::ZeroPageX,   cycles: 4},  //  SBC
    Info {mode: Mode::ZeroPageX,   cycles: 6},  //  INC
    Info {mode: Mode::ZeroPageX,   cycles: 6},  // *ISC
    Info {mode: Mode::Implied,     cycles: 2},  //  SED
    Info {mode: Mode::AbsoluteY,   cycles: 4},  //  SBC
    Info {mode: Mode::Implied,     cycles: 2},  // *NOP
    Info {mode: Mode::AbsoluteY,   cycles: 8},  // *ISC
    Info {mode: Mode::AbsoluteY,   cycles: 7},  // *IGN
    Info {mode: Mode::AbsoluteX,   cycles: 4},  //  SBC
    Info {mode: Mode::AbsoluteX,   cycles: 7},  //  INC
    Info {mode: Mode::AbsoluteX,   cycles: 7},  // *ISC
];
