use crate::util::{concat_u8, get_bit};
use super::lookup_table::Instruction;

#[derive(Copy, Clone, Default)]
pub struct Cpu {
    // Registers
    pub a:   u8,
    pub x:   u8,
    pub y:   u8,
    pub s:   u8,
    pub p_n: bool,
    pub p_v: bool,
    pub p_d: bool,
    pub p_i: bool,
    pub p_z: bool,
    pub p_c: bool,
    pub pc:  u16,
    // Interrupts
    pub nmi_interrupt:     bool,
    pub nmi_internal_flag: bool,
    pub interrupt_request: bool,
    pub irq_internal_flag: bool,
    // Internal
    pub instruction:        Instruction,
    pub instruction_cycle:  i8,
    pub data:               u8,
    pub lower_address:      u8,
    pub upper_address:      u8,
    pub lower_pointer:      u8,
    pub upper_pointer:      u8,
    pub branch_offset:      u8,
    pub branching:          bool,
    pub internal_carry_out: bool,
    pub cycles:             u64,
    // Debugging
    pub instruction_count: u64,
    pub target: u64,
}

impl Cpu {
    pub fn new() -> Cpu {Default::default()}

    pub fn set_upper_pc(&mut self, byte: u8) {
        self.pc &= 0b00000000_11111111;
        self.pc |= (byte as u16) << 8;
    }
    pub fn set_lower_pc(&mut self, byte: u8) {
        self.pc &= 0b11111111_00000000;
        self.pc |= byte as u16;
    }

    pub fn get_p(&self) -> u8 {
        (self.p_n as u8) << 7 | 
        (self.p_v as u8) << 6 | 
        1 << 5 |
        (self.p_d as u8) << 3 |
        (self.p_i as u8) << 2 |
        (self.p_z as u8) << 1 |
        (self.p_c as u8)
    }
    pub fn set_p(&mut self, byte: u8) {
        self.p_n = get_bit(byte, 7);
        self.p_v = get_bit(byte, 6);
        self.p_d = get_bit(byte, 3);
        self.p_i = get_bit(byte, 2);
        self.p_z = get_bit(byte, 1);
        self.p_c = get_bit(byte, 0);
    }

    pub fn get_address(&self) -> u16 {
        concat_u8(self.upper_address, self.lower_address)
    }
    pub fn get_pointer(&self) -> u16 {
        concat_u8(self.upper_pointer, self.lower_pointer)
    }
}
