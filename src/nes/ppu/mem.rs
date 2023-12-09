use crate::nes::cartridge::Mirroring;
use crate::nes::Nes;

pub fn read_vram(addr: u16, nes: &mut Nes) -> u8 {
    // Colour palette reads don't actually put anything on the PPU address bus? I think?
    if addr < 0x3F00 {nes.ppu.addr_bus = addr;}
    match addr {
        0x0000..=0x1FFF => nes.cart.read_chr(addr),
        0x2000..=0x3EFF => nes.ppu.vram[vram_addr_to_nametables(addr, nes.cart.mirroring()) as usize],
        0x3F00..=0x3F1F => {
            let mut colour = nes.ppu.palette_mem[get_palette_mem_addr(addr)];
            if nes.ppu.greyscale {colour &= 0b0011_0000;}
            colour
        }
        _ => 0,
    }
}

pub fn write_vram(addr: u16, val: u8, nes: &mut Nes) {
    if addr < 0x3F00 {nes.ppu.addr_bus = addr;}
    match addr {
        0x0000..=0x1FFF => nes.cart.write_chr(addr, val),
        0x2000..=0x3EFF => nes.ppu.vram[vram_addr_to_nametables(addr, nes.cart.mirroring()) as usize] = val,
        0x3F00..=0x3F1F => nes.ppu.palette_mem[get_palette_mem_addr(addr)] = val,
        _ => (),
    }
}

pub fn increment_v_after_ppudata_access(nes: &mut Nes) {
    let increment = if nes.ppu.increment_select == false {1} else {32};
    nes.ppu.v = nes.ppu.v.wrapping_add(increment);
    nes.ppu.addr_bus = nes.ppu.v;
}

fn get_palette_mem_addr(addr: u16) -> usize {
    let offset = addr as usize - 0x3F00;
    if offset > 0xF && offset % 4 == 0 {
        offset - 0x10
    } else {
        offset
    }
}

pub fn vram_addr_to_nametables(addr: u16, mirroring: Mirroring) -> u16 {
    // The physical nametables sit at 0x2000..=0x23FF and 0x2400..=0x27FF
    let truncated = addr & 0b0000_1111_1111_1111;
    match mirroring {
        Mirroring::Vertical => truncated % 0x800,
        Mirroring::Horizontal => (truncated / 0x800) * 0x400 + (truncated % 0x400),
        Mirroring::SingleScreenLower => truncated % 0x400,
        Mirroring::SingleScreenUpper => 0x400 + (truncated % 0x400)
    }
}
