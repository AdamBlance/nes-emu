
use super::cartridge::{
    Cartridge,
    Mirroring,
    basic_nametable_mirroring,
    KB,
};

// iNES mapper 0: NROM-128 and NROM-256

pub struct CartridgeM0 {
    pub prg_rom: Vec<u8>,
    pub chr_rom: Vec<u8>,
    pub mirroring: Mirroring,
}

impl CartridgeM0 {
    pub fn new(prg_rom: Vec<u8>, chr_rom: Vec<u8>, mirroring: Mirroring) -> CartridgeM0 {
        CartridgeM0 {
            prg_rom,
            chr_rom,
            mirroring,
        }
    }
}

impl Cartridge for CartridgeM0 {
    // NROM doesn't support PRG RAM
    // NROM has no internal registers to write to
    // NROM-128 is 16KB mirrored twice, NROM-256 is 32KB
    fn read_prg_rom(&self, addr: u16) -> u8 {
        self.prg_rom[addr as usize % self.prg_rom.len()]
    }
    // CHR ROM is fixed 8KB
    fn read_chr(&mut self, addr: u16) -> u8 {
        self.chr_rom[addr as usize]
    }    
    fn get_physical_ntable_addr(&self, addr: u16) -> u16 {
        basic_nametable_mirroring(addr, self.mirroring)
    }
}
