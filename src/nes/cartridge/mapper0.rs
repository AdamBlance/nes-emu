use super::cartridge_def::{CartMemory, Cartridge, Mirroring, RomConfig};
use serde::{Deserialize, Serialize};

// iNES mapper 0: NROM-128 and NROM-256

#[derive(Clone, Serialize, Deserialize)]
pub struct CartridgeM0 {
    rom_data: CartMemory,
    mirroring: Mirroring,
}

impl CartridgeM0 {
    pub fn new(rom_config: RomConfig) -> CartridgeM0 {
        CartridgeM0 {
            rom_data: rom_config.data,
            mirroring: rom_config.ines_mirroring,
        }
    }
}
#[typetag::serde]
impl Cartridge for CartridgeM0 {
    // NROM doesn't support PRG RAM
    // NROM has no internal registers to write to
    // NROM-128 is 16KB mirrored twice, NROM-256 is 32KB
    fn read_prg_rom(&self, addr: u16) -> u8 {
        self.rom_data.prg_rom[addr as usize % self.rom_data.prg_rom.len()]
    }
    // CHR ROM is fixed 8KB
    fn read_chr(&mut self, addr: u16) -> u8 {
        self.rom_data.chr_mem.read(addr as usize)
    }
    // NROM doesn't actually support CHR RAM but some homebrew games use this mapper with RAM
    fn write_chr(&mut self, addr: u16, value: u8) {
        self.rom_data.chr_mem.write(addr as usize, value);
    }
    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }
}
