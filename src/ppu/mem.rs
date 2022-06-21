use crate::nes::Nes;

pub fn read_vram(addr: u16, nes: &mut Nes) -> u8 {
    match addr {
        0x0000..=0x1FFF => nes.cart.read_chr(addr),
        0x2000..=0x2FFF => nes.ppu.vram[nes.cart.get_physical_ntable_addr(addr) as usize],
        0x3000..=0x3EFF => nes.ppu.vram[nes.cart.get_physical_ntable_addr(addr - 0x1000) as usize],
        0x3F00..=0x3F1F => {
            let mut colour = nes.ppu.palette_mem[get_palette_mem_addr(addr)];
            if nes.ppu.greyscale {colour &= 0b0011_0000;}
            colour
        }
        _ => 0,
    }
}

pub fn write_vram(addr: u16, val: u8, nes: &mut Nes) {
    match addr {
        0x0000..=0x1FFF => nes.cart.write_chr(addr, val),
        0x2000..=0x2FFF => nes.ppu.vram[nes.cart.get_physical_ntable_addr(addr) as usize] = val,
        0x3000..=0x3EFF => nes.ppu.vram[nes.cart.get_physical_ntable_addr(addr - 0x1000) as usize] = val,
        0x3F00..=0x3F1F => nes.ppu.palette_mem[get_palette_mem_addr(addr)] = val,
        _ => (),
    }
}

pub fn increment_v_after_ppudata_access(nes: &mut Nes) {
    let increment = if nes.ppu.increment_select == false {1} else {32};
    nes.ppu.v = nes.ppu.v.wrapping_add(increment);
}

fn get_palette_mem_addr(addr: u16) -> usize {
    let offset = addr as usize - 0x3F00;
    if offset > 0xF && offset % 4 == 0 {
        offset - 0x10
    } else {
        offset
    }
}