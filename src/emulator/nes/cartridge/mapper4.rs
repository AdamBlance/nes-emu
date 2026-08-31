use super::cartridge_def::{CartMemory, Cartridge, Mirroring, RomConfig, KB};
use crate::util::get_bit;
use serde::{Deserialize, Serialize};
use std::rc::Rc;

#[derive(Clone, Serialize, Deserialize)]
pub struct CartridgeM4 {
    rom_data: CartMemory,

    bank_index: u8,

    prg_bank_0_or_2: usize,
    prg_bank_1: usize,

    chr_2kb_bank_0: usize,
    chr_2kb_bank_1: usize,

    chr_1kb_bank_0: usize,
    chr_1kb_bank_1: usize,
    chr_1kb_bank_2: usize,
    chr_1kb_bank_3: usize,

    prg_fixed_bank_select: bool,
    chr_bank_size_select: bool,

    mirroring: Mirroring,

    scanline_counter_init: u8,
    scanline_counter_curr: u8,

    last_a12_value: bool,

    scanline_counter_reset_flag: bool,

    irq_enable: bool,

    interrupt_request: bool,

    a12_filtering_counter: u8,
}

impl CartridgeM4 {
    pub fn new(rom_config: RomConfig) -> CartridgeM4 {
        CartridgeM4 {
            rom_data: rom_config.data,
            bank_index: 0,
            prg_bank_0_or_2: 0,
            prg_bank_1: 0,
            chr_2kb_bank_0: 0,
            chr_2kb_bank_1: 0,
            chr_1kb_bank_0: 0,
            chr_1kb_bank_1: 0,
            chr_1kb_bank_2: 0,
            chr_1kb_bank_3: 0,
            prg_fixed_bank_select: false,
            chr_bank_size_select: false,
            mirroring: Mirroring::Vertical,
            scanline_counter_init: 0,
            scanline_counter_curr: 0,
            last_a12_value: false,
            scanline_counter_reset_flag: false,
            irq_enable: false,
            interrupt_request: false,
            a12_filtering_counter: 0,
        }
    }
}

#[typetag::serde]
impl Cartridge for CartridgeM4 {
    // MMC3 can optionally have PRG RAM
    fn read_prg_ram(&mut self, addr: u16) -> Option<u8> {
        self.rom_data.prg_ram.as_ref()?.get((addr - 0x6000) as usize).cloned()
    }
    fn write_prg_ram(&mut self, addr: u16, byte: u8) {
        if let Some(ram) = self.rom_data.prg_ram.as_mut() {
            Rc::make_mut(ram)[(addr - 0x6000) as usize] = byte;
        }
    }

    fn read_prg_rom(&self, addr: u16) -> u8 {
        let base_bank_addr = match (addr, self.prg_fixed_bank_select) {
            (0xA000..=0xBFFF, _) => self.prg_bank_1 * 8 * KB + (addr as usize - 0xA000),
            (0xE000..=0xFFFF, _) => self.rom_data.prg_rom.len() - 8 * KB + (addr as usize - 0xE000),

            (0x8000..=0x9FFF, false) => self.prg_bank_0_or_2 * 8 * KB + (addr as usize - 0x8000),
            (0xC000..=0xDFFF, false) => {
                self.rom_data.prg_rom.len() - 16 * KB + (addr as usize - 0xC000)
            }

            (0x8000..=0x9FFF, true) => {
                self.rom_data.prg_rom.len() - 16 * KB + (addr as usize - 0x8000)
            }
            (0xC000..=0xDFFF, true) => self.prg_bank_0_or_2 * 8 * KB + (addr as usize - 0xC000),

            _ => unreachable!(),
        };
        self.rom_data.prg_rom[base_bank_addr]
    }
    fn write_prg_rom(&mut self, addr: u16, byte: u8) {
        let even = addr % 2 == 0;

        let ubyte = byte as usize;

        match (addr, even) {
            (0x8000..=0x9FFF, true) => {
                // println!("Even write!");
                self.bank_index = byte & 0b0000_0111;
                self.prg_fixed_bank_select = (byte & 0b0100_0000) > 0;
                self.chr_bank_size_select = (byte & 0b1000_0000) > 0;
            }
            (0x8000..=0x9FFF, false) => {
                // println!("Odd write!");
                match self.bank_index {
                    0b000 => self.chr_2kb_bank_0 = ubyte & 0b1111_1110,
                    0b001 => self.chr_2kb_bank_1 = ubyte & 0b1111_1110,

                    0b010 => self.chr_1kb_bank_0 = ubyte,
                    0b011 => self.chr_1kb_bank_1 = ubyte,
                    0b100 => self.chr_1kb_bank_2 = ubyte,
                    0b101 => self.chr_1kb_bank_3 = ubyte,

                    0b110 => self.prg_bank_0_or_2 = ubyte & 0b0011_1111,
                    0b111 => self.prg_bank_1 = ubyte & 0b0011_1111,
                    _ => unreachable!(),
                }
            }
            (0xA000..=0xBFFF, true) => {
                self.mirroring = if byte & 1 == 0 {
                    Mirroring::Vertical
                } else {
                    Mirroring::Horizontal
                }
            }
            (0xA000..=0xBFFF, false) => {
                // PRG RAM write protect, omitted for now
            }
            (0xC000..=0xDFFF, true) => {
                self.scanline_counter_init = byte;
            }
            (0xC000..=0xDFFF, false) => {
                self.scanline_counter_reset_flag = true;
            }
            (0xE000..=0xFFFF, true) => {
                self.irq_enable = false;
                self.interrupt_request = false;
            }
            (0xE000..=0xFFFF, false) => {
                self.irq_enable = true;
            }
            _ => unreachable!(),
        }
    }

    fn read_chr(&mut self, addr: u16) -> u8 {
        let uaddr = addr as usize;

        let chr_addr = if !self.chr_bank_size_select {
            match addr {
                0x0000..=0x07FF => self.chr_2kb_bank_0 * KB + uaddr,
                0x0800..=0x0FFF => self.chr_2kb_bank_1 * KB + (uaddr - 0x0800),

                0x1000..=0x13FF => self.chr_1kb_bank_0 * KB + (uaddr - 0x1000),
                0x1400..=0x17FF => self.chr_1kb_bank_1 * KB + (uaddr - 0x1400),
                0x1800..=0x1BFF => self.chr_1kb_bank_2 * KB + (uaddr - 0x1800),
                0x1C00..=0x1FFF => self.chr_1kb_bank_3 * KB + (uaddr - 0x1C00),
                _ => unreachable!(),
            }
        } else {
            match addr {
                0x0000..=0x03FF => self.chr_1kb_bank_0 * KB + uaddr,
                0x0400..=0x07FF => self.chr_1kb_bank_1 * KB + (uaddr - 0x0400),
                0x0800..=0x0BFF => self.chr_1kb_bank_2 * KB + (uaddr - 0x0800),
                0x0C00..=0x0FFF => self.chr_1kb_bank_3 * KB + (uaddr - 0x0C00),

                0x1000..=0x17FF => self.chr_2kb_bank_0 * KB + (uaddr - 0x1000),
                0x1800..=0x1FFF => self.chr_2kb_bank_1 * KB + (uaddr - 0x1800),
                _ => unreachable!(),
            }
        };
        self.rom_data.chr_mem.read(chr_addr)
    }

    fn asserting_irq(&mut self) -> bool {
        self.interrupt_request
    }

    fn ppu_tick(&mut self, addr_bus: u16) {
        // https://archive.nes.science/nesdev-forums/f2/t7718.xhtml
        // https://forums.nesdev.org/viewtopic.php?t=8807
        // https://www.nesdev.org/wiki/CPU_pinout

        self.a12_filtering_counter = self.a12_filtering_counter.saturating_sub(1);

        let new_a12_value = get_bit(addr_bus, 12);

        // If PPU has gone from fetching background tiles to fetching sprite tiles
        if !self.last_a12_value && new_a12_value {
            // If last rising edge was more than 16 PPU cycles ago, update scanline counter
            if self.a12_filtering_counter == 0 {
                if self.scanline_counter_curr == 0 || self.scanline_counter_reset_flag {
                    self.scanline_counter_curr = self.scanline_counter_init;
                    self.scanline_counter_reset_flag = false;
                } else {
                    self.scanline_counter_curr -= 1;

                    if self.irq_enable && self.scanline_counter_curr == 0 {
                        self.interrupt_request = true;
                    }
                }
            }
            // Reset the 16 PPU cycles ago counter whenever there is a rising edge on A12
            self.a12_filtering_counter = 16;
        }
        self.last_a12_value = new_a12_value;
    }

    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }
}
