use dyn_clone::DynClone;
use serde::{Deserialize, Serialize};
use std::rc::Rc;

#[derive(Copy, Clone, Serialize, Deserialize)]
pub enum Mirroring {
    Vertical,
    Horizontal,
    SingleScreenLower,
    SingleScreenUpper,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum ChrMem {
    Rom(Rc<Vec<u8>>),
    Ram(Rc<Vec<u8>>),
}
impl ChrMem {
    pub fn new(rom_data: Option<Vec<u8>>) -> Self {
        match rom_data {
            Some(data) => Self::Rom(Rc::new(data)),
            None => Self::Ram(Rc::new(vec![0u8; 0x2000])),
        }
    }

    pub fn read(&self, addr: usize) -> u8 {
        match self {
            ChrMem::Rom(rom) => rom[addr],
            ChrMem::Ram(ram) => ram[addr],
        }
    }
    pub fn write(&mut self, addr: usize, value: u8) {
        if let ChrMem::Ram(ram) = self {
            Rc::make_mut(ram)[addr] = value
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct RomConfig {
    pub ines_mapper_id: u8,
    pub ines_mirroring: Mirroring,
    pub data: CartMemory,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct CartMemory {
    pub prg_ram: Option<Rc<Vec<u8>>>,
    pub prg_rom: Rc<Vec<u8>>,
    pub chr_mem: ChrMem,
}

impl CartMemory {
    pub fn new(prg_rom: Vec<u8>, chr_rom: Option<Vec<u8>>, has_prg_ram: bool) -> Self {
        CartMemory {
            prg_ram: match has_prg_ram {
                // TODO: Fix this
                /*
                   Mario 3 writes to and reads from 0x6000-0x7FFF where PRG RAM would be
                   but the cartridge doesn't actually have PRG RAM so any reads should just return
                   the open bus. There are no mentions of Mario 3 relying on open bus behaviour
                   to function but weirdly the game crashes at startup if don't enable PRG RAM.
                   Tried to figure out what was going wrong but to no avail. It's difficult without
                   a CPU instruction view, so maybe I should flesh that out first before trying
                   to fix this bug.
                */
                true | false => Some(Rc::new(vec![0u8; 0x2000])),
                // true => Some(Rc::new(vec![0u8; 0x2000])),
                // false => None,
            },
            prg_rom: Rc::new(prg_rom),
            chr_mem: ChrMem::new(chr_rom),
        }
    }
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
    fn read_prg_ram(&mut self, _addr: u16) -> Option<u8> {
        None
    }
    fn write_prg_ram(&mut self, _addr: u16, _byte: u8) {}

    fn read_prg_rom(&self, addr: u16) -> u8;
    fn write_prg_rom(&mut self, _addr: u16, _byte: u8) {}

    fn read_chr(&mut self, addr: u16) -> u8;
    fn write_chr(&mut self, _addr: u16, _byte: u8) {}

    fn asserting_irq(&mut self) -> bool {
        false
    }

    fn cpu_tick(&mut self) {}
    fn ppu_tick(&mut self, _addr_bus: u16) {}

    fn mirroring(&self) -> Mirroring;
}
