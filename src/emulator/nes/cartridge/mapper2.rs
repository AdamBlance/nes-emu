use super::cartridge_def::{CartMemory, Cartridge, Mirroring, RomConfig};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct CartridgeM2 {
    rom_data: CartMemory,
    bank_select: usize,
    mirroring: Mirroring,
}
impl CartridgeM2 {
    pub fn new(rom_config: RomConfig) -> CartridgeM2 {
        CartridgeM2 {
            rom_data: rom_config.data,
            mirroring: rom_config.ines_mirroring,

            bank_select: 0,
        }
    }
}
#[typetag::serde]
impl Cartridge for CartridgeM2 {
    // UxROM doesn't have PRG RAM support

    fn read_prg_rom(&self, addr: u16) -> u8 {
        match addr {
            // Swappable 16KB at start of cartridge range
            0x8000..=0xBFFF => {
                self.rom_data.prg_rom[(self.bank_select * 0x4000) + (addr as usize - 0x8000)]
            }
            // Fixed 16KB at end of addressable range
            0xC000..=0xFFFF => {
                self.rom_data.prg_rom
                    [(self.rom_data.prg_rom.len() - 0x4000) + (addr as usize - 0xC000)]
            }
            _ => unreachable!(),
        }
    }
    fn write_prg_rom(&mut self, _addr: u16, byte: u8) {
        self.bank_select = byte as usize;
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
