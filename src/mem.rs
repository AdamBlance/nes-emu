
fn read_three_bytes(addr: u16, wram: &[u8], cart: &Cartridge) -> (u8, u8, u8) {
    (
        read_mem(addr, wram, cart),
        read_mem(addr.saturating_add(1), wram, cart),
        read_mem(addr.saturating_add(2), wram, cart)
    )
}

fn read_indexed_indirect(addr: u8, index: u8, wram: &[u8], cart: &Cartridge) -> u8 {
    let zp_addr_0: u8 = addr.wrapping_add(index);
    let zp_addr_1: u8 = zp_addr_0.wrapping_add(1);
    let lsb = wram[zp_addr_0];
    let msb = wram[zp_addr_1];

    read(concat_u8(msb, lsb), wram, cart)
}

fn read_indirect_indexed(

fn read(addr: u16, wram: &[u8], cart: &Cartridge) -> u8 {
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

fn write(addr: u16, val: u8, wram: &mut [u8]) {
    if addr < 0x2000 {
        wram[(addr % 0x800) as usize] = val;
    }
}