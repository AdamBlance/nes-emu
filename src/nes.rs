use crate::cpu::Cpu;
use crate::ppu::Ppu;
use crate::apu::Apu;
use crate::cartridge::Cartridge;

pub struct Nes {
    // Hardware
    pub cpu:         Cpu,
    pub ppu:         Ppu,
    pub apu:         Apu,
    pub wram:        [u8; 2048],
    pub cartridge:   Cartridge,
    pub controller1: Controller,
    pub controller2: Controller,
    // External
    pub frame:        Vec<u8>,
    // Debugging
    pub ppu_log_toggle: bool,
    pub old_cpu_state:  Cpu,
    pub old_ppu_state:  Ppu,
}

impl Nes {
    pub fn new(cartridge: Cartridge, audio_queue: Sender<f32>) -> Nes {
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

            ppu_log_toggle: false,
            old_cpu_state: Cpu::new(),
            old_ppu_state: Ppu::new(),
        }
    }
}