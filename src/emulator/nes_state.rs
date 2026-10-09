mod addressing;
mod mem;

use crate::emulator::nes::apu::Apu;
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
