use crate::cpu::Cpu;
use crate::ppu::Ppu;
use crate::apu::Apu;
use crate::controller::Controller;
use crate::cartridge::Cartridge;
use std::fs::File;
use std::sync::mpsc::Sender;

pub struct Nes {
    // Hardware
    pub cpu:         Cpu,
    pub ppu:         Ppu,
    pub apu:         Apu,
    pub wram:        [u8; 2048],
    pub cart:   Box<dyn Cartridge>,
    pub con1: Controller,
    pub con2: Controller,
    // External
    pub frame:        Vec<u8>,
    pub logfile: File
}

impl Nes {
    pub fn new(cartridge: Box<dyn Cartridge>, audio_queue: Sender<(f32, f32)>, sample_rate: u32, logfile: File) -> Nes {
        Nes { 
            cpu: Cpu::new(),
            ppu: Ppu::new(),
            apu: Apu::new(audio_queue, sample_rate),
            wram: [0; 2048],
            cart: cartridge,
            con1: Default::default(),
            con2: Default::default(),

            // RGBA image (4 channels)
            frame: vec![0u8; 256usize * 240 * 4], 
            logfile
        }
    }
}