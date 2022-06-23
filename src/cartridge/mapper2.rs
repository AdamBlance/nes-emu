
use super::cartridge::{
    Cartridge, 
    Mirroring, 
    basic_nametable_mirrroring,
    KB,
};

pub struct CartridgeM2 {
    pub prg_rom: Vec<u8>,
    pub chr_rom: Vec<u8>,

    pub chr_rom_is_ram: bool,
    pub bank_select: usize,

    pub mirroring: Mirroring,
}
impl CartridgeM2 {
    pub fn new(prg_rom: Vec<u8>, chr_rom: Vec<u8>, chr_rom_is_ram: bool, mirroring: Mirroring) -> CartridgeM2 {
        CartridgeM2 {
            prg_rom,
            chr_rom,
            chr_rom_is_ram,
            mirroring,

            bank_select: 0,
        }
    }
}
impl Cartridge for CartridgeM2 {
    // UxROM doesn't have PRG RAM support

    fn read_prg_rom(&mut self, addr: u16) -> u8 {
        match addr {
            // Swappable 16KB at start of cartridge range
            0x8000..=0xBFFF => self.prg_rom[(self.bank_select * 0x4000) + (addr as usize - 0x8000)],
            // Fixed 16KB at end of addressable range
            0xC000..=0xFFFF => self.prg_rom[(self.prg_rom.len() - 0x4000) + (addr as usize - 0xC000)],
            _ => unreachable!(),
        }
    }
    fn write_prg_rom(&mut self, _addr: u16, byte: u8, _cpu_cycle: u64) {
        self.bank_select = byte as usize;   
    }

    fn read_chr(&mut self, addr: u16) -> u8 {
        self.chr_rom[addr as usize]
    }
    fn write_chr(&mut self, addr: u16, byte: u8) {
        if self.chr_rom_is_ram {
            self.chr_rom[addr as usize] = byte;
        }
    }
    fn get_physical_ntable_addr(&self, addr: u16) -> u16 {
        basic_nametable_mirrroring(addr, self.mirroring)
    }

}