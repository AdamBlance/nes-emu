/*

    A mapper maps arbitrary ROM data into the fixed address space of the NES
    In our case, we have the cartridge object which has all PRG rom in it. 
    We just need to call some function to update the mapper's registers every time we write to 
    an address > 0x8000. 
    That will update the mapper's internal registers
    Then, when we try to read from 0x8000+, the mapper object gives us an index into the 
    prg rom vector in the cartridge!

*/

pub trait Mapper {
    fn get_raw_prg_address(&mut self, addr: u16) -> usize;
    fn get_raw_chr_address(&mut self, addr: u16) -> usize;

    // CPU cycle is included so that mappers can check for writes on consecutive CPU cycles
    // This is done because of the dummy writes done by RMW instructions
    // Mapper 1 does this

    fn prg_write(&mut self, addr: u16, byte_written: u8, cpu_cycle: u64) {}
    fn chr_write(&mut self, addr: u16, byte_written: u8, cpu_cycle: u64) {}

    fn get_physical_ntable_addr(&self, addr: u16) -> u16;

}


pub struct Mapper0 {
    pub prg_size: usize
    pub vertical_mirroring: bool,
}

impl Mapper for Mapper0 {
    fn get_raw_prg_address(&mut self, addr: u16) -> usize {
        addr as usize % self.prg_size
    }
    fn get_raw_chr_address(&mut self, addr: u16) -> usize {
        addr as usize
    }
    fn get_physical_ntable_addr(&self, addr: u16) -> u16 {
        
    }
    

}


pub struct Mapper1 {
    pub prg_size: usize,
    pub shift_register: u8,
    pub write_counter: u8,
    pub last_write_cycle: u64,

}

impl Mapper for Mapper1 {
    fn prg_write(&mut self, addr: u16, byte_written: u8, cpu_cycle: u64) {
        if (byte_written & 0b1000_0000) > 0 {
            self.shift_register = 0;
            self.write_counter = 0;
        } else {
            if cpu_cycle - self.last_write_cycle != 1 {
                self.shift_register <<= 1;
                self.shift_register |= byte_written & 1;
                self.write_counter += 1;
            }
            if self.write_counter == 5 {

            }
            
        }
        self.last_write_cycle = cpu_cycle;
    }
}



pub struct Mapper2 {
    pub prg_size: usize,
    pub bank_select: usize,
    pub chr_ram: bool,
}

impl Mapper for Mapper2 {
    fn get_raw_prg_address(&mut self, addr: u16) -> usize {

        match addr {
            0x8000..=0xBFFF => ((self.bank_select * 0x4000) + (addr as usize - 0x8000)),
            0xC000..=0xFFFF => ((self.prg_size - 0x4000) + (addr as usize - 0xC000)),
            _ => unreachable!(),
        }
    }

    fn get_raw_chr_address(&mut self, addr: u16) -> usize {
        addr as usize
    }

    fn prg_write(&mut self, addr: u16, byte_written: u8, cpu_cycle: u64) {
        self.bank_select = byte_written as usize;
    }

}
