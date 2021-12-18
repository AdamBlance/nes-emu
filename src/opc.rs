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
   AbsoluteIndirect, 
}

fn get_addr_mode(opcode: u8) -> Mode {
    
    // Match outliers
    match opcode {
        0x20 => Mode::Absolute,
        0x6C => Mode::AbsoluteIndirect,
    }

    // Determine addressing mode from opcode bits
    // https://wiki.nesdev.org/w/index.php/CPU_unofficial_opcodes
    let grp_lsb = (opcode & 0b00000001);
    let grp_msb = (opcode & 0b00000010) >> 1;
    let col     = (opcode & 0b00011100) >> 2;
    let row = (opcode & 0b11100000) >> 5;

    match (grp_msb, grp_lsb, col, row) {
        (_,1,2,_) | (_,0,0,4..=7) => Mode::Immediate,
        (_,1,6,_) | (1,_,7,4..=5) => Mode::AbsoluteY,
        (1,_,5,4..=5)             => Mode::ZeroPageY,
        (_,_,3,_)                 => Mode::Absolute,
        (_,_,7,_)                 => Mode::AbsoluteX,
        (_,_,1,_)                 => Mode::ZeroPage,
        (_,_,5,_)                 => Mode::ZeroPageX
        (_,1,0,_)                 => Mode::IndirectX,
        (_,1,4,_)                 => Mode::IndirectY,
        (0,0,4,_)                 => Mode::Relative,
        _                         => Mode::Implied,
    }
}

pub const ADC_ABS:  u8 = 0x6D;
pub const ADC_ABSX: u8 = 0x7D;
pub const ADC_ABSY: u8 = 0x79;
pub const ADC_IMM:  u8 = 0x69;
pub const ADC_INDX: u8 = 0x61;
pub const ADC_INDY: u8 = 0x71;
pub const ADC_ZPG:  u8 = 0x65;
pub const ADC_ZPGX: u8 = 0x75;

pub const AND_ABS:  u8 = 0x2D;
pub const AND_ABSX: u8 = 0x3D;
pub const AND_ABSY: u8 = 0x39;
pub const AND_IMM:  u8 = 0x29;
pub const AND_INDX: u8 = 0x21;
pub const AND_INDY: u8 = 0x31;
pub const AND_ZPG:  u8 = 0x25;
pub const AND_ZPGX: u8 = 0x35;

pub const ASL_ABS:  u8 = 0x0E;
pub const ASL_ABSX: u8 = 0x1E;
pub const ASL_ACC:  u8 = 0x0A;
pub const ASL_ZPG:  u8 = 0x06;
pub const ASL_ZPGX: u8 = 0x16;

pub const BCC: u8 = 0x90;  
pub const BCS: u8 = 0xB0;  
pub const BEQ: u8 = 0xF0;
pub const BMI: u8 = 0x30;
pub const BNE: u8 = 0xD0;
pub const BPL: u8 = 0x10;
pub const BVC: u8 = 0x50;
pub const BVS: u8 = 0x70;

pub const BIT_ABS: u8 = 0x2C;
pub const BIT_ZPG: u8 = 0x24;

pub const BRK: u8 = 0x00;

pub const CLC: u8 = 0x18;
pub const CLD: u8 = 0xD8;
pub const CLI: u8 = 0x58;
pub const CLV: u8 = 0xB8;

pub const CMP_ABS:  u8 = 0xCD;
pub const CMP_ABSX: u8 = 0xDD;
pub const CMP_ABSY: u8 = 0xD9;
pub const CMP_IMM:  u8 = 0xC9;
pub const CMP_INDX: u8 = 0xC1;
pub const CMP_INDY: u8 = 0xD1;
pub const CMP_ZPG:  u8 = 0xC5;
pub const CMP_ZPGX: u8 = 0xD5;

pub const CPX_ABS: u8 = 0xEC;
pub const CPX_IMM: u8 = 0xE0;
pub const CPX_ZPG: u8 = 0xE4;

pub const CPY_ABS: u8 = 0xCC;
pub const CPY_IMM: u8 = 0xC0;
pub const CPY_ZPG: u8 = 0xC4;

pub const DEC_ABS:  u8 = 0xCE;
pub const DEC_ABSX: u8 = 0xDE;
pub const DEC_ZPG:  u8 = 0xC6;
pub const DEC_ZPGX: u8 = 0xD6;

pub const DEX: u8 = 0xCA;
pub const DEY: u8 = 0x88;

pub const EOR_ABS:  u8 = 0x4D;
pub const EOR_ABSX: u8 = 0x5D;
pub const EOR_ABSY: u8 = 0x59;
pub const EOR_IMM:  u8 = 0x49;
pub const EOR_INDX: u8 = 0x41;
pub const EOR_INDY: u8 = 0x51;
pub const EOR_ZPG:  u8 = 0x45;
pub const EOR_ZPGX: u8 = 0x55;

pub const INC_ABS:  u8 = 0xEE;
pub const INC_ABSX: u8 = 0xFE;
pub const INC_ZPG:  u8 = 0xE6;
pub const INC_ZPGX: u8 = 0xF6;

pub const INX: u8 = 0xE8;
pub const INY: u8 = 0xC8;

pub const JMP_ABS: u8 = 0x4C;
pub const JMP_IND: u8 = 0x6C;

pub const JSR: u8 = 0x20;

pub const LDA_ABS:  u8 = 0xAD;
pub const LDA_ABSX: u8 = 0xBD;
pub const LDA_ABSY: u8 = 0xB9;
pub const LDA_IMM:  u8 = 0xA9;
pub const LDA_INDX: u8 = 0xA1;
pub const LDA_INDY: u8 = 0xB1;
pub const LDA_ZPG:  u8 = 0xA5;
pub const LDA_ZPGX: u8 = 0xB5;

pub const LDX_ABS:  u8 = 0xAE;
pub const LDX_ABSY: u8 = 0xBE;
pub const LDX_IMM:  u8 = 0xA2;
pub const LDX_ZPG:  u8 = 0xA6;
pub const LDX_ZPGY: u8 = 0xB6;

pub const LDY_ABS:  u8 = 0xAC;
pub const LDY_ABSX: u8 = 0xBC;
pub const LDY_IMM:  u8 = 0xA0;
pub const LDY_ZPG:  u8 = 0xA4;
pub const LDY_ZPGX: u8 = 0xB4;

pub const LSR_ABS:  u8 = 0x4E;
pub const LSR_ABSX: u8 = 0x5E;
pub const LSR_ACC:  u8 = 0x4A;
pub const LSR_ZPG:  u8 = 0x46;
pub const LSR_ZPGX: u8 = 0x56;

pub const NOP: u8 = 0xEA;

pub const ORA_ABS:  u8 = 0x0D;
pub const ORA_ABSX: u8 = 0x1D;
pub const ORA_ABSY: u8 = 0x19;
pub const ORA_IMM:  u8 = 0x09;
pub const ORA_INDX: u8 = 0x01;
pub const ORA_INDY: u8 = 0x11;
pub const ORA_ZPG:  u8 = 0x05;
pub const ORA_ZPGX: u8 = 0x15;

pub const PHA: u8 = 0x48;
pub const PHP: u8 = 0x08;
pub const PLA: u8 = 0x68;
pub const PLP: u8 = 0x28;

pub const ROL_ABS:  u8 = 0x2E;
pub const ROL_ABSX: u8 = 0x3E;
pub const ROL_ACC:  u8 = 0x2A;
pub const ROL_ZPG:  u8 = 0x26;
pub const ROL_ZPGX: u8 = 0x36;

pub const ROR_ABS:  u8 = 0x6E;
pub const ROR_ABSX: u8 = 0x7E;
pub const ROR_ACC:  u8 = 0x6A;
pub const ROR_ZPG:  u8 = 0x66;
pub const ROR_ZPGX: u8 = 0x76;

pub const RTI: u8 = 0x40;
pub const RTS: u8 = 0x60;

pub const SBC_ABS:  u8 = 0xED;
pub const SBC_ABSX: u8 = 0xFD;
pub const SBC_ABSY: u8 = 0xF9;
pub const SBC_IMM:  u8 = 0xE9;
pub const SBC_INDX: u8 = 0xE1;
pub const SBC_INDY: u8 = 0xF1;
pub const SBC_ZPG:  u8 = 0xE5;
pub const SBC_ZPGX: u8 = 0xF5;

pub const SEC: u8 = 0x38;
pub const SED: u8 = 0xF8;
pub const SEI: u8 = 0x78;

pub const STA_ABS:  u8 = 0x8D;
pub const STA_ABSX: u8 = 0x9D;
pub const STA_ABSY: u8 = 0x99;
pub const STA_INDX: u8 = 0x81;
pub const STA_INDY: u8 = 0x91;
pub const STA_ZPG:  u8 = 0x85;
pub const STA_ZPGX: u8 = 0x95;

pub const STX_ABS:  u8 = 0x8E;
pub const STX_ZPG:  u8 = 0x86;
pub const STX_ZPGY: u8 = 0x96;

pub const STY_ABS:  u8 = 0x8C;
pub const STY_ZPG:  u8 = 0x84;
pub const STY_ZPGX: u8 = 0x94;

pub const TAX: u8 = 0xAA;
pub const TAY: u8 = 0xA8;
pub const TSX: u8 = 0xBA;
pub const TXA: u8 = 0x8A;
pub const TXS: u8 = 0x9A;
pub const TYA: u8 = 0x98;