use std::rc::Rc;
use dyn_clone::DynClone;
use serde::{Deserialize, Serialize};
use crate::emulator;
use crate::util::get_bit_u16;
use crate::nes::cartridge::{mapper0, mapper1, mapper2, mapper3, mapper4, mapper7};

#[derive(Copy, Clone, Serialize, Deserialize)]
pub enum Mirroring {
    Vertical,
    Horizontal,
    SingleScreenLower,
    SingleScreenUpper,
}

pub enum ChrMem {
    Rom(Rc<Vec<u8>>),
    Ram(Vec<u8>),
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

    fn get_physical_ntable_addr(&self, addr: u16) -> u16;

    fn asserting_irq(&mut self) -> bool {false}

    fn cpu_tick(&mut self) {}
    fn ppu_tick(&mut self, addr_bus: u16) {}
}

pub fn new_cartridge(rom_data: emulator::RomData) -> Box<dyn Cartridge> {
    match rom_data.mapper_id {
        0 => Box::new(mapper0::CartridgeM0::new(Rc::new(rom_data.prg_rom), Rc::new(rom_data.chr_rom), rom_data.mirroring_config)),
        1 => Box::new(mapper1::CartridgeM1::new(rom_data.prg_rom, rom_data.chr_rom, rom_data.chr_rom_is_ram)),
        2 => Box::new(mapper2::CartridgeM2::new(rom_data.prg_rom, rom_data.chr_rom, rom_data.chr_rom_is_ram, rom_data.mirroring_config)),
        3 => Box::new(mapper3::CartridgeM3::new(rom_data.prg_rom, rom_data.chr_rom, rom_data.mirroring_config)),
        4 => Box::new(mapper4::CartridgeM4::new(rom_data.prg_rom, rom_data.chr_rom)),
        7 => Box::new(mapper7::CartridgeM7::new(rom_data.prg_rom)),
        id => unimplemented!("Mapper {id} not implemented"),
    }
}

pub fn basic_nametable_mirroring(addr: u16, mirroring: Mirroring) -> u16 {
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
