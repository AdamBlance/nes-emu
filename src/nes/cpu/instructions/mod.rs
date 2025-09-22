use crate::nes::cpu::instructions::branch::{BranchInstr, BranchOpc};
use crate::nes::cpu::instructions::control::{ControlInstr, ControlOpc};
use crate::nes::cpu::instructions::interrupts::{Interrupt, InterruptType};
use crate::nes::cpu::instructions::jump::{JumpInstr, JumpOpc, JumpType};
use crate::nes::cpu::instructions::memory::{
    AddressingConfig, AddressingMode, MemoryAccessType, MemoryInstr, MemoryOpc,
};
use crate::nes::cpu::instructions::nonmemory::{NonMemoryInstr, NonMemoryOpc};
use serde::{Deserialize, Serialize};
use crate::nes::{cpu, Nes};

mod branch;
mod control;
mod jump;
mod memory;
mod nonmemory;
mod common;
pub mod interrupts;

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum Instr {
    Branch(BranchInstr),
    Control(ControlInstr),
    Jump(JumpInstr),
    Memory(MemoryInstr),
    NonMemory(NonMemoryInstr),
    Jam,
    // For the sake of simplicity, an interrupt is considered an instruction
    Interrupt(Interrupt)
}

impl Default for Instr {
    fn default() -> Self {
        Self::NonMemory(Default::default())
    }
}
impl Instr {
    pub const DUMMY_INSTR: Instr = Self::NonMemory(NonMemoryInstr::DUMMY_INSTR);
    // TODO: Figure out how to have the do_next_cycle method in here!
    pub fn is_finished(&self) -> bool {
        match self {
            Self::Branch(instr) => instr.is_finished(),
            Self::Control(instr) => instr.is_finished(),
            Self::Jump(instr) => instr.is_finished(),
            Self::Memory(instr) => instr.is_finished(),
            Self::NonMemory(instr) => instr.is_finished(),
            Self::Interrupt(interrupt) => interrupt.is_finished(),
            Self::Jam => true
        }
    }
    pub fn new_interrupt(interrupt_type: InterruptType) -> Self {
        Self::Interrupt(Interrupt::new(interrupt_type))
    }
    pub fn do_next_cycle(&mut self, nes: &mut Nes) {
        // println!("instr: {:?}", self);
        match self {
            Instr::Branch(instr) => instr.do_next_instruction_cycle(nes),
            Instr::Control(instr) => instr.do_next_instruction_cycle(nes),
            Instr::Jump(instr) => instr.do_next_instruction_cycle(nes),
            Instr::Memory(instr) => instr.do_next_instruction_cycle(nes),
            Instr::NonMemory(instr) => instr.do_next_instruction_cycle(nes),
            Instr::Interrupt(interrupt) => interrupt.do_next_interrupt_cycle(nes),
            Instr::Jam => panic!("JAM!")
        };
    }
    pub fn from_opcode(opcode: u8) -> Self {
        match opcode {
            0x00 => Instr::Control(ControlInstr::new(ControlOpc::BRK)),
            0x01 => Instr::Memory(MemoryInstr::new(
                MemoryOpc::ORA,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::IndirectX,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0x02 => Instr::Jam,
            0x03 => Instr::Memory(MemoryInstr::new(
                MemoryOpc::SLO,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::IndirectX,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0x04 => Instr::Memory(MemoryInstr::new(
                MemoryOpc::NOP,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPage,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0x05 => Instr::Memory(MemoryInstr::new(
                MemoryOpc::ORA,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPage,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0x06 => Instr::Memory(MemoryInstr::new(
                MemoryOpc::ASL,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPage,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0x07 => Instr::Memory(MemoryInstr::new(
                MemoryOpc::SLO,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPage,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0x08 => Instr::Control(ControlInstr::new(ControlOpc::PHP)),
            0x09 => Instr::Memory(MemoryInstr::new(
                MemoryOpc::ORA,
                AddressingConfig::Immediate,
            )),
            0x0A => Instr::NonMemory(NonMemoryInstr::new(NonMemoryOpc::ASL)),
            0x0B => Instr::Memory(MemoryInstr::new(
                MemoryOpc::ANC,
                AddressingConfig::Immediate,
            )),
            0x0C => Instr::Memory(MemoryInstr::new(
                MemoryOpc::NOP,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::Absolute,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0x0D => Instr::Memory(MemoryInstr::new(
                MemoryOpc::ORA,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::Absolute,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0x0E => Instr::Memory(MemoryInstr::new(
                MemoryOpc::ASL,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::Absolute,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0x0F => Instr::Memory(MemoryInstr::new(
                MemoryOpc::SLO,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::Absolute,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0x10 => Instr::Branch(BranchInstr::new(BranchOpc::BPL)),
            0x11 => Instr::Memory(MemoryInstr::new(
                MemoryOpc::ORA,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::IndirectY,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0x12 => Instr::Jam,
            0x13 => Instr::Memory(MemoryInstr::new(
                MemoryOpc::SLO,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::IndirectY,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0x14 => Instr::Memory(MemoryInstr::new(
                MemoryOpc::NOP,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPageX,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0x15 => Instr::Memory(MemoryInstr::new(
                MemoryOpc::ORA,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPageX,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0x16 => Instr::Memory(MemoryInstr::new(
                MemoryOpc::ASL,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPageX,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0x17 => Instr::Memory(MemoryInstr::new(
                MemoryOpc::SLO,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPageX,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0x18 => Instr::NonMemory(NonMemoryInstr::new(NonMemoryOpc::CLC)),
            0x19 => Instr::Memory(MemoryInstr::new(
                MemoryOpc::ORA,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::AbsoluteY,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0x1A => Instr::NonMemory(NonMemoryInstr::new(NonMemoryOpc::NOP)),
            0x1B => Instr::Memory(MemoryInstr::new(
                MemoryOpc::SLO,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::AbsoluteY,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0x1C => Instr::Memory(MemoryInstr::new(
                MemoryOpc::NOP,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::AbsoluteX,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0x1D => Instr::Memory(MemoryInstr::new(
                MemoryOpc::ORA,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::AbsoluteX,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0x1E => Instr::Memory(MemoryInstr::new(
                MemoryOpc::ASL,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::AbsoluteX,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0x1F => Instr::Memory(MemoryInstr::new(
                MemoryOpc::SLO,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::AbsoluteX,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0x20 => Instr::Jump(JumpInstr::new(JumpOpc::JSR, JumpType::JumpToSubroutine)),
            0x21 => Instr::Memory(MemoryInstr::new(
                MemoryOpc::AND,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::IndirectX,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0x22 => Instr::Jam,
            0x23 => Instr::Memory(MemoryInstr::new(
                MemoryOpc::RLA,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::IndirectX,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0x24 => Instr::Memory(MemoryInstr::new(
                MemoryOpc::BIT,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPage,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0x25 => Instr::Memory(MemoryInstr::new(
                MemoryOpc::AND,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPage,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0x26 => Instr::Memory(MemoryInstr::new(
                MemoryOpc::ROL,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPage,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0x27 => Instr::Memory(MemoryInstr::new(
                MemoryOpc::RLA,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPage,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0x28 => Instr::Control(ControlInstr::new(ControlOpc::PLP)),
            0x29 => Instr::Memory(MemoryInstr::new(
                MemoryOpc::AND,
                AddressingConfig::Immediate,
            )),
            0x2A => Instr::NonMemory(NonMemoryInstr::new(NonMemoryOpc::ROL)),
            0x2B => Instr::Memory(MemoryInstr::new(
                MemoryOpc::ANC,
                AddressingConfig::Immediate,
            )),
            0x2C => Instr::Memory(MemoryInstr::new(
                MemoryOpc::BIT,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::Absolute,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0x2D => Instr::Memory(MemoryInstr::new(
                MemoryOpc::AND,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::Absolute,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0x2E => Instr::Memory(MemoryInstr::new(
                MemoryOpc::ROL,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::Absolute,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0x2F => Instr::Memory(MemoryInstr::new(
                MemoryOpc::RLA,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::Absolute,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0x30 => Instr::Branch(BranchInstr::new(BranchOpc::BMI)),
            0x31 => Instr::Memory(MemoryInstr::new(
                MemoryOpc::AND,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::IndirectY,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0x32 => Instr::Jam,
            0x33 => Instr::Memory(MemoryInstr::new(
                MemoryOpc::RLA,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::IndirectY,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0x34 => Instr::Memory(MemoryInstr::new(
                MemoryOpc::NOP,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPageX,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0x35 => Instr::Memory(MemoryInstr::new(
                MemoryOpc::AND,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPageX,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0x36 => Instr::Memory(MemoryInstr::new(
                MemoryOpc::ROL,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPageX,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0x37 => Instr::Memory(MemoryInstr::new(
                MemoryOpc::RLA,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPageX,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0x38 => Instr::NonMemory(NonMemoryInstr::new(NonMemoryOpc::SEC)),
            0x39 => Instr::Memory(MemoryInstr::new(
                MemoryOpc::AND,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::AbsoluteY,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0x3A => Instr::NonMemory(NonMemoryInstr::new(NonMemoryOpc::NOP)),
            0x3B => Instr::Memory(MemoryInstr::new(
                MemoryOpc::RLA,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::AbsoluteY,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0x3C => Instr::Memory(MemoryInstr::new(
                MemoryOpc::NOP,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::AbsoluteX,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0x3D => Instr::Memory(MemoryInstr::new(
                MemoryOpc::AND,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::AbsoluteX,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0x3E => Instr::Memory(MemoryInstr::new(
                MemoryOpc::ROL,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::AbsoluteX,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0x3F => Instr::Memory(MemoryInstr::new(
                MemoryOpc::RLA,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::AbsoluteX,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0x40 => Instr::Control(ControlInstr::new(ControlOpc::RTI)),
            0x41 => Instr::Memory(MemoryInstr::new(
                MemoryOpc::EOR,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::IndirectX,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0x42 => Instr::Jam,
            0x43 => Instr::Memory(MemoryInstr::new(
                MemoryOpc::SRE,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::IndirectX,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0x44 => Instr::Memory(MemoryInstr::new(
                MemoryOpc::NOP,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPage,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0x45 => Instr::Memory(MemoryInstr::new(
                MemoryOpc::EOR,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPage,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0x46 => Instr::Memory(MemoryInstr::new(
                MemoryOpc::LSR,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPage,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0x47 => Instr::Memory(MemoryInstr::new(
                MemoryOpc::SRE,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPage,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0x48 => Instr::Control(ControlInstr::new(ControlOpc::PHA)),
            0x49 => Instr::Memory(MemoryInstr::new(
                MemoryOpc::EOR,
                AddressingConfig::Immediate,
            )),
            0x4A => Instr::NonMemory(NonMemoryInstr::new(NonMemoryOpc::LSR)),
            0x4B => Instr::Memory(MemoryInstr::new(
                MemoryOpc::ASR,
                AddressingConfig::Immediate,
            )),
            0x4C => Instr::Jump(JumpInstr::new(JumpOpc::JMP, JumpType::JumpToAddr)),
            0x4D => Instr::Memory(MemoryInstr::new(
                MemoryOpc::EOR,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::Absolute,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0x4E => Instr::Memory(MemoryInstr::new(
                MemoryOpc::LSR,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::Absolute,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0x4F => Instr::Memory(MemoryInstr::new(
                MemoryOpc::SRE,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::Absolute,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0x50 => Instr::Branch(BranchInstr::new(BranchOpc::BVC)),
            0x51 => Instr::Memory(MemoryInstr::new(
                MemoryOpc::EOR,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::IndirectY,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0x52 => Instr::Jam,
            0x53 => Instr::Memory(MemoryInstr::new(
                MemoryOpc::SRE,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::IndirectY,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0x54 => Instr::Memory(MemoryInstr::new(
                MemoryOpc::NOP,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPageX,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0x55 => Instr::Memory(MemoryInstr::new(
                MemoryOpc::EOR,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPageX,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0x56 => Instr::Memory(MemoryInstr::new(
                MemoryOpc::LSR,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPageX,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0x57 => Instr::Memory(MemoryInstr::new(
                MemoryOpc::SRE,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPageX,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0x58 => Instr::NonMemory(NonMemoryInstr::new(NonMemoryOpc::CLI)),
            0x59 => Instr::Memory(MemoryInstr::new(
                MemoryOpc::EOR,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::AbsoluteY,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0x5A => Instr::NonMemory(NonMemoryInstr::new(NonMemoryOpc::NOP)),
            0x5B => Instr::Memory(MemoryInstr::new(
                MemoryOpc::SRE,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::AbsoluteY,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0x5C => Instr::Memory(MemoryInstr::new(
                MemoryOpc::NOP,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::AbsoluteX,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0x5D => Instr::Memory(MemoryInstr::new(
                MemoryOpc::EOR,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::AbsoluteX,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0x5E => Instr::Memory(MemoryInstr::new(
                MemoryOpc::LSR,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::AbsoluteX,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0x5F => Instr::Memory(MemoryInstr::new(
                MemoryOpc::SRE,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::AbsoluteX,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0x60 => Instr::Control(ControlInstr::new(ControlOpc::RTS)),
            0x61 => Instr::Memory(MemoryInstr::new(
                MemoryOpc::ADC,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::IndirectX,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0x62 => Instr::Jam,
            0x63 => Instr::Memory(MemoryInstr::new(
                MemoryOpc::RRA,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::IndirectX,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0x64 => Instr::Memory(MemoryInstr::new(
                MemoryOpc::NOP,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPage,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0x65 => Instr::Memory(MemoryInstr::new(
                MemoryOpc::ADC,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPage,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0x66 => Instr::Memory(MemoryInstr::new(
                MemoryOpc::ROR,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPage,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0x67 => Instr::Memory(MemoryInstr::new(
                MemoryOpc::RRA,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPage,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0x68 => Instr::Control(ControlInstr::new(ControlOpc::PLA)),
            0x69 => Instr::Memory(MemoryInstr::new(
                MemoryOpc::ADC,
                AddressingConfig::Immediate,
            )),
            0x6A => Instr::NonMemory(NonMemoryInstr::new(NonMemoryOpc::ROR)),
            0x6B => Instr::Memory(MemoryInstr::new(
                MemoryOpc::ARR,
                AddressingConfig::Immediate,
            )),
            0x6C => Instr::Jump(JumpInstr::new(JumpOpc::JMP, JumpType::JumpToPointerAddr)),
            0x6D => Instr::Memory(MemoryInstr::new(
                MemoryOpc::ADC,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::Absolute,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0x6E => Instr::Memory(MemoryInstr::new(
                MemoryOpc::ROR,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::Absolute,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0x6F => Instr::Memory(MemoryInstr::new(
                MemoryOpc::RRA,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::Absolute,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0x70 => Instr::Branch(BranchInstr::new(BranchOpc::BVS)),
            0x71 => Instr::Memory(MemoryInstr::new(
                MemoryOpc::ADC,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::IndirectY,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0x72 => Instr::Jam,
            0x73 => Instr::Memory(MemoryInstr::new(
                MemoryOpc::RRA,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::IndirectY,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0x74 => Instr::Memory(MemoryInstr::new(
                MemoryOpc::NOP,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPageX,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0x75 => Instr::Memory(MemoryInstr::new(
                MemoryOpc::ADC,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPageX,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0x76 => Instr::Memory(MemoryInstr::new(
                MemoryOpc::ROR,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPageX,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0x77 => Instr::Memory(MemoryInstr::new(
                MemoryOpc::RRA,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPageX,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0x78 => Instr::NonMemory(NonMemoryInstr::new(NonMemoryOpc::SEI)),
            0x79 => Instr::Memory(MemoryInstr::new(
                MemoryOpc::ADC,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::AbsoluteY,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0x7A => Instr::NonMemory(NonMemoryInstr::new(NonMemoryOpc::NOP)),
            0x7B => Instr::Memory(MemoryInstr::new(
                MemoryOpc::RRA,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::AbsoluteY,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0x7C => Instr::Memory(MemoryInstr::new(
                MemoryOpc::NOP,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::AbsoluteX,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0x7D => Instr::Memory(MemoryInstr::new(
                MemoryOpc::ADC,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::AbsoluteX,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0x7E => Instr::Memory(MemoryInstr::new(
                MemoryOpc::ROR,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::AbsoluteX,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0x7F => Instr::Memory(MemoryInstr::new(
                MemoryOpc::RRA,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::AbsoluteX,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0x80 => Instr::Memory(MemoryInstr::new(
                MemoryOpc::NOP,
                AddressingConfig::Immediate,
            )),
            0x81 => Instr::Memory(MemoryInstr::new(
                MemoryOpc::STA,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::IndirectX,
                    access_type: MemoryAccessType::Write,
                },
            )),
            0x82 => Instr::Memory(MemoryInstr::new(
                MemoryOpc::NOP,
                AddressingConfig::Immediate,
            )),
            0x83 => Instr::Memory(MemoryInstr::new(
                MemoryOpc::SAX,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::IndirectX,
                    access_type: MemoryAccessType::Write,
                },
            )),
            0x84 => Instr::Memory(MemoryInstr::new(
                MemoryOpc::STY,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPage,
                    access_type: MemoryAccessType::Write,
                },
            )),
            0x85 => Instr::Memory(MemoryInstr::new(
                MemoryOpc::STA,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPage,
                    access_type: MemoryAccessType::Write,
                },
            )),
            0x86 => Instr::Memory(MemoryInstr::new(
                MemoryOpc::STX,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPage,
                    access_type: MemoryAccessType::Write,
                },
            )),
            0x87 => Instr::Memory(MemoryInstr::new(
                MemoryOpc::SAX,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPage,
                    access_type: MemoryAccessType::Write,
                },
            )),
            0x88 => Instr::NonMemory(NonMemoryInstr::new(NonMemoryOpc::DEY)),
            0x89 => Instr::Memory(MemoryInstr::new(
                MemoryOpc::NOP,
                AddressingConfig::Immediate,
            )),
            0x8A => Instr::NonMemory(NonMemoryInstr::new(NonMemoryOpc::TXA)),
            0x8B => Instr::Memory(MemoryInstr::new(
                MemoryOpc::XAA,
                AddressingConfig::Immediate,
            )),
            0x8C => Instr::Memory(MemoryInstr::new(
                MemoryOpc::STY,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::Absolute,
                    access_type: MemoryAccessType::Write,
                },
            )),
            0x8D => Instr::Memory(MemoryInstr::new(
                MemoryOpc::STA,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::Absolute,
                    access_type: MemoryAccessType::Write,
                },
            )),
            0x8E => Instr::Memory(MemoryInstr::new(
                MemoryOpc::STX,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::Absolute,
                    access_type: MemoryAccessType::Write,
                },
            )),
            0x8F => Instr::Memory(MemoryInstr::new(
                MemoryOpc::SAX,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::Absolute,
                    access_type: MemoryAccessType::Write,
                },
            )),
            0x90 => Instr::Branch(BranchInstr::new(BranchOpc::BCC)),
            0x91 => Instr::Memory(MemoryInstr::new(
                MemoryOpc::STA,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::IndirectY,
                    access_type: MemoryAccessType::Write,
                },
            )),
            0x92 => Instr::Jam,
            0x93 => Instr::Memory(MemoryInstr::new(
                MemoryOpc::SHA,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::IndirectY,
                    access_type: MemoryAccessType::Write,
                },
            )),
            0x94 => Instr::Memory(MemoryInstr::new(
                MemoryOpc::STY,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPageX,
                    access_type: MemoryAccessType::Write,
                },
            )),
            0x95 => Instr::Memory(MemoryInstr::new(
                MemoryOpc::STA,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPageX,
                    access_type: MemoryAccessType::Write,
                },
            )),
            0x96 => Instr::Memory(MemoryInstr::new(
                MemoryOpc::STX,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPageY,
                    access_type: MemoryAccessType::Write,
                },
            )),
            0x97 => Instr::Memory(MemoryInstr::new(
                MemoryOpc::SAX,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPageY,
                    access_type: MemoryAccessType::Write,
                },
            )),
            0x98 => Instr::NonMemory(NonMemoryInstr::new(NonMemoryOpc::TYA)),
            0x99 => Instr::Memory(MemoryInstr::new(
                MemoryOpc::STA,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::AbsoluteY,
                    access_type: MemoryAccessType::Write,
                },
            )),
            0x9A => Instr::NonMemory(NonMemoryInstr::new(NonMemoryOpc::TXS)),
            0x9B => Instr::Memory(MemoryInstr::new(
                MemoryOpc::SHS,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::AbsoluteY,
                    access_type: MemoryAccessType::Write,
                },
            )),
            0x9C => Instr::Memory(MemoryInstr::new(
                MemoryOpc::SHY,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::AbsoluteX,
                    access_type: MemoryAccessType::Write,
                },
            )),
            0x9D => Instr::Memory(MemoryInstr::new(
                MemoryOpc::STA,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::AbsoluteX,
                    access_type: MemoryAccessType::Write,
                },
            )),
            0x9E => Instr::Memory(MemoryInstr::new(
                MemoryOpc::SHX,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::AbsoluteY,
                    access_type: MemoryAccessType::Write,
                },
            )),
            0x9F => Instr::Memory(MemoryInstr::new(
                MemoryOpc::SHA,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::AbsoluteY,
                    access_type: MemoryAccessType::Write,
                },
            )),
            0xA0 => Instr::Memory(MemoryInstr::new(
                MemoryOpc::LDY,
                AddressingConfig::Immediate,
            )),
            0xA1 => Instr::Memory(MemoryInstr::new(
                MemoryOpc::LDA,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::IndirectX,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0xA2 => Instr::Memory(MemoryInstr::new(
                MemoryOpc::LDX,
                AddressingConfig::Immediate,
            )),
            0xA3 => Instr::Memory(MemoryInstr::new(
                MemoryOpc::LAX,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::IndirectX,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0xA4 => Instr::Memory(MemoryInstr::new(
                MemoryOpc::LDY,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPage,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0xA5 => Instr::Memory(MemoryInstr::new(
                MemoryOpc::LDA,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPage,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0xA6 => Instr::Memory(MemoryInstr::new(
                MemoryOpc::LDX,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPage,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0xA7 => Instr::Memory(MemoryInstr::new(
                MemoryOpc::LAX,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPage,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0xA8 => Instr::NonMemory(NonMemoryInstr::new(NonMemoryOpc::TAY)),
            0xA9 => Instr::Memory(MemoryInstr::new(
                MemoryOpc::LDA,
                AddressingConfig::Immediate,
            )),
            0xAA => Instr::NonMemory(NonMemoryInstr::new(NonMemoryOpc::TAX)),
            0xAB => Instr::Memory(MemoryInstr::new(
                MemoryOpc::LAX,
                AddressingConfig::Immediate,
            )),
            0xAC => Instr::Memory(MemoryInstr::new(
                MemoryOpc::LDY,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::Absolute,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0xAD => Instr::Memory(MemoryInstr::new(
                MemoryOpc::LDA,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::Absolute,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0xAE => Instr::Memory(MemoryInstr::new(
                MemoryOpc::LDX,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::Absolute,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0xAF => Instr::Memory(MemoryInstr::new(
                MemoryOpc::LAX,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::Absolute,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0xB0 => Instr::Branch(BranchInstr::new(BranchOpc::BCS)),
            0xB1 => Instr::Memory(MemoryInstr::new(
                MemoryOpc::LDA,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::IndirectY,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0xB2 => Instr::Jam,
            0xB3 => Instr::Memory(MemoryInstr::new(
                MemoryOpc::LAX,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::IndirectY,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0xB4 => Instr::Memory(MemoryInstr::new(
                MemoryOpc::LDY,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPageX,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0xB5 => Instr::Memory(MemoryInstr::new(
                MemoryOpc::LDA,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPageX,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0xB6 => Instr::Memory(MemoryInstr::new(
                MemoryOpc::LDX,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPageY,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0xB7 => Instr::Memory(MemoryInstr::new(
                MemoryOpc::LAX,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPageY,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0xB8 => Instr::NonMemory(NonMemoryInstr::new(NonMemoryOpc::CLV)),
            0xB9 => Instr::Memory(MemoryInstr::new(
                MemoryOpc::LDA,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::AbsoluteY,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0xBA => Instr::NonMemory(NonMemoryInstr::new(NonMemoryOpc::TSX)),
            0xBB => Instr::Memory(MemoryInstr::new(
                MemoryOpc::LAS,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::AbsoluteY,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0xBC => Instr::Memory(MemoryInstr::new(
                MemoryOpc::LDY,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::AbsoluteX,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0xBD => Instr::Memory(MemoryInstr::new(
                MemoryOpc::LDA,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::AbsoluteX,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0xBE => Instr::Memory(MemoryInstr::new(
                MemoryOpc::LDX,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::AbsoluteY,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0xBF => Instr::Memory(MemoryInstr::new(
                MemoryOpc::LAX,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::AbsoluteY,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0xC0 => Instr::Memory(MemoryInstr::new(
                MemoryOpc::CPY,
                AddressingConfig::Immediate,
            )),
            0xC1 => Instr::Memory(MemoryInstr::new(
                MemoryOpc::CMP,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::IndirectX,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0xC2 => Instr::Memory(MemoryInstr::new(
                MemoryOpc::NOP,
                AddressingConfig::Immediate,
            )),
            0xC3 => Instr::Memory(MemoryInstr::new(
                MemoryOpc::DCP,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::IndirectX,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0xC4 => Instr::Memory(MemoryInstr::new(
                MemoryOpc::CPY,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPage,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0xC5 => Instr::Memory(MemoryInstr::new(
                MemoryOpc::CMP,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPage,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0xC6 => Instr::Memory(MemoryInstr::new(
                MemoryOpc::DEC,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPage,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0xC7 => Instr::Memory(MemoryInstr::new(
                MemoryOpc::DCP,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPage,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0xC8 => Instr::NonMemory(NonMemoryInstr::new(NonMemoryOpc::INY)),
            0xC9 => Instr::Memory(MemoryInstr::new(
                MemoryOpc::CMP,
                AddressingConfig::Immediate,
            )),
            0xCA => Instr::NonMemory(NonMemoryInstr::new(NonMemoryOpc::DEX)),
            0xCB => Instr::Memory(MemoryInstr::new(
                MemoryOpc::SBX,
                AddressingConfig::Immediate,
            )),
            0xCC => Instr::Memory(MemoryInstr::new(
                MemoryOpc::CPY,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::Absolute,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0xCD => Instr::Memory(MemoryInstr::new(
                MemoryOpc::CMP,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::Absolute,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0xCE => Instr::Memory(MemoryInstr::new(
                MemoryOpc::DEC,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::Absolute,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0xCF => Instr::Memory(MemoryInstr::new(
                MemoryOpc::DCP,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::Absolute,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0xD0 => Instr::Branch(BranchInstr::new(BranchOpc::BNE)),
            0xD1 => Instr::Memory(MemoryInstr::new(
                MemoryOpc::CMP,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::IndirectY,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0xD2 => Instr::Jam,
            0xD3 => Instr::Memory(MemoryInstr::new(
                MemoryOpc::DCP,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::IndirectY,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0xD4 => Instr::Memory(MemoryInstr::new(
                MemoryOpc::NOP,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPageX,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0xD5 => Instr::Memory(MemoryInstr::new(
                MemoryOpc::CMP,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPageX,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0xD6 => Instr::Memory(MemoryInstr::new(
                MemoryOpc::DEC,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPageX,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0xD7 => Instr::Memory(MemoryInstr::new(
                MemoryOpc::DCP,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPageX,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0xD8 => Instr::NonMemory(NonMemoryInstr::new(NonMemoryOpc::CLD)),
            0xD9 => Instr::Memory(MemoryInstr::new(
                MemoryOpc::CMP,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::AbsoluteY,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0xDA => Instr::NonMemory(NonMemoryInstr::new(NonMemoryOpc::NOP)),
            0xDB => Instr::Memory(MemoryInstr::new(
                MemoryOpc::DCP,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::AbsoluteY,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0xDC => Instr::Memory(MemoryInstr::new(
                MemoryOpc::NOP,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::AbsoluteX,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0xDD => Instr::Memory(MemoryInstr::new(
                MemoryOpc::CMP,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::AbsoluteX,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0xDE => Instr::Memory(MemoryInstr::new(
                MemoryOpc::DEC,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::AbsoluteX,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0xDF => Instr::Memory(MemoryInstr::new(
                MemoryOpc::DCP,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::AbsoluteX,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0xE0 => Instr::Memory(MemoryInstr::new(
                MemoryOpc::CPX,
                AddressingConfig::Immediate,
            )),
            0xE1 => Instr::Memory(MemoryInstr::new(
                MemoryOpc::SBC,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::IndirectX,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0xE2 => Instr::Memory(MemoryInstr::new(
                MemoryOpc::NOP,
                AddressingConfig::Immediate,
            )),
            0xE3 => Instr::Memory(MemoryInstr::new(
                MemoryOpc::ISB,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::IndirectX,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0xE4 => Instr::Memory(MemoryInstr::new(
                MemoryOpc::CPX,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPage,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0xE5 => Instr::Memory(MemoryInstr::new(
                MemoryOpc::SBC,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPage,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0xE6 => Instr::Memory(MemoryInstr::new(
                MemoryOpc::INC,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPage,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0xE7 => Instr::Memory(MemoryInstr::new(
                MemoryOpc::ISB,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPage,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0xE8 => Instr::NonMemory(NonMemoryInstr::new(NonMemoryOpc::INX)),
            0xE9 => Instr::Memory(MemoryInstr::new(
                MemoryOpc::SBC,
                AddressingConfig::Immediate,
            )),
            0xEA => Instr::NonMemory(NonMemoryInstr::new(NonMemoryOpc::NOP)),
            0xEB => Instr::Memory(MemoryInstr::new(
                MemoryOpc::SBC,
                AddressingConfig::Immediate,
            )),
            0xEC => Instr::Memory(MemoryInstr::new(
                MemoryOpc::CPX,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::Absolute,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0xED => Instr::Memory(MemoryInstr::new(
                MemoryOpc::SBC,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::Absolute,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0xEE => Instr::Memory(MemoryInstr::new(
                MemoryOpc::INC,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::Absolute,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0xEF => Instr::Memory(MemoryInstr::new(
                MemoryOpc::ISB,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::Absolute,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0xF0 => Instr::Branch(BranchInstr::new(BranchOpc::BEQ)),
            0xF1 => Instr::Memory(MemoryInstr::new(
                MemoryOpc::SBC,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::IndirectY,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0xF2 => Instr::Jam,
            0xF3 => Instr::Memory(MemoryInstr::new(
                MemoryOpc::ISB,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::IndirectY,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0xF4 => Instr::Memory(MemoryInstr::new(
                MemoryOpc::NOP,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPageX,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0xF5 => Instr::Memory(MemoryInstr::new(
                MemoryOpc::SBC,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPageX,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0xF6 => Instr::Memory(MemoryInstr::new(
                MemoryOpc::INC,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPageX,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0xF7 => Instr::Memory(MemoryInstr::new(
                MemoryOpc::ISB,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPageX,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0xF8 => Instr::NonMemory(NonMemoryInstr::new(NonMemoryOpc::SED)),
            0xF9 => Instr::Memory(MemoryInstr::new(
                MemoryOpc::SBC,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::AbsoluteY,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0xFA => Instr::NonMemory(NonMemoryInstr::new(NonMemoryOpc::NOP)),
            0xFB => Instr::Memory(MemoryInstr::new(
                MemoryOpc::ISB,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::AbsoluteY,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0xFC => Instr::Memory(MemoryInstr::new(
                MemoryOpc::NOP,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::AbsoluteX,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0xFD => Instr::Memory(MemoryInstr::new(
                MemoryOpc::SBC,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::AbsoluteX,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0xFE => Instr::Memory(MemoryInstr::new(
                MemoryOpc::INC,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::AbsoluteX,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0xFF => Instr::Memory(MemoryInstr::new(
                MemoryOpc::ISB,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::AbsoluteX,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
        }
    }
}
