use crate::nes::Nes;

pub fn read_vram(addr: u16, nes: &mut Nes) -> u8 {
    let a = addr as usize;
    match addr {
        0x0000..=0x1FFF => nes.cartridge.read_chr(addr),
        0x2000..=0x2FFF => nes.ppu.vram[nes.cartridge.get_physical_ntable_addr(addr) as usize],
        0x3000..=0x3EFF => nes.ppu.vram[nes.cartridge.get_physical_ntable_addr(addr - 0x1000) as usize],
        0x3F00..=0x3F0F => nes.ppu.palette_mem[a - 0x3F00],
        0x3F10..=0x3F1F => {
            let temp = a - 0x3F00;
            if temp % 4 == 0 {
                nes.ppu.palette_mem[temp - 0x10]  // need to extract this stuff to avoid duplication
            } else {
                nes.ppu.palette_mem[temp]
            }
        }
        _ => 0,
    }
}

pub fn write_vram(addr: u16, val: u8, nes: &mut Nes) {
    let a = addr as usize;
    match addr {
        0x0000..=0x1FFF => nes.cartridge.write_chr(addr, val),
        0x2000..=0x2FFF => nes.ppu.vram[nes.cartridge.get_physical_ntable_addr(addr) as usize] = val,
        0x3000..=0x3EFF => nes.ppu.vram[nes.cartridge.get_physical_ntable_addr(addr - 0x1000) as usize] = val,
        0x3F00..=0x3F0F => nes.ppu.palette_mem[a - 0x3F00] = val,
        0x3F10..=0x3F1F => {
            let temp = a - 0x3F00;
            if temp % 4 == 0 {
                nes.ppu.palette_mem[temp - 0x10] = val;
            } else {
                nes.ppu.palette_mem[temp] = val;
            }
        }
        _ => (),
    }
}

pub fn increment_v_after_ppudata_access(nes: &mut Nes) {
    let increment = if nes.ppu.increment_select == false {1} else {32};
    nes.ppu.v = nes.ppu.v.wrapping_add(increment);
}