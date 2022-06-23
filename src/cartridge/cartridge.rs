use crate::util::get_bit_u16;


#[derive(Copy, Clone)]
pub enum Mirroring {
    Vertical,
    Horizontal,
    SingleScreenLower,
    SingleScreenUpper,
}

pub const KB: usize = 0x400;

/*

    Passing cpu_cycle to keep track of system state won't be enough for all mappers. 
    MMC3 has a scanline counter that advances when it sees bit 12 of the PPU address bus go from 
    low to high. This has to be updated the moment this happens, not just at the next cartridge 
    access. This is because the mapper needs to be able to send an interrupt request to the CPU 
    regardless of when the last CHR read happened.
    I'll need a method for updating state every PPU cycle and every CPU cycle. 

*/

// All cartridges must implement this
pub trait Cartridge {
    fn read_prg_ram(&mut self, _addr: u16) -> u8 {0}
    fn write_prg_ram(&mut self, _addr: u16, _byte: u8) {}

    fn read_prg_rom(&mut self, addr: u16) -> u8;
    fn write_prg_rom(&mut self, _addr: u16, _byte: u8, _cpu_cycle: u64) {}

    fn read_chr(&mut self, addr: u16) -> u8;
    fn write_chr(&mut self, _addr: u16, _byte: u8) {}

    fn get_physical_ntable_addr(&self, addr: u16) -> u16;

    fn asserting_irq(&mut self) -> bool {false}

    fn cpu_tick(&mut self) {}
    fn ppu_tick(&mut self) {}
}


pub fn basic_nametable_mirrroring(addr: u16, mirroring: Mirroring) -> u16 {
    // The physical nametables sit at 0x2000..=0x23FF and 0x2400..=0x27FF
    let vram_addr = match mirroring {
        Mirroring::Vertical => match addr {
            0x2000..=0x23FF => addr,
            0x2400..=0x27FF => addr,
            0x2800..=0x2BFF => addr - 0x800,
            0x2C00..=0x2FFF => addr - 0x800,
            _ => unreachable!(),
        }
        Mirroring::Horizontal => match addr {
            0x2000..=0x23FF => addr,
            0x2400..=0x27FF => addr - 0x400,
            0x2800..=0x2BFF => addr - 0x400,
            0x2C00..=0x2FFF => addr - 0x800,
            _ => unreachable!(),
        }
        Mirroring::SingleScreenLower => match addr {
            0x2000..=0x23FF => addr,
            0x2400..=0x27FF => addr - 0x400,
            0x2800..=0x2BFF => addr - 0x800,
            0x2C00..=0x2FFF => addr - 0xC00,
            _ => unreachable!(),
        }
        Mirroring::SingleScreenUpper => match addr {
            0x2000..=0x23FF => addr + 0x400,
            0x2400..=0x27FF => addr,
            0x2800..=0x2BFF => addr - 0x400,
            0x2C00..=0x2FFF => addr - 0x800,
            _ => unreachable!(),
        }
    };
    vram_addr - 0x2000
}
