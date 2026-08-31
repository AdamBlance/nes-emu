use super::cartridge_def::{CartMemory, Cartridge, Mirroring, RomConfig};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct CartridgeM3 {
    rom_data: CartMemory,
    mirroring: Mirroring,
    bank_select: usize,
}
impl CartridgeM3 {
    pub fn new(rom_config: RomConfig) -> CartridgeM3 {
        CartridgeM3 {
            rom_data: rom_config.data,
            mirroring: rom_config.ines_mirroring,
            bank_select: 0,
        }
    }
}

#[typetag::serde]
impl Cartridge for CartridgeM3 {
    fn read_prg_rom(&self, addr: u16) -> u8 {
        self.rom_data.prg_rom[addr as usize % self.rom_data.prg_rom.len()]
    }
    fn write_prg_rom(&mut self, _addr: u16, byte: u8) {
        self.bank_select = (byte & 0b0000_0011) as usize;
    }
    fn read_chr(&mut self, addr: u16) -> u8 {
        self.rom_data
            .chr_mem
            .read(self.bank_select * 0x2000 + addr as usize)
    }
    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }
}
