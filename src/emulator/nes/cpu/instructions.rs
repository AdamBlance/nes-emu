use self::branch::{BranchInstr, BranchOpc};
use self::control::{ControlInstr, ControlOpc};
use self::interrupts::{Interrupt, InterruptType};
use self::jump::{JumpInstr, JumpOpc, JumpType};
use self::memory::{AddressingConfig, AddressingMode, MemoryAccessType, MemoryInstr, MemoryOpc};
use self::nonmemory::{NonMemoryInstr, NonMemoryOpc};
use crate::emulator::nes::Nes;
use serde::{Deserialize, Serialize};

mod branch;
mod common;
mod control;
pub mod interrupts;
mod jump;
mod memory;
mod nonmemory;

trait ControlThingy {
    fn next_cycle(&mut self, nes: &mut Nes) -> bool;
    fn name(&self) -> &str;
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum ControlSequence {
    Branch(BranchInstr),
    Control(ControlInstr),
    Jump(JumpInstr),
    Memory(MemoryInstr),
    NonMemory(NonMemoryInstr),
    Jam,
    // For the sake of simplicity, an interrupt is considered an instruction
    Interrupt(Interrupt),
}

impl Default for ControlSequence {
    fn default() -> Self {
        Self::NonMemory(Default::default())
    }
}
impl ControlSequence {
    pub const DUMMY_INSTR: ControlSequence = Self::NonMemory(NonMemoryInstr::DUMMY_INSTR);
    // TODO: Figure out how to have the do_next_cycle method in here!
    pub fn is_finished(&self) -> bool {
        match self {
            Self::Branch(instr) => instr.is_finished(),
            Self::Control(instr) => instr.is_finished(),
            Self::Jump(instr) => instr.is_finished(),
            Self::Memory(instr) => instr.is_finished(),
            Self::NonMemory(instr) => instr.is_finished(),
            Self::Interrupt(interrupt) => interrupt.is_finished(),
            Self::Jam => true,
        }
    }
    pub fn new_interrupt(interrupt_type: InterruptType) -> Self {
        Self::Interrupt(Interrupt::new(interrupt_type))
    }
    pub fn do_next_cycle(&mut self, nes: &mut Nes) {
        // println!("instr: {:?}", self);
        match self {
            ControlSequence::Branch(instr) => instr.do_next_instruction_cycle(nes),
            ControlSequence::Control(instr) => instr.do_next_instruction_cycle(nes),
            ControlSequence::Jump(instr) => instr.do_next_instruction_cycle(nes),
            ControlSequence::Memory(instr) => instr.do_next_instruction_cycle(nes),
            ControlSequence::NonMemory(instr) => instr.do_next_instruction_cycle(nes),
            ControlSequence::Interrupt(interrupt) => interrupt.do_next_interrupt_cycle(nes),
            ControlSequence::Jam => panic!("JAM!"),
        };
    }
    pub fn from_opcode(opcode: u8) -> Self {
        match opcode {
            0x00 => ControlSequence::Control(ControlInstr::new(ControlOpc::BRK)),
            0x01 => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::ORA,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::IndirectX,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0x02 => ControlSequence::Jam,
            0x03 => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::SLO,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::IndirectX,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0x04 => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::NOP,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPage,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0x05 => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::ORA,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPage,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0x06 => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::ASL,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPage,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0x07 => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::SLO,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPage,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0x08 => ControlSequence::Control(ControlInstr::new(ControlOpc::PHP)),
            0x09 => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::ORA,
                AddressingConfig::Immediate,
            )),
            0x0A => ControlSequence::NonMemory(NonMemoryInstr::new(NonMemoryOpc::ASL)),
            0x0B => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::ANC,
                AddressingConfig::Immediate,
            )),
            0x0C => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::NOP,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::Absolute,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0x0D => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::ORA,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::Absolute,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0x0E => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::ASL,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::Absolute,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0x0F => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::SLO,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::Absolute,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0x10 => ControlSequence::Branch(BranchInstr::new(BranchOpc::BPL)),
            0x11 => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::ORA,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::IndirectY,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0x12 => ControlSequence::Jam,
            0x13 => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::SLO,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::IndirectY,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0x14 => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::NOP,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPageX,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0x15 => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::ORA,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPageX,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0x16 => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::ASL,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPageX,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0x17 => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::SLO,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPageX,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0x18 => ControlSequence::NonMemory(NonMemoryInstr::new(NonMemoryOpc::CLC)),
            0x19 => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::ORA,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::AbsoluteY,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0x1A => ControlSequence::NonMemory(NonMemoryInstr::new(NonMemoryOpc::NOP)),
            0x1B => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::SLO,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::AbsoluteY,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0x1C => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::NOP,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::AbsoluteX,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0x1D => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::ORA,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::AbsoluteX,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0x1E => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::ASL,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::AbsoluteX,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0x1F => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::SLO,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::AbsoluteX,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0x20 => ControlSequence::Jump(JumpInstr::new(JumpOpc::JSR, JumpType::JumpToSubroutine)),
            0x21 => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::AND,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::IndirectX,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0x22 => ControlSequence::Jam,
            0x23 => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::RLA,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::IndirectX,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0x24 => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::BIT,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPage,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0x25 => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::AND,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPage,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0x26 => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::ROL,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPage,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0x27 => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::RLA,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPage,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0x28 => ControlSequence::Control(ControlInstr::new(ControlOpc::PLP)),
            0x29 => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::AND,
                AddressingConfig::Immediate,
            )),
            0x2A => ControlSequence::NonMemory(NonMemoryInstr::new(NonMemoryOpc::ROL)),
            0x2B => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::ANC,
                AddressingConfig::Immediate,
            )),
            0x2C => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::BIT,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::Absolute,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0x2D => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::AND,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::Absolute,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0x2E => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::ROL,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::Absolute,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0x2F => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::RLA,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::Absolute,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0x30 => ControlSequence::Branch(BranchInstr::new(BranchOpc::BMI)),
            0x31 => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::AND,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::IndirectY,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0x32 => ControlSequence::Jam,
            0x33 => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::RLA,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::IndirectY,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0x34 => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::NOP,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPageX,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0x35 => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::AND,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPageX,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0x36 => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::ROL,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPageX,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0x37 => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::RLA,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPageX,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0x38 => ControlSequence::NonMemory(NonMemoryInstr::new(NonMemoryOpc::SEC)),
            0x39 => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::AND,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::AbsoluteY,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0x3A => ControlSequence::NonMemory(NonMemoryInstr::new(NonMemoryOpc::NOP)),
            0x3B => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::RLA,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::AbsoluteY,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0x3C => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::NOP,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::AbsoluteX,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0x3D => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::AND,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::AbsoluteX,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0x3E => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::ROL,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::AbsoluteX,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0x3F => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::RLA,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::AbsoluteX,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0x40 => ControlSequence::Control(ControlInstr::new(ControlOpc::RTI)),
            0x41 => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::EOR,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::IndirectX,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0x42 => ControlSequence::Jam,
            0x43 => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::SRE,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::IndirectX,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0x44 => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::NOP,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPage,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0x45 => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::EOR,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPage,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0x46 => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::LSR,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPage,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0x47 => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::SRE,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPage,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0x48 => ControlSequence::Control(ControlInstr::new(ControlOpc::PHA)),
            0x49 => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::EOR,
                AddressingConfig::Immediate,
            )),
            0x4A => ControlSequence::NonMemory(NonMemoryInstr::new(NonMemoryOpc::LSR)),
            0x4B => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::ASR,
                AddressingConfig::Immediate,
            )),
            0x4C => ControlSequence::Jump(JumpInstr::new(JumpOpc::JMP, JumpType::JumpToAddr)),
            0x4D => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::EOR,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::Absolute,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0x4E => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::LSR,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::Absolute,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0x4F => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::SRE,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::Absolute,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0x50 => ControlSequence::Branch(BranchInstr::new(BranchOpc::BVC)),
            0x51 => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::EOR,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::IndirectY,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0x52 => ControlSequence::Jam,
            0x53 => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::SRE,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::IndirectY,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0x54 => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::NOP,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPageX,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0x55 => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::EOR,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPageX,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0x56 => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::LSR,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPageX,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0x57 => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::SRE,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPageX,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0x58 => ControlSequence::NonMemory(NonMemoryInstr::new(NonMemoryOpc::CLI)),
            0x59 => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::EOR,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::AbsoluteY,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0x5A => ControlSequence::NonMemory(NonMemoryInstr::new(NonMemoryOpc::NOP)),
            0x5B => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::SRE,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::AbsoluteY,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0x5C => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::NOP,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::AbsoluteX,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0x5D => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::EOR,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::AbsoluteX,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0x5E => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::LSR,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::AbsoluteX,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0x5F => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::SRE,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::AbsoluteX,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0x60 => ControlSequence::Control(ControlInstr::new(ControlOpc::RTS)),
            0x61 => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::ADC,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::IndirectX,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0x62 => ControlSequence::Jam,
            0x63 => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::RRA,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::IndirectX,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0x64 => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::NOP,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPage,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0x65 => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::ADC,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPage,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0x66 => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::ROR,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPage,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0x67 => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::RRA,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPage,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0x68 => ControlSequence::Control(ControlInstr::new(ControlOpc::PLA)),
            0x69 => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::ADC,
                AddressingConfig::Immediate,
            )),
            0x6A => ControlSequence::NonMemory(NonMemoryInstr::new(NonMemoryOpc::ROR)),
            0x6B => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::ARR,
                AddressingConfig::Immediate,
            )),
            0x6C => {
                ControlSequence::Jump(JumpInstr::new(JumpOpc::JMP, JumpType::JumpToPointerAddr))
            }
            0x6D => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::ADC,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::Absolute,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0x6E => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::ROR,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::Absolute,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0x6F => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::RRA,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::Absolute,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0x70 => ControlSequence::Branch(BranchInstr::new(BranchOpc::BVS)),
            0x71 => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::ADC,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::IndirectY,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0x72 => ControlSequence::Jam,
            0x73 => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::RRA,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::IndirectY,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0x74 => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::NOP,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPageX,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0x75 => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::ADC,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPageX,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0x76 => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::ROR,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPageX,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0x77 => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::RRA,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPageX,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0x78 => ControlSequence::NonMemory(NonMemoryInstr::new(NonMemoryOpc::SEI)),
            0x79 => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::ADC,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::AbsoluteY,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0x7A => ControlSequence::NonMemory(NonMemoryInstr::new(NonMemoryOpc::NOP)),
            0x7B => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::RRA,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::AbsoluteY,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0x7C => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::NOP,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::AbsoluteX,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0x7D => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::ADC,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::AbsoluteX,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0x7E => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::ROR,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::AbsoluteX,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0x7F => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::RRA,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::AbsoluteX,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0x80 => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::NOP,
                AddressingConfig::Immediate,
            )),
            0x81 => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::STA,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::IndirectX,
                    access_type: MemoryAccessType::Write,
                },
            )),
            0x82 => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::NOP,
                AddressingConfig::Immediate,
            )),
            0x83 => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::SAX,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::IndirectX,
                    access_type: MemoryAccessType::Write,
                },
            )),
            0x84 => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::STY,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPage,
                    access_type: MemoryAccessType::Write,
                },
            )),
            0x85 => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::STA,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPage,
                    access_type: MemoryAccessType::Write,
                },
            )),
            0x86 => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::STX,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPage,
                    access_type: MemoryAccessType::Write,
                },
            )),
            0x87 => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::SAX,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPage,
                    access_type: MemoryAccessType::Write,
                },
            )),
            0x88 => ControlSequence::NonMemory(NonMemoryInstr::new(NonMemoryOpc::DEY)),
            0x89 => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::NOP,
                AddressingConfig::Immediate,
            )),
            0x8A => ControlSequence::NonMemory(NonMemoryInstr::new(NonMemoryOpc::TXA)),
            0x8B => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::XAA,
                AddressingConfig::Immediate,
            )),
            0x8C => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::STY,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::Absolute,
                    access_type: MemoryAccessType::Write,
                },
            )),
            0x8D => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::STA,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::Absolute,
                    access_type: MemoryAccessType::Write,
                },
            )),
            0x8E => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::STX,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::Absolute,
                    access_type: MemoryAccessType::Write,
                },
            )),
            0x8F => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::SAX,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::Absolute,
                    access_type: MemoryAccessType::Write,
                },
            )),
            0x90 => ControlSequence::Branch(BranchInstr::new(BranchOpc::BCC)),
            0x91 => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::STA,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::IndirectY,
                    access_type: MemoryAccessType::Write,
                },
            )),
            0x92 => ControlSequence::Jam,
            0x93 => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::SHA,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::IndirectY,
                    access_type: MemoryAccessType::Write,
                },
            )),
            0x94 => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::STY,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPageX,
                    access_type: MemoryAccessType::Write,
                },
            )),
            0x95 => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::STA,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPageX,
                    access_type: MemoryAccessType::Write,
                },
            )),
            0x96 => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::STX,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPageY,
                    access_type: MemoryAccessType::Write,
                },
            )),
            0x97 => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::SAX,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPageY,
                    access_type: MemoryAccessType::Write,
                },
            )),
            0x98 => ControlSequence::NonMemory(NonMemoryInstr::new(NonMemoryOpc::TYA)),
            0x99 => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::STA,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::AbsoluteY,
                    access_type: MemoryAccessType::Write,
                },
            )),
            0x9A => ControlSequence::NonMemory(NonMemoryInstr::new(NonMemoryOpc::TXS)),
            0x9B => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::SHS,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::AbsoluteY,
                    access_type: MemoryAccessType::Write,
                },
            )),
            0x9C => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::SHY,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::AbsoluteX,
                    access_type: MemoryAccessType::Write,
                },
            )),
            0x9D => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::STA,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::AbsoluteX,
                    access_type: MemoryAccessType::Write,
                },
            )),
            0x9E => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::SHX,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::AbsoluteY,
                    access_type: MemoryAccessType::Write,
                },
            )),
            0x9F => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::SHA,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::AbsoluteY,
                    access_type: MemoryAccessType::Write,
                },
            )),
            0xA0 => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::LDY,
                AddressingConfig::Immediate,
            )),
            0xA1 => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::LDA,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::IndirectX,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0xA2 => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::LDX,
                AddressingConfig::Immediate,
            )),
            0xA3 => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::LAX,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::IndirectX,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0xA4 => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::LDY,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPage,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0xA5 => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::LDA,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPage,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0xA6 => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::LDX,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPage,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0xA7 => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::LAX,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPage,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0xA8 => ControlSequence::NonMemory(NonMemoryInstr::new(NonMemoryOpc::TAY)),
            0xA9 => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::LDA,
                AddressingConfig::Immediate,
            )),
            0xAA => ControlSequence::NonMemory(NonMemoryInstr::new(NonMemoryOpc::TAX)),
            0xAB => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::LAX,
                AddressingConfig::Immediate,
            )),
            0xAC => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::LDY,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::Absolute,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0xAD => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::LDA,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::Absolute,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0xAE => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::LDX,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::Absolute,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0xAF => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::LAX,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::Absolute,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0xB0 => ControlSequence::Branch(BranchInstr::new(BranchOpc::BCS)),
            0xB1 => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::LDA,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::IndirectY,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0xB2 => ControlSequence::Jam,
            0xB3 => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::LAX,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::IndirectY,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0xB4 => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::LDY,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPageX,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0xB5 => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::LDA,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPageX,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0xB6 => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::LDX,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPageY,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0xB7 => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::LAX,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPageY,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0xB8 => ControlSequence::NonMemory(NonMemoryInstr::new(NonMemoryOpc::CLV)),
            0xB9 => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::LDA,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::AbsoluteY,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0xBA => ControlSequence::NonMemory(NonMemoryInstr::new(NonMemoryOpc::TSX)),
            0xBB => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::LAS,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::AbsoluteY,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0xBC => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::LDY,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::AbsoluteX,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0xBD => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::LDA,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::AbsoluteX,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0xBE => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::LDX,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::AbsoluteY,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0xBF => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::LAX,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::AbsoluteY,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0xC0 => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::CPY,
                AddressingConfig::Immediate,
            )),
            0xC1 => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::CMP,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::IndirectX,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0xC2 => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::NOP,
                AddressingConfig::Immediate,
            )),
            0xC3 => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::DCP,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::IndirectX,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0xC4 => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::CPY,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPage,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0xC5 => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::CMP,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPage,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0xC6 => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::DEC,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPage,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0xC7 => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::DCP,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPage,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0xC8 => ControlSequence::NonMemory(NonMemoryInstr::new(NonMemoryOpc::INY)),
            0xC9 => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::CMP,
                AddressingConfig::Immediate,
            )),
            0xCA => ControlSequence::NonMemory(NonMemoryInstr::new(NonMemoryOpc::DEX)),
            0xCB => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::SBX,
                AddressingConfig::Immediate,
            )),
            0xCC => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::CPY,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::Absolute,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0xCD => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::CMP,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::Absolute,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0xCE => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::DEC,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::Absolute,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0xCF => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::DCP,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::Absolute,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0xD0 => ControlSequence::Branch(BranchInstr::new(BranchOpc::BNE)),
            0xD1 => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::CMP,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::IndirectY,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0xD2 => ControlSequence::Jam,
            0xD3 => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::DCP,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::IndirectY,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0xD4 => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::NOP,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPageX,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0xD5 => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::CMP,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPageX,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0xD6 => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::DEC,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPageX,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0xD7 => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::DCP,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPageX,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0xD8 => ControlSequence::NonMemory(NonMemoryInstr::new(NonMemoryOpc::CLD)),
            0xD9 => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::CMP,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::AbsoluteY,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0xDA => ControlSequence::NonMemory(NonMemoryInstr::new(NonMemoryOpc::NOP)),
            0xDB => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::DCP,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::AbsoluteY,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0xDC => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::NOP,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::AbsoluteX,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0xDD => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::CMP,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::AbsoluteX,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0xDE => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::DEC,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::AbsoluteX,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0xDF => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::DCP,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::AbsoluteX,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0xE0 => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::CPX,
                AddressingConfig::Immediate,
            )),
            0xE1 => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::SBC,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::IndirectX,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0xE2 => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::NOP,
                AddressingConfig::Immediate,
            )),
            0xE3 => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::ISB,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::IndirectX,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0xE4 => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::CPX,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPage,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0xE5 => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::SBC,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPage,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0xE6 => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::INC,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPage,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0xE7 => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::ISB,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPage,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0xE8 => ControlSequence::NonMemory(NonMemoryInstr::new(NonMemoryOpc::INX)),
            0xE9 => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::SBC,
                AddressingConfig::Immediate,
            )),
            0xEA => ControlSequence::NonMemory(NonMemoryInstr::new(NonMemoryOpc::NOP)),
            0xEB => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::SBC,
                AddressingConfig::Immediate,
            )),
            0xEC => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::CPX,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::Absolute,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0xED => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::SBC,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::Absolute,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0xEE => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::INC,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::Absolute,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0xEF => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::ISB,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::Absolute,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0xF0 => ControlSequence::Branch(BranchInstr::new(BranchOpc::BEQ)),
            0xF1 => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::SBC,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::IndirectY,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0xF2 => ControlSequence::Jam,
            0xF3 => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::ISB,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::IndirectY,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0xF4 => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::NOP,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPageX,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0xF5 => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::SBC,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPageX,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0xF6 => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::INC,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPageX,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0xF7 => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::ISB,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::ZeroPageX,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0xF8 => ControlSequence::NonMemory(NonMemoryInstr::new(NonMemoryOpc::SED)),
            0xF9 => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::SBC,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::AbsoluteY,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0xFA => ControlSequence::NonMemory(NonMemoryInstr::new(NonMemoryOpc::NOP)),
            0xFB => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::ISB,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::AbsoluteY,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0xFC => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::NOP,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::AbsoluteX,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0xFD => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::SBC,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::AbsoluteX,
                    access_type: MemoryAccessType::Read,
                },
            )),
            0xFE => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::INC,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::AbsoluteX,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
            0xFF => ControlSequence::Memory(MemoryInstr::new(
                MemoryOpc::ISB,
                AddressingConfig::Addressed {
                    addr_mode: AddressingMode::AbsoluteX,
                    access_type: MemoryAccessType::ReadModifyWrite,
                },
            )),
        }
    }
}
