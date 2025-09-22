pub mod instructions;
mod addressing;
pub mod step;

use crate::util::{concat_u8, get_bit};
use serde::{Deserialize, Serialize};
use crate::nes::cpu::instructions::Instr;

#[derive(Copy, Clone, Default, Debug, Serialize, Deserialize)]
pub struct Cpu {
    pub reg: Registers,
    pub interrupts: Interrupts,
    pub ireg: WorkingRegisters,
    // pub instr: Instr,
    pub debug: CpuDebug,
}

#[derive(Copy, Clone, Default, Debug, Serialize, Deserialize)]
pub struct Registers {
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
}

#[derive(Copy, Clone, Default, Debug, Serialize, Deserialize)]
pub struct Interrupts {
    pub prev_nmi_signal: bool,
    pub nmi_edge_detector_output: bool,
    pub nmi_pending: bool,
    pub prev_irq_signal: bool,
    pub irq_pending: bool,
    pub interrupt_vector: u16,
}

#[derive(Copy, Clone, Default, Debug, Serialize, Deserialize)]
pub struct WorkingRegisters {
    pub data: u8,                  // Internal working register used by instructions
    pub lower_address: u8,         // Lower 8 bits of address bus
    pub upper_address: u8,         // Upper 8 bits of address bus
    pub low_indirect_address: u8,  // Lower 8 bits of pointer (when using indirect addressing)
    pub high_indirect_address: u8, // Upper 8 bits of pointer (when using indirect addressing)
    pub branch_offset: u8,         // Value to offset PC by when branching
    pub carry_out: bool,           // Set when lower 8 bits of address/PC/pointer overflows when offset is added
    pub open_bus: u8,              // Data bus that can be read by reading unused memory locations
}


#[derive(Copy, Clone, Default, Debug, Serialize, Deserialize)]
pub struct CpuDebug {
    pub cycles: u64,
    pub instruction_count: u64,
}

impl Cpu {
    pub fn new(initial_pc: u16) -> Cpu {
        Cpu {
            reg: Registers {
                pc: initial_pc,
                s: 0xFD,
                p_i: true,
                ..Default::default()
            },
            debug: CpuDebug {
                cycles: 8,
                instruction_count: 0,
            },
            // instr: Instr::DUMMY_INSTR,
            ..Default::default()
        }
    }

    pub fn set_upper_pc(&mut self, byte: u8) {
        self.reg.pc &= 0b00000000_11111111;
        self.reg.pc |= (byte as u16) << 8;
    }
    pub fn set_lower_pc(&mut self, byte: u8) {
        self.reg.pc &= 0b11111111_00000000;
        self.reg.pc |= byte as u16;
    }

    pub fn get_p(&self) -> u8 {
        (self.reg.p_n as u8) << 7 |
        (self.reg.p_v as u8) << 6 |
        // 1 << 5 |
        (self.reg.p_d as u8) << 3 |
        (self.reg.p_i as u8) << 2 |
        (self.reg.p_z as u8) << 1 |
        (self.reg.p_c as u8)
    }
    pub fn set_p(&mut self, byte: u8) {
        self.reg.p_n = get_bit(byte, 7);
        self.reg.p_v = get_bit(byte, 6);
        self.reg.p_d = get_bit(byte, 3);
        self.reg.p_i = get_bit(byte, 2);
        self.reg.p_z = get_bit(byte, 1);
        self.reg.p_c = get_bit(byte, 0);
    }

    pub fn get_address(&self) -> u16 {
        concat_u8(self.ireg.upper_address, self.ireg.lower_address)
    }
    pub fn get_pointer(&self) -> u16 {
        concat_u8(self.ireg.high_indirect_address, self.ireg.low_indirect_address)
    }

    pub fn clear_internal_registers(&mut self) {
        // Persist open bus
        self.ireg = WorkingRegisters { open_bus: self.ireg.open_bus, ..Default::default() };
    }
}
