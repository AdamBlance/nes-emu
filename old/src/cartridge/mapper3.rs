
use super::cartridge::{
    Cartridge, 
    Mirroring, 
    basic_nametable_mirrroring,
    KB,
};

pub struct CartridgeM3 {
    // pub prg_ram: Vec<u8>,
    pub prg_rom: Vec<u8>,
    pub chr_rom: Vec<u8>,
    pub mirroring: Mirroring,

    pub bank_select: usize,
}
impl CartridgeM3 {
    pub fn new(prg_rom: Vec<u8>, chr_rom: Vec<u8>, mirroring: Mirroring) -> CartridgeM3 {
        CartridgeM3 {
            // prg_ram: [0; 0x2000].to_vec(),
            prg_rom,
            chr_rom,
            mirroring,
            bank_select: 0,
        }
    }
}
impl Cartridge for CartridgeM3 {
    // fn write_prg_ram(&mut self, addr: u16, byte: u8) {
    //     self.prg_ram[(addr - 0x6000) as usize] = byte;
    //     let string = String::from_utf8_lossy(&self.prg_ram[4..]);
    //     println!("{string}");
    // }
    fn read_prg_rom(&self, addr: u16) -> u8 {
        self.prg_rom[addr as usize % self.prg_rom.len()]
    }
    fn write_prg_rom(&mut self, _addr: u16, byte: u8) {
        // self.bank_select = (byte & 0b1111_1111) as usize;
        self.bank_select = (byte & 0b0000_0011) as usize;
    }

    fn read_chr(&mut self, addr: u16) -> u8 {
        self.chr_rom[self.bank_select * 0x2000 + addr as usize]
    }

    fn get_physical_ntable_addr(&self, addr: u16) -> u16 {
        basic_nametable_mirrroring(addr, self.mirroring)
    }
}