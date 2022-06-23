
use super::cartridge::{
    Cartridge, 
    Mirroring, 
    basic_nametable_mirrroring,
    KB,
};


pub struct CartridgeM1 {
    pub prg_ram: Vec<u8>,
    pub prg_rom: Vec<u8>,
    pub chr_rom: Vec<u8>,
    pub chr_rom_is_ram: bool,
    pub shift_register: u8,
    pub write_counter: u8,
    pub last_write_cycle: u64,
    pub mirroring: Mirroring,
    pub prg_bank_mode: u8,
    pub chr_bank_mode: u8,
    pub chr_bank_0: usize,
    pub chr_bank_1: usize,
    pub prg_bank: usize,
}

impl CartridgeM1 {
    pub fn new(prg_rom: Vec<u8>, chr_rom: Vec<u8>, chr_rom_is_ram: bool,) -> CartridgeM1 {
        CartridgeM1 {
            prg_ram: [0u8; 0x2000].to_vec(),
            prg_rom,
            chr_rom,
            chr_rom_is_ram,
            
            shift_register: 0,
            write_counter: 0,
            last_write_cycle: u64::MAX,
            mirroring: Mirroring::Vertical,
            prg_bank_mode: 3,
            chr_bank_mode: 0,
            chr_bank_0: 0,
            chr_bank_1: 0,
            prg_bank: 0,
        }
    }

    fn calc_chr_addr(&self, addr: u16) -> usize {
        let addru = addr as usize;
        match self.chr_bank_mode {
            0 => (self.chr_bank_0 & 0b11110) as usize * 8*KB + addru,
            1 => { match addr {
                0x0000..=0x0FFF => self.chr_bank_0 * 4*KB + addru,
                0x1000..=0x1FFF => self.chr_bank_1 * 4*KB + (addru - 0x1000),
                _ => unreachable!(),
            }}
            _ => unreachable!(),
        }
    }
}
impl Cartridge for CartridgeM1 {
    // MMC1 can optionally have PRG RAM
    fn read_prg_ram(&mut self, addr: u16) -> u8 {
        if !self.prg_ram.is_empty() {
            self.prg_ram[(addr - 0x6000) as usize]
        } else {
            0
        }
    }
    fn write_prg_ram(&mut self, addr: u16, byte: u8) {
        if !self.prg_ram.is_empty() {
            self.prg_ram[(addr - 0x6000) as usize] = byte;
        }
    }

    fn read_prg_rom(&mut self, addr: u16) -> u8 {
        let addru = addr as usize;
        match self.prg_bank_mode { // Use KB const for readability
            0 | 1 => self.prg_rom[(self.prg_bank & 0b11110) * 32*KB + (addru - 0x8000)],
            2 => { match addr {
                0x8000..=0xBFFF => self.prg_rom[addru - 0x8000],
                0xC000..=0xFFFF => self.prg_rom[(self.prg_bank * 16*KB) + (addru - 0xC000)],
                _ => unreachable!(),
            }}
            3 => { match addr {
                0x8000..=0xBFFF => {
                    // println!("First bank read {:04X}", addr);
                    self.prg_rom[(self.prg_bank * 16*KB) + (addru - 0x8000)]
                }
                0xC000..=0xFFFF => {
                    // println!("Second bank read {:04X}", addr);
                    self.prg_rom[(self.prg_rom.len() - 16*KB) + (addru - 0xC000)]
                }
                _ => unreachable!(),
            }}
            _ => unreachable!(),

        }
    }
    fn write_prg_rom(&mut self, addr: u16, byte: u8, cpu_cycle: u64) {

        // println!("ROM write - data {:08b}, addr {:04X}, cpu cycle {}", byte, addr, cpu_cycle);

        if (byte & 0b1000_0000) > 0 {
            self.shift_register = 0;
            self.write_counter = 0;
            self.prg_bank_mode = 3;
        } 
        // Ignore consecutive writes
        else if cpu_cycle - 1 != self.last_write_cycle {
            self.last_write_cycle = cpu_cycle;

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
        self.chr_rom[self.calc_chr_addr(addr)]
    }
    fn write_chr(&mut self, addr: u16, byte: u8) {
        if self.chr_rom_is_ram {
            let chr_addr = self.calc_chr_addr(addr);
            self.chr_rom[chr_addr] = byte;
        }
    }

    fn get_physical_ntable_addr(&self, addr: u16) -> u16 {
        basic_nametable_mirrroring(addr, self.mirroring)
    }

}