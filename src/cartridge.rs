
enum Mirroring {
    Vertical,
    Horizontal,
    SingleScreenLower,
    SingleScreenUpper,
}


pub trait Cartridge {
    fn read_prg(&mut self, addr: u16) -> u8;
    fn read_chr(&mut self, addr: u16) -> u8;

    fn write_prg(&mut self, addr: u16, byte: u8, cpu_cycle: u64) {}
    fn write_chr(&mut self, addr: u16, byte: u8, cpu_cycle: u64) {}

    fn get_physical_ntable_addr(&self, addr: u16) -> u16;
}




pub struct CartridgeM0 {
    pub prg_rom: Vec<u8>,
    pub chr_rom: Vec<u8>,

    pub mirroring: Mirroring,
}

impl Cartridge for CartridgeM0 {
    // NROM-128 is 16KB mirrored twice, NROM-256 is 32KB
    fn read_prg(&mut self, addr: u16) -> u8 {
        addr as usize % self.prg_rom.len()
    }
    // CHR ROM is fixed 8KB
    fn read_chr(&mut self, addr: u16) -> u8 {
        addr as usize
    }
    
    fn get_physical_ntable_addr(&self, addr: u16) -> u16 {
        basic_nametable_mirrroring(addr, self.vertical_mirroring)
    }
}








pub struct CartridgeM1 {
    pub prg_ram: Vec<u8>,
    pub prg_rom: Vec<u8>,
    pub chr_rom: Vec<u8>,

    pub shift_register: u8,
    pub write_counter: u8,
    pub last_write_cycle: u64,

    
}

impl Cartridge for CartridgeM1 {

    fn read_prg(&mut self, addr: u16) -> u8 {
        
    }
    fn read_chr(&mut self, addr: u16) -> u8 {
        
    }

    fn write_prg(&mut self, addr: u16, byte: u8, cpu_cycle: u64) {
        if (byte & 0b1000_0000) > 0 {
            self.shift_register = 0;
            self.write_counter = 0;
        } 
        // Ignore consecutive writes
        else if (cpu_cycle - self.last_write_cycle) != 1 {
            self.last_write_cycle = cpu_cycle;

            self.shift_register >>= 1;
            self.shift_register |= (byte & 1) << 4;
            self.write_counter += 1;

            if self.write_counter == 5 {

            }

        }
    }

    fn write_chr(&mut self, addr: u16, byte: u8, cpu_cycle: u64) {
        
    }

    fn get_physical_ntable_addr(&self, addr: u16) -> u16 {
        
    }



}









pub struct CartridgeM2 {
    pub prg_rom: Vec<u8>,
    pub chr_rom: Vec<u8>,

    pub chr_rom_is_ram: bool,
    pub bank_select: usize,
}

impl Cartridge for CartridgeM2 {
    fn read_prg(&mut self, addr: u16) -> usize {
        match addr {
            // Swappable 16KB at start of cartridge range
            0x8000..=0xBFFF => (self.bank_select * 0x4000) + (addr as usize - 0x8000),
            // Fixed 16KB at end of addressable range
            0xC000..=0xFFFF => (self.prg_size - 0x4000) + (addr as usize - 0xC000),
        }
    }
    fn read_chr(&mut self, addr: u16) -> u8 {
        addr as usize
    }
    fn write_prg(&mut self, addr: u16, byte: u8, cpu_cycle: u64) {
        self.bank_select = byte as usize;   
    }
    fn write_chr(&mut self, addr: u16, byte: u8, cpu_cycle: u64) {
        if self.chr_rom_is_ram {
            self.chr_rom[addr as usize] = byte;
        }
    }

}













fn basic_nametable_mirrroring(addr: u16, mirroring: Mirroring) -> u16 {
    // The physical nametables sit at 0x2000..=0x23FF and 0x2400..=0x27FF
    if vertical_mirroring { match addr {
        0x2000..=0x23FF => addr,
        0x2400..=0x27FF => addr,
        0x2800..=0x2BFF => addr - 0x800,
        0x2C00..=0x2FFF => addr - 0x800,
    }} 
    else { match addr {
        0x2000..=0x23FF => addr,
        0x2400..=0x27FF => addr - 0x400,
        0x2800..=0x2BFF => addr - 0x400,
        0x2C00..=0x2FFF => addr - 0x800,
    }}
}


pub struct Cartridge {
    pub prg_ram: Vec<u8>,  
    pub prg_rom: Vec<u8>,  
    pub chr_rom: Vec<u8>,
    pub chr_rom_is_ram: bool,
    pub mapper: Box<dyn Mapper>,
}

impl Cartridge {
    pub fn new(ines_data: Vec<u8>) -> Cartridge {
        // Information extracted from iNES header
        let num_prg_16kb_chunks   = ines_data[4] as usize;
        let num_chr_8kb_chunks    = ines_data[5] as usize;
        let has_prg_ram           = (ines_data[6] & 0b0010) > 0;

        let vertical_mirroring    = (ines_data[6] & 0b0001) > 0;
        let four_screen_mirroring = (ines_data[6] & 0b1000) > 0;
        
        let mapper_id             = (ines_data[6] >> 4) 
                                  | (ines_data[7] & 0b1111_0000);

        let chr_rom_is_ram = num_chr_8kb_chunks == 0;

        // Program ROM begins immediately after 16 byte header
        let prg_end = 16 + (num_prg_16kb_chunks * 0x4000);
        let chr_end = prg_end + (num_chr_8kb_chunks * 0x2000);
        let prg_size = prg_end - 16; 

        /*
            Things that can happen with a mapper
            There is PRG RAM that can be written to (this depends on the mapper)
            NROM doesn't support this, for the NES at least

            There is CHR RAM
        */

        let mapper: Box<dyn Mapper> = match mapper_id {
            0 => Box::new(Mapper0 {prg_size, vertical_mirroring}),
            _ => panic!(),
        };

        let chr_rom_is_ram = chr_blocks == 0;

        let chr_rom = if !chr_rom_is_ram {
            ines_data[prg_end..chr_end].to_vec()
        } else {
            [0u8; 0x2000].to_vec()
        };

        Cartridge {
            prg_rom: ines_data[prg_start..prg_end].to_vec(),
            chr_rom,
            mapper,
            v_mirroring: (ines_data[6] & 1) > 0,
            chr_rom_is_ram,
        }
    }
}