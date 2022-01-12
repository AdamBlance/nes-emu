use crate::util::*;
use crate::hw::*;

pub fn read_mem_u16(addr: u16, nes: &Nes) -> u16 {
    let lsb = read_mem(addr, nes);
    let msb = read_mem(addr.wrapping_add(1), nes);
    concat_u8(msb, lsb)
}

pub fn read_mem(addr: u16, nes: &Nes) -> u8 {
    match addr {
        0x0000..=0x1FFF => nes.wram[(addr % 0x800) as usize],
        0x2000..=0x3FFF => 0,
        0x4000..=0x401F => 0,
        0x4020..=0x5FFF => 0,
        0x6000..=0x7FFF => 0,
        0x8000..=0xBFFF => nes.cart.prg_rom[(addr - 0x8000) as usize],
        0xC000..=0xFFFF => nes.cart.prg_rom[(addr - 0xC000) as usize],
    }
}

pub fn write_mem(addr: u16, val: u8, nes: &mut Nes) {
    if addr < 0x2000 {
        nes.wram[(addr % 0x800) as usize] = val;
    }
}

