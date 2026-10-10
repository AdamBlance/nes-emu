mod addressing;
pub mod instructions;
pub mod step;

use crate::emulator::nes::cpu::instructions::ControlSequence;
use crate::nes::cpu::instructions::Instr;
use crate::util::{concat_u8, get_bit};
use modular_bitfield::bitfield;
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Default, Debug, Serialize, Deserialize)]
pub struct Cpu {
    pub reg: Registers,
    pub interrupts: Interrupts,
    pub ireg: InternalRegisters,
    pub debug: CpuDebug,
}

#[bitfield]
#[derive(Clone, Copy, Default, Debug, Serialize, Deserialize)]
pub struct Status {
    was_negative: bool,
    overflow: bool,
    _padding: bool,
    in_break: bool,
    decimal: bool,
    interrupt_disable: bool,
    was_zero: bool,
    carry_out: bool,
}

impl Status {
    fn as_byte(&self) -> u8 {
        const ALWAYS_SET_BIT: u8 = 0b0010_0000;
        let [byte] = self.bytes;
        byte | ALWAYS_SET_BIT
    }
}

#[derive(Copy, Clone, Default, Debug, Serialize, Deserialize)]
pub struct Registers {
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub s: u8,
    pub status: Status,
    pub pc: MemAddress,
}

#[derive(Clone, Copy, Debug, Default)]
struct MemAddress(u16);
impl MemAddress {
    fn set_upper(&mut self, byte: u8) {
        self.0 = (0x00FF & self.0) | ((byte as u16) << 8);
    }
    fn set_lower(&mut self, byte: u8) -> self {
        self.0 = (0xFF00 & self.0) | byte as u16;
    }
    fn upper(&self) -> u8 {
        (self.0 >> 8) as u8
    }
    fn lower(&self) -> u8 {
        self.0 as u8
    }
    fn increment(&mut self) {
        self.0 = self.0.wrapping_add(1);
    }
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
pub struct InternalRegisters {
    pub data: u8, // Internal working register used by instructions
    pub address: MemAddress,
    pub pointer_address: MemAddress,
    pub branch_offset: u8, // Value to offset PC by when branching
    pub carry_out: bool, // Set when lower 8 bits of address/PC/pointer overflows when offset is added
    pub open_bus: u8,    // Data bus that can be read by reading unused memory locations
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
            ..Default::default()
        }
    }

    pub fn clear_internal_registers(&mut self) {
        // Persist open bus
        self.ireg = InternalRegisters {
            open_bus: self.ireg.open_bus,
            ..Default::default()
        };
    }
}
