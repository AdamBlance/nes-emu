use super::cartridge_def::{CartMemory, Cartridge, Mirroring, RomConfig};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct CartridgeM7 {
    rom_data: CartMemory,
    mirroring: Mirroring,
    bank_select: usize,
}
impl CartridgeM7 {
    pub fn new(rom_config: RomConfig) -> CartridgeM7 {
        CartridgeM7 {
            rom_data: rom_config.data,
            mirroring: Mirroring::SingleScreenLower,
            bank_select: 0,
        }
    }
}

#[typetag::serde]
impl Cartridge for CartridgeM7 {
    fn read_prg_rom(&self, addr: u16) -> u8 {
        self.rom_data.prg_rom[self.bank_select * 0x8000 + (addr as usize - 0x8000)]
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
        self.rom_data.chr_mem.read(addr as usize)
    }
    fn write_chr(&mut self, addr: u16, value: u8) {
        self.rom_data.chr_mem.write(addr as usize, value);
    }
    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }
}
