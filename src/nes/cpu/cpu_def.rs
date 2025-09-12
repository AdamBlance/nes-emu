use super::lookup_table::Instruction;
use crate::util::{concat_u8, get_bit};
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Default, Debug, Serialize, Deserialize)]
pub struct Cpu {
    // Registers
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub s: u8,
    pub p_n: bool,
    pub p_v: bool,
    pub p_d: bool,
    pub p_i: bool,
    pub p_z: bool,
    pub p_c: bool,
    pub pc: u16,
    // Interrupts
    pub prev_nmi_signal: bool,
    pub nmi_edge_detector_output: bool,
    pub nmi_pending: bool,
    pub prev_irq_signal: bool,
    pub irq_pending: bool,
    pub interrupt_vector: u16,
    // Internal
    pub instruction: Instruction,
    pub instruction_done: bool,
    pub instruction_cycle: i8,
    pub interrupt_cycle: i8,
    pub data: u8,
    pub lower_address: u8,
    pub upper_address: u8,
    pub low_indirect_address: u8,
    pub high_indirect_address: u8,
    pub branch_offset: u8,
    pub branching: bool,
    pub internal_carry_out: bool,
    pub cycles: u64,
    pub ppustatus_read_time: (i32, i32),
    pub open_bus: u8,
    // Debugging
    pub instruction_count: u64,
}

impl Cpu {
    pub fn new(initial_pc: u16) -> Cpu {
        Cpu {
            pc: initial_pc,
            cycles: 8,
            p_i: true,
            s: 0xFD,
            ..Default::default()
        }
    }

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
        // 1 << 5 |
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
        concat_u8(self.high_indirect_address, self.low_indirect_address)
    }
}
