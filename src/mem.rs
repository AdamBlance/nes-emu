use crate::util::*;
use crate::hw::*;

pub fn read_mem_u16(addr: u16, nes: &mut Nes) -> u16 {
    let lsb = read_mem(addr, nes);
    let msb = read_mem(addr.wrapping_add(1), nes);
    concat_u8(msb, lsb)
}

pub fn read_mem_u16_zp(addr: u16, nes: &mut Nes) -> u16 {
    let lsb = read_mem(addr, nes);
    let msb = read_mem((addr.wrapping_add(1) % 256), nes);
    concat_u8(msb, lsb)
}

pub fn read_mem(addr: u16, nes: &mut Nes) -> u8 {
    match addr {
        0x0000..=0x1FFF => nes.wram[(addr % 0x800) as usize],
        0x2000..=0x2001 => 0,
        0x2002          => {
            nes.ppu.w = false;
            ppustatus_to_byte(nes)
        }
        0x2003..=0x2007 => 0,
        0x2008..=0x3FFF => 0,
        0x4000..=0x401F => 0,
        0x4020..=0x5FFF => 0,
        0x6000..=0x7FFF => 0,
        0x8000..=0xBFFF => nes.cart.prg_rom[(addr - 0x8000) as usize],
        0xC000..=0xFFFF => nes.cart.prg_rom[(addr - 0x8000) as usize],
    }
}

// Needs to set ppu_written_to when affecting registers
pub fn write_mem(addr: u16, val: u8, nes: &mut Nes) {
    let val_u16 = val as u16;
    match addr {
        0x0000..=0x1FFF => nes.wram[(addr % 0x800) as usize] = val,
        0x2000          => byte_to_ppuctrl(val, nes),
        0x2001          => byte_to_ppumask(val, nes),
        0x2005          => {
            if !nes.ppu.w {
                nes.ppu.t &= 0b1_111_11_11111_00000;
                nes.ppu.t |= (val as u16) >> 3;
                nes.ppu.x = val & 0b0000_0111;
            } else {
                nes.ppu.t &= 0b1_000_11_00000_11111;
                nes.ppu.t |= (val_u16 & 0b11111_000) << 2;
                nes.ppu.t |= (val_u16 & 0b00000_111) << 12;
            }
            nes.ppu.w = !nes.ppu.w;
        }
        0x2006          => {
            if !nes.ppu.w {
                nes.ppu.t &= 0b1_0_00000_11111111;
                nes.ppu.t |= (val_u16 & 0b00011111) << 8;
            } else {
                nes.ppu.t &= 0b1_1_11111_00000000;
                nes.ppu.t |= val_u16;
                nes.ppu.v = nes.ppu.t;
            }
            nes.ppu.w = !nes.ppu.w;
        }
        _ => (),
    }
}

fn byte_to_ppuctrl(byte: u8, nes: &mut Nes) {
    nes.ppu.nmi_enable = get_bit(byte, 7);
    nes.ppu.master_slave = get_bit(byte, 6);
    nes.ppu.sprite_height = get_bit(byte, 5);
    nes.ppu.background_tile_select = get_bit(byte, 4);
    nes.ppu.sprite_tile_select = get_bit(byte, 3);
    nes.ppu.increment_mode = get_bit(byte, 2);
    nes.ppu.nametable_select = byte & 0b0000_0011;
}
fn byte_to_ppumask(byte: u8, nes: &mut Nes) {
    nes.ppu.blue_emphasis = get_bit(byte, 7);
    nes.ppu.green_emphasis = get_bit(byte, 6);
    nes.ppu.red_emphasis = get_bit(byte, 5);
    nes.ppu.sprite_enable = get_bit(byte, 4);
    nes.ppu.background_enable = get_bit(byte, 3);
    nes.ppu.sprite_left_column_enable = get_bit(byte, 2);
    nes.ppu.background_left_column_enable = get_bit(byte, 1);
    nes.ppu.greyscale = get_bit(byte, 0);
}
fn ppustatus_to_byte(nes: &Nes) -> u8 {
    (if nes.ppu.vblank          {0b1000_0000} else {0}) | 
    (if nes.ppu.sprite_zero_hit {0b0100_0000} else {0}) | 
    (if nes.ppu.sprite_overflow {0b0000_1000} else {0})
}
fn byte_to_ppustatus(byte: u8, nes: &mut Nes) {
    nes.ppu.vblank = get_bit(byte, 7);
    nes.ppu.sprite_zero_hit = get_bit(byte, 6);
    nes.ppu.sprite_overflow = get_bit(byte, 5);
}
