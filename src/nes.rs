pub mod apu;
pub mod cartridge;
pub mod controller;
pub mod cpu;
mod mem;
pub mod ppu;

use crate::nes::apu::Apu;
use crate::nes::cartridge::Cartridge;
use crate::nes::controller::Controller;
use crate::nes::cpu::Cpu;
use crate::nes::ppu::Ppu;
use crate::util::concat_u8;

pub struct Nes {
    // Hardware
    pub cpu: Cpu,
    pub ppu: Ppu,
    pub apu: Apu,
    pub wram: Vec<u8>,
    pub cart: Box<dyn Cartridge>,
    pub con1: Controller,
    pub con2: Controller,
    // External
    pub frame: Vec<u8>,
}

impl Nes {
    pub fn new(cartridge: Box<dyn Cartridge>) -> Nes {
        Nes {
            cpu: Cpu::new(concat_u8(
                cartridge.read_prg_rom(0xFFFD),
                cartridge.read_prg_rom(0xFFFC),
            )),
            ppu: Ppu::new(),
            apu: Apu::new(),
            wram: vec![0; 2048],
            cart: cartridge,
            con1: Default::default(),
            con2: Default::default(),

            // RGBA image (4 channels)
            frame: vec![0u8; 256usize * 240 * 4],
        }
    }
}
