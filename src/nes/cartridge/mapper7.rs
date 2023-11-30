
use super::cartridge::{
    Cartridge,
    Mirroring,
    basic_nametable_mirroring,
    KB,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct CartridgeM7 {
    pub prg_rom: Vec<u8>,
    pub chr_ram: Vec<u8>,
    pub mirroring: Mirroring,

    pub bank_select: usize,
}
impl CartridgeM7 {
    pub fn new(prg_rom: Vec<u8>) -> CartridgeM7 {
        CartridgeM7 {
            prg_rom,
            chr_ram: [0; 0x2000].to_vec(),
            mirroring: Mirroring::SingleScreenLower,
            bank_select: 0,
        }
    }
}

#[typetag::serde]
impl Cartridge for CartridgeM7 {
    fn read_prg_rom(&self, addr: u16) -> u8 {
        self.prg_rom[self.bank_select * 0x8000 + (addr as usize - 0x8000)]
    }
    fn write_prg_rom(&mut self, _addr: u16, byte: u8) {
        self.bank_select = (byte & 0b0000_0111) as usize;
        self.mirroring = if (byte & 0b0001_0000) == 0 {
            Mirroring::SingleScreenLower
        } else {
            Mirroring::SingleScreenUpper
        };
    }

    fn read_chr(&mut self, addr: u16) -> u8 {
        self.chr_ram[addr as usize]
    }

    fn write_chr(&mut self, addr: u16, byte: u8) {
        self.chr_ram[addr as usize] = byte;
    }

    fn get_physical_ntable_addr(&self, addr: u16) -> u16 {
        basic_nametable_mirroring(addr, self.mirroring)
    }
}