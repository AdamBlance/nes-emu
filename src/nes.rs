use crate::cpu::Cpu;
use crate::ppu::Ppu;
use crate::apu::Apu;
use crate::controller::Controller;
use crate::cartridge::Cartridge;
use std::sync::mpsc::Sender;

pub struct Nes {
    // Hardware
    pub cpu:         Cpu,
    pub ppu:         Ppu,
    pub apu:         Apu,
    pub wram:        [u8; 2048],
    pub cartridge:   Box<dyn Cartridge>,
    pub controller1: Controller,
    pub controller2: Controller,
    // External
    pub frame:        Vec<u8>,
}

impl Nes {
    pub fn new(cartridge: Box<dyn Cartridge>, audio_queue: Sender<(f32, f32)>) -> Nes {
        Nes { 
            cpu: Cpu::new(),
            ppu: Ppu::new(),
            apu: Apu::new(audio_queue),
            wram: [0; 2048],
            cartridge,
            controller1: Default::default(),
            controller2: Default::default(),

            // RGBA image (4 channels)
            frame: vec![0u8; 256usize * 240 * 4], 
        }
    }
}