
#[derive(Copy, Clone)]
pub enum Mirroring {
    Vertical,
    Horizontal,
    SingleScreenLower,
    SingleScreenUpper,
}

const KB: usize = 0x400;


// All cartridges must implement this
pub trait Cartridge {
    fn read_prg_ram(&mut self, _addr: u16) -> u8 {0}
    fn write_prg_ram(&mut self, _addr: u16, _byte: u8) {}

    fn read_prg_rom(&mut self, addr: u16) -> u8;
    fn write_prg_rom(&mut self, _addr: u16, _byte: u8, _cpu_cycle: u64) {}

    fn read_chr(&mut self, addr: u16) -> u8;
    fn write_chr(&mut self, _addr: u16, _byte: u8) {}

    fn get_physical_ntable_addr(&self, addr: u16) -> u16;
}








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
    fn read_prg_rom(&mut self, addr: u16) -> u8 {
        self.prg_rom[addr as usize % self.prg_rom.len()]
    }
    // CHR ROM is fixed 8KB
    fn read_chr(&mut self, addr: u16) -> u8 {
        self.chr_rom[addr as usize]
    }    
    fn get_physical_ntable_addr(&self, addr: u16) -> u16 {
        basic_nametable_mirrroring(addr, self.mirroring)
    }
}








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







pub struct CartridgeM2 {
    pub prg_rom: Vec<u8>,
    pub chr_rom: Vec<u8>,

    pub chr_rom_is_ram: bool,
    pub bank_select: usize,

    pub mirroring: Mirroring,
}
impl CartridgeM2 {
    pub fn new(prg_rom: Vec<u8>, chr_rom: Vec<u8>, chr_rom_is_ram: bool, mirroring: Mirroring) -> CartridgeM2 {
        CartridgeM2 {
            prg_rom,
            chr_rom,
            chr_rom_is_ram,
            mirroring,

            bank_select: 0,
        }
    }
}
impl Cartridge for CartridgeM2 {
    // UxROM doesn't have PRG RAM support

    fn read_prg_rom(&mut self, addr: u16) -> u8 {
        match addr {
            // Swappable 16KB at start of cartridge range
            0x8000..=0xBFFF => self.prg_rom[(self.bank_select * 0x4000) + (addr as usize - 0x8000)],
            // Fixed 16KB at end of addressable range
            0xC000..=0xFFFF => self.prg_rom[(self.prg_rom.len() - 0x4000) + (addr as usize - 0xC000)],
            _ => unreachable!(),
        }
    }
    fn write_prg_rom(&mut self, _addr: u16, byte: u8, _cpu_cycle: u64) {
        self.bank_select = byte as usize;   
    }

    fn read_chr(&mut self, addr: u16) -> u8 {
        self.chr_rom[addr as usize]
    }
    fn write_chr(&mut self, addr: u16, byte: u8) {
        if self.chr_rom_is_ram {
            self.chr_rom[addr as usize] = byte;
        }
    }
    fn get_physical_ntable_addr(&self, addr: u16) -> u16 {
        basic_nametable_mirrroring(addr, self.mirroring)
    }

}

















// MMC3 goes here, will take some time to do

/*

    notes:
    
    6000-7FFF = optional ram
    8000-9FFF = 8KB PRG ROM bank
    A000-BFFF = 8KB PRG ROM bank, always switchable
    C000-DFFF = 8KB PRG ROM bank
    E000-FFFF = 8KB PRG ROM, always fixed to the last bank

    0000-07FF = 2KB CHR ROM bank
    0800-0FFF = 2KB CHR ROM bank
    1000-13FF = 1KB CHR ROM bank
    1400-17FF = 1KB CHR ROM bank
    1800-1BFF = 1KB CHR ROM bank
    1C00-1FFF = 1KB CHR ROM bank

    



    8000-9FFF -> mapping 

    EVEN
    
    bits 2,1,0 select a bank to configure

    bits 5,4,3 are unused

    bit 6 chooses which PRG bank is fixed and which is swappable
    when 0, 8000-9FFF is switchable, C000-DFFF fixed to second last bank
    when 1, 8000-9FFF is fixed to second last bank, C000-DFFF is switchable 

    bit 7 chooses whether CHR ROM is made up of 
    2*2KB + 4*1KB banks, or 4*1KB banks + 2*2KB banks

    ODD 

    the byte written selects the bank number of the bank selected by the even write


    A000-BFFF -> mirroring

    EVEN

    only bit 0 is considered, 0 for vertical mirroring, 1 for horizontal
    this bit is ignored if 4 screen mirroring is used

    ODD

    Write protection for ram, although doesn't really matter, can just leave it out for now




    scanline counter!

    so CHR rom is made up of two pattern tables, 0000-0FFF and 1000-1FFF
    
    if all background tiles are fetched from the 0000-0FFF and all sprite tiles are 
    fetched from 1000-1FFF, bit 12 of the address line will flip once every scanline,
    when sprite fetches begin.
    This can be used to count the scanline that is currently being rendered



    C000-DFFF -> irq stuff

    EVEN

    written byte specifies the reset value for the scanline counter
    the counter is initialised to this value when it reaches 0 or is reset


    ODD

    writing any value reloads the IRQ counter at the next rising edge of PPU addr bit 12


    E000-FFFF -> irq stuff 

    EVEN
    
    prevents future interrupts from being raised, but will allow any pending interrupt to complete

    ODD

    enables interrupts



    






*/



























fn basic_nametable_mirrroring(addr: u16, mirroring: Mirroring) -> u16 {
    // The physical nametables sit at 0x2000..=0x23FF and 0x2400..=0x27FF
    let vram_addr = match mirroring {
        Mirroring::Vertical => match addr {
            0x2000..=0x23FF => addr,
            0x2400..=0x27FF => addr,
            0x2800..=0x2BFF => addr - 0x800,
            0x2C00..=0x2FFF => addr - 0x800,
            _ => unreachable!(),
        }
        Mirroring::Horizontal => match addr {
            0x2000..=0x23FF => addr,
            0x2400..=0x27FF => addr - 0x400,
            0x2800..=0x2BFF => addr - 0x400,
            0x2C00..=0x2FFF => addr - 0x800,
            _ => unreachable!(),
        }
        Mirroring::SingleScreenLower => match addr {
            0x2000..=0x23FF => addr,
            0x2400..=0x27FF => addr - 0x400,
            0x2800..=0x2BFF => addr - 0x800,
            0x2C00..=0x2FFF => addr - 0xC00,
            _ => unreachable!(),
        }
        Mirroring::SingleScreenUpper => match addr {
            0x2000..=0x23FF => addr + 0x400,
            0x2400..=0x27FF => addr,
            0x2800..=0x2BFF => addr - 0x400,
            0x2C00..=0x2FFF => addr - 0x800,
            _ => unreachable!(),
        }
    };
    vram_addr - 0x2000
}
