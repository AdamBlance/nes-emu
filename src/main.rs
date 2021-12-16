#![feature(bigint_helper_methods)]
#![allow(unused_variables)]
#![allow(dead_code)]
#![allow(unused_parens)]

use std::fs;

mod opc;

struct Nes {
    cpu: Cpu,
    wram: [u8; 2048],
    cart: Cartridge,
}

struct Cpu {
    a: u8,
    x: u8,
    y: u8,
    s: u8,
    p: u8,
    pc: u16,
    cycles: u64,
}

enum PBit {
    N,  // Negative
    V,  // Overflow
    B,  // Break
    I,  // IRQ Disable
    Z,  // Zero
    C,  // Carry
}

struct Cartridge {
    prg_rom: Vec<u8>,
    chr_rom: Vec<u8>,
    mapper: u8,
}

fn main() {

    let ines_data = fs::read("rom.nes").expect("Failed to read rom");

    if ines_data.len() < 16 {
        panic!();
    }
    if &ines_data[0..4] != b"NES\x1A" {
        panic!();
    }
    
    let prg_start: usize = 16;
    let prg_end = prg_start + (ines_data[4] as usize) * 16384;
    let chr_end = prg_end + (ines_data[5] as usize) * 8192;

    let cart = Cartridge {
        prg_rom: ines_data[prg_start..prg_end].to_vec(),
        chr_rom: ines_data[prg_end..chr_end].to_vec(),
        mapper: (ines_data[7] & 0xF0) | (ines_data[6] >> 4),
    };


    emulate(&cart);

}

fn read_three_bytes(addr: u16, wram: &[u8], cart: &Cartridge) -> (u8, u8, u8) {
    (
        read_mem(addr, wram, cart),
        read_mem(addr.saturating_add(1), wram, cart),
        read_mem(addr.saturating_add(2), wram, cart)
    )
}

fn read_mem(addr: u16, wram: &[u8], cart: &Cartridge) -> u8 {
    match addr {
        0x0000..=0x1FFF => wram[(addr % 0x800) as usize],
        0x2000..=0x3FFF => wram[(0x2000 + (addr % 0x8)) as usize],
        0x4000..=0x401F => wram[addr as usize],
        0x4020..=0x5FFF => wram[addr as usize],
        0x6000..=0x7FFF => wram[addr as usize],
        0x8000..=0xBFFF => cart.prg_rom[addr as usize],
        0xC000..=0xFFFF => cart.prg_rom[(addr - 0x4000) as usize],
    }
}

fn write_mem(addr: u16, val: u8, wram: &mut [u8]) {
    if addr < 0x2000 {
        wram[(addr % 0x800) as usize] = val;
    }
}

fn read_p_bit(bit: PBit, cpu: &Cpu) -> bool {
    match bit {
        PBit::N => (cpu.p & 0b10000000) != 0,
        PBit::V => (cpu.p & 0b01000000) != 0,
        PBit::B => (cpu.p & 0b00010000) != 0,
        PBit::I => (cpu.p & 0b00000100) != 0,
        PBit::Z => (cpu.p & 0b00000010) != 0,
        PBit::C => (cpu.p & 0b00000001) != 0,
    }
}

fn write_c_bit(bit: PBit, val: bool, cpu: &mut Cpu) {
    let set = val as u8;
    match bit {
        PBit::N => cpu.p |= (set << 7),
        PBit::V => cpu.p |= (set << 6),
        PBit::B => cpu.p |= (set << 4),
        PBit::I => cpu.p |= (set << 2),
        PBit::Z => cpu.p |= (set << 1),
        PBit::C => cpu.p |= set,
    }
}

fn emulate(cart: &Cartridge) {

    let mut cpu = Cpu {
        a: 0,
        x: 0,
        y: 0,
        pc: ((cart.prg_rom[0xFFFD] as u16) << 4) + (cart.prg_rom[0xFFFC] as u16),
        s: 0,
        p: 0,
        cycles: 0, 
    };

    let mut wram: [u8; 2048] = [0; 2048];

    // Start main emulator loop

    loop {
        
        let (opcode, byte2, byte3) = read_three_bytes(cpu.pc, &wram, cart);
        
        match opcode {
            opc::ADC_ABS  => {
                // let (sum, carry) = read_mem()
            },
            opc::ADC_ABSX => {}, 
            opc::ADC_ABSY => {}, 
            opc::ADC_IMM  => {}, 
            opc::ADC_INDX => {}, 
            opc::ADC_INDY => {}, 
            opc::ADC_ZPG  => {}, 
            opc::ADC_ZPGX => {}, 
            
            opc::AND_ABS => {}, 
            opc::AND_ABSX => {}, 
            opc::AND_ABSY => {}, 
            opc::AND_IMM => {}, 
            opc::AND_INDX => {}, 
            opc::AND_INDY => {}, 
            opc::AND_ZPG => {}, 
            opc::AND_ZPGX => {}, 
            
            opc::ASL_ABS => {}, 
            opc::ASL_ABSX => {}, 
            opc::ASL_ACC => {}, 
            opc::ASL_ZPG => {}, 
            opc::ASL_ZPGX => {}, 
            
            opc::BCC => {}, 
            opc::BCS => {}, 
            opc::BEQ => {}, 
            opc::BMI => {}, 
            opc::BNE => {}, 
            opc::BPL => {}, 
            opc::BVC => {}, 
            opc::BVS => {}, 
            
            opc::BIT_ABS => {}, 
            opc::BIT_ZPG => {}, 
            
            opc::BRK => {}, 
            
            opc::CLC => {}, 
            opc::CLD => {}, 
            opc::CLI => {}, 
            opc::CLV => {}, 
            
            opc::CMP_ABS => {}, 
            opc::CMP_ABSX => {}, 
            opc::CMP_ABSY => {}, 
            opc::CMP_IMM => {}, 
            opc::CMP_INDX => {}, 
            opc::CMP_INDY => {}, 
            opc::CMP_ZPG => {}, 
            opc::CMP_ZPGX => {}, 
            
            opc::CPX_ABS => {}, 
            opc::CPX_IMM => {}, 
            opc::CPX_ZPG => {}, 
            
            opc::CPY_ABS => {}, 
            opc::CPY_IMM => {}, 
            opc::CPY_ZPG => {}, 
            
            opc::DEC_ABS => {}, 
            opc::DEC_ABSX => {}, 
            opc::DEC_ZPG => {}, 
            opc::DEC_ZPGX => {}, 
            
            opc::DEX => {}, 
            opc::DEY => {}, 
                        
            opc::EOR_ABS => {}, 
            opc::EOR_ABSX => {}, 
            opc::EOR_ABSY => {}, 
            opc::EOR_IMM => {}, 
            opc::EOR_INDX => {}, 
            opc::EOR_INDY => {}, 
            opc::EOR_ZPG => {}, 
            opc::EOR_ZPGX => {}, 
            
            opc::INC_ABS => {}, 
            opc::INC_ABSX => {}, 
            opc::INC_ZPG => {}, 
            opc::INC_ZPGX => {}, 
            
            opc::INX => {}, 
            opc::INY => {}, 
            
            opc::JMP_ABS => {}, 
            opc::JMP_IND => {}, 
            
            opc::JSR => {}, 
            
            opc::LDA_ABS => {}, 
            opc::LDA_ABSX => {}, 
            opc::LDA_ABSY => {}, 
            opc::LDA_IMM => {}, 
            opc::LDA_INDX => {}, 
            opc::LDA_INDY => {}, 
            opc::LDA_ZPG => {}, 
            opc::LDA_ZPGX => {}, 
            
            opc::LDX_ABS => {}, 
            opc::LDX_ABSY => {}, 
            opc::LDX_IMM => {}, 
            opc::LDX_ZPG => {}, 
            opc::LDX_ZPGY => {}, 
            
            opc::LDY_ABS => {}, 
            opc::LDY_ABSX => {}, 
            opc::LDY_IMM => {}, 
            opc::LDY_ZPG => {}, 
            opc::LDY_ZPGX => {}, 
            
            opc::LSR_ABS => {}, 
            opc::LSR_ABSX => {}, 
            opc::LSR_ACC => {}, 
            opc::LSR_ZPG => {}, 
            opc::LSR_ZPGX => {}, 
            
            opc::NOP => {}, 
            
            opc::ORA_ABS => {}, 
            opc::ORA_ABSX => {}, 
            opc::ORA_ABSY => {}, 
            opc::ORA_IMM => {}, 
            opc::ORA_INDX => {}, 
            opc::ORA_INDY => {}, 
            opc::ORA_ZPG => {}, 
            opc::ORA_ZPGX => {}, 
            
            opc::PHA => {}, 
            opc::PHP => {}, 
            opc::PLA => {}, 
            opc::PLP => {}, 
            
            opc::ROL_ABS => {}, 
            opc::ROL_ABSX => {}, 
            opc::ROL_ACC => {}, 
            opc::ROL_ZPG => {}, 
            opc::ROL_ZPGX => {}, 
            
            opc::ROR_ABS => {}, 
            opc::ROR_ABSX => {}, 
            opc::ROR_ACC => {}, 
            opc::ROR_ZPG => {}, 
            opc::ROR_ZPGX => {}, 

            opc::RTI => {}, 
            opc::RTS => {}, 
            
            opc::SBC_ABS => {}, 
            opc::SBC_ABSX => {}, 
            opc::SBC_ABSY => {}, 
            opc::SBC_IMM => {}, 
            opc::SBC_INDX => {}, 
            opc::SBC_INDY => {}, 
            opc::SBC_ZPG => {}, 
            opc::SBC_ZPGX => {}, 
            
            opc::SEC => {}, 
            opc::SED => {}, 
            opc::SEI => {}, 
            
            opc::STA_ABS => {}, 
            opc::STA_ABSX => {}, 
            opc::STA_ABSY => {}, 
            opc::STA_INDX => {}, 
            opc::STA_INDY => {}, 
            opc::STA_ZPG => {}, 
            opc::STA_ZPGX => {}, 
            
            opc::STX_ABS => {}, 
            opc::STX_ZPG => {}, 
            opc::STX_ZPGY => {}, 
            
            opc::STY_ABS => {}, 
            opc::STY_ZPG => {}, 
            opc::STY_ZPGX => {}, 
            
            opc::TAX => {}, 
            opc::TAY => {}, 
            opc::TSX => {}, 
            opc::TXA => {}, 
            opc::TXS => {}, 
            opc::TYA => {}, 
            _ => {},
        }

    }
    
}