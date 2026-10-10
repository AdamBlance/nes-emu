mod addressing;
mod consts;
mod main_memory;
mod memory_mapped_registers;
mod ppu_memory;

use crate::emulator::nes::apu::Apu;
use crate::emulator::nes::cartridge::Cartridge;
use crate::emulator::nes::cpu::Cpu;
use crate::emulator::nes::ppu::Ppu;

struct NesState {
    pub cpu: Cpu,
    pub ppu: Ppu,
    pub apu: Apu,
    pub wram: Vec<u8>,
    pub cart: Box<dyn Cartridge>,
    pub con1: Controller,
    pub con2: Controller,
}
