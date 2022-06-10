
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
    fn update_state(&mut self, addr: u16, byte_written: u8);   
}


pub struct Mapper0 {
    pub prg_size: usize
}

impl Mapper for Mapper0 {
    fn get_raw_prg_address(&mut self, addr: u16) -> usize {
        addr as usize % self.prg_size
    }
    fn update_state(&mut self, _addr: u16, _byte_written: u8) {}
}




pub struct Mapper2 {
    pub prg_size: usize,
    pub bank_select: usize,
}

impl Mapper for Mapper2 {
    fn get_raw_prg_address(&mut self, addr: u16) -> usize {

        match addr {
            0x8000..=0xBFFF => ((self.bank_select * 0x4000) + (addr as usize - 0x8000)),
            0xC000..=0xFFFF => ((self.prg_size - 0x4000) + (addr as usize - 0xC000)),
            _ => unreachable!(),
        }
    }
    fn update_state(&mut self, _addr: u16, byte_written: u8) {
        self.bank_select = byte_written as usize;
    }
}
