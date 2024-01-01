use crate::nes::cpu::lookup_table::{Mode, INSTRUCTIONS};
use crate::util::concat_u8;

#[derive(Debug, Copy, Clone)]
pub enum InstrBytes {
    I1(u8),
    I2(u8, u8),
    I3(u8, u8, u8),
}

#[derive(Debug, Copy, Clone)]
pub struct CpuDebuggerInstruction {
    pub opc_addr: u16,
    pub bytes: InstrBytes,
}

impl CpuDebuggerInstruction {
    pub fn debug_string(&self) -> String {
        let instr = match self.bytes {
            InstrBytes::I1(opc) => INSTRUCTIONS[opc as usize],
            InstrBytes::I2(opc, ..) => INSTRUCTIONS[opc as usize],
            InstrBytes::I3(opc, ..) => INSTRUCTIONS[opc as usize],
        };
        match self.bytes {
            InstrBytes::I1(_) => match instr.mode {
                Mode::Accumulator | Mode::Implied => {
                    format!("{:04X} {:#?}", self.opc_addr, instr.name)
                }
                _ => unreachable!(),
            },
            InstrBytes::I2(_, arg1) => match instr.mode {
                Mode::Immediate => format!("{:04X} {:#?} {:02X}", self.opc_addr, instr.name, arg1),
                Mode::ZeroPage => {
                    format!("{:04X} {:#?} ZP[{:02X}]", self.opc_addr, instr.name, arg1)
                }
                Mode::ZeroPageX => {
                    format!("{:04X} {:#?} ZP[{:02X}+X]", self.opc_addr, instr.name, arg1)
                }
                Mode::ZeroPageY => {
                    format!("{:04X} {:#?} ZP[{:02X}+Y]", self.opc_addr, instr.name, arg1)
                }
                Mode::IndirectX => format!(
                    "{:04X} {:#?} MEM[ ZP16[{:02X}+X] ]",
                    self.opc_addr, instr.name, arg1
                ),
                Mode::IndirectY => format!(
                    "{:04X} {:#?} MEM[ ZP16[{:02X}]+Y ]",
                    self.opc_addr, instr.name, arg1
                ),
                Mode::Relative => format!("{:04X} {:#?} (offset)", self.opc_addr, instr.name),
                _ => unreachable!(),
            },
            InstrBytes::I3(_, arg1, arg2) => match instr.mode {
                Mode::Absolute => format!(
                    "{:04X} {:#?} MEM[{:04X}]",
                    self.opc_addr,
                    instr.name,
                    concat_u8(arg2, arg1)
                ),
                Mode::AbsoluteX => format!(
                    "{:04X} {:#?} MEM[{:04X}+X]",
                    self.opc_addr,
                    instr.name,
                    concat_u8(arg2, arg1)
                ),
                Mode::AbsoluteY => format!(
                    "{:04X} {:#?} MEM[{:04X}+Y]",
                    self.opc_addr,
                    instr.name,
                    concat_u8(arg2, arg1)
                ),
                Mode::AbsoluteI => format!(
                    "{:04X} {:#?} MEM[ MEM16[{:04X}] ]",
                    self.opc_addr,
                    instr.name,
                    concat_u8(arg2, arg1)
                ),
                _ => unreachable!(),
            },
        }
    }
}
