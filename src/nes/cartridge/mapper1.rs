use super::cartridge_def::{CartMemory, Cartridge, Mirroring, RomConfig, KB};
use serde::{Deserialize, Serialize};
use std::rc::Rc;

#[derive(Clone, Serialize, Deserialize)]
pub struct CartridgeM1 {
    rom_data: CartMemory,
    mirroring: Mirroring,

    chr_bank_0: usize,
    chr_bank_1: usize,
    prg_bank: usize,

    prg_bank_mode: u8,
    chr_bank_mode: u8,

    shift_register: u8,
    write_counter: u8,

    consecutive_write_counter: u8,
}

impl CartridgeM1 {
    pub fn new(rom_config: RomConfig) -> CartridgeM1 {
        CartridgeM1 {
            rom_data: rom_config.data,
            mirroring: Mirroring::Vertical,

            chr_bank_0: 0,
            chr_bank_1: 0,
            prg_bank: 0,

            prg_bank_mode: 3,
            chr_bank_mode: 0,

            shift_register: 0,
            write_counter: 0,

            consecutive_write_counter: 0,
        }
    }

    fn calc_chr_addr(&self, addr: usize) -> usize {
        match self.chr_bank_mode {
            0 => (self.chr_bank_0 & 0b11110) * 8 * KB + addr,
            1 => match addr {
                0x0000..=0x0FFF => self.chr_bank_0 * 4 * KB + addr,
                0x1000..=0x1FFF => self.chr_bank_1 * 4 * KB + (addr - 0x1000),
                _ => unreachable!(),
            },
            _ => unreachable!(),
        }
    }
}

#[typetag::serde]
impl Cartridge for CartridgeM1 {
    // MMC1 can optionally have PRG RAM
    fn read_prg_ram(&mut self, addr: u16) -> Option<u8> {
        self.rom_data.prg_ram.as_ref()?.get((addr - 0x6000) as usize).cloned()
    }
    fn write_prg_ram(&mut self, addr: u16, byte: u8) {
        if let Some(ram) = self.rom_data.prg_ram.as_mut() {
            Rc::make_mut(ram)[(addr - 0x6000) as usize] = byte;
        }
    }

    fn read_prg_rom(&self, addr: u16) -> u8 {
        let addru = addr as usize;
        match self.prg_bank_mode {
            0 | 1 => self.rom_data.prg_rom[(self.prg_bank & 0b11110) * 32 * KB + (addru - 0x8000)],
            2 => match addr {
                0x8000..=0xBFFF => self.rom_data.prg_rom[addru - 0x8000],
                0xC000..=0xFFFF => {
                    self.rom_data.prg_rom[(self.prg_bank * 16 * KB) + (addru - 0xC000)]
                }
                _ => unreachable!(),
            },
            3 => match addr {
                0x8000..=0xBFFF => {
                    self.rom_data.prg_rom[(self.prg_bank * 16 * KB) + (addru - 0x8000)]
                }
                0xC000..=0xFFFF => {
                    self.rom_data.prg_rom
                        [(self.rom_data.prg_rom.len() - 16 * KB) + (addru - 0xC000)]
                }
                _ => unreachable!(),
            },
            _ => unreachable!(),
        }
    }

    fn write_prg_rom(&mut self, addr: u16, byte: u8) {
        if (byte & 0b1000_0000) > 0 {
            self.shift_register = 0;
            self.write_counter = 0;
            self.prg_bank_mode = 3;
        }
        // Ignore consecutive writes
        else if self.consecutive_write_counter == 0 {
            self.consecutive_write_counter = 2;

            self.shift_register >>= 1;
            self.shift_register |= (byte & 1) << 4;
            self.write_counter += 1;

            if self.write_counter == 5 {
                match addr {
                    // Control register
                    0x8000..=0x9FFF => {
                        self.mirroring = match self.shift_register & 0b00011 {
                            0 => Mirroring::SingleScreenLower,
                            1 => Mirroring::SingleScreenUpper,
                            2 => Mirroring::Vertical,
                            3 => Mirroring::Horizontal,
                            _ => unreachable!(),
                        };
                        self.prg_bank_mode = (self.shift_register & 0b01100) >> 2;
                        self.chr_bank_mode = (self.shift_register & 0b10000) >> 4;
                    }
                    0xA000..=0xBFFF => self.chr_bank_0 = self.shift_register as usize,
                    0xC000..=0xDFFF => self.chr_bank_1 = self.shift_register as usize,
                    0xE000..=0xFFFF => self.prg_bank = (self.shift_register & 0b01111) as usize,
                    _ => unreachable!(),
                }
                self.shift_register = 0;
                self.write_counter = 0;
            }
        }
    }

    fn read_chr(&mut self, addr: u16) -> u8 {
        self.rom_data
            .chr_mem
            .read(self.calc_chr_addr(addr as usize))
    }
    fn write_chr(&mut self, addr: u16, byte: u8) {
        self.rom_data.chr_mem.write(addr as usize, byte);
    }

    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }

    fn cpu_tick(&mut self) {
        self.consecutive_write_counter = self.consecutive_write_counter.saturating_sub(1);
    }
}
