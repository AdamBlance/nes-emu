use dyn_clone::DynClone;
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Serialize, Deserialize)]
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
#[typetag::serde(tag = "type")]
pub trait Cartridge: DynClone {
    fn read_prg_ram(&mut self, addr: u16) -> u8 {0}
    fn write_prg_ram(&mut self, addr: u16, byte: u8) {}

    fn read_prg_rom(&self, addr: u16) -> u8;
    fn write_prg_rom(&mut self, addr: u16, byte: u8) {}

    fn read_chr(&mut self, addr: u16) -> u8;
    fn write_chr(&mut self, addr: u16, byte: u8) {}

    fn asserting_irq(&mut self) -> bool {false}

    fn cpu_tick(&mut self) {}
    fn ppu_tick(&mut self, addr_bus: u16) {}

    fn mirroring(&self) -> Mirroring;
}
