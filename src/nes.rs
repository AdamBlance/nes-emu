pub(crate) mod cpu;
pub(crate) mod ppu;
pub mod cartridge;
pub mod apu;
pub mod controller;
mod mem;

use crate::nes::cpu::Cpu;
use crate::nes::ppu::Ppu;
use crate::nes::apu::Apu;
use crate::nes::controller::Controller;
use crate::nes::cartridge::Cartridge;
use std::fs::File;
use std::sync::mpsc::Sender;
use crate::util::concat_u8;

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
    // pub logfile: File
}

impl Nes {
    pub fn new(cartridge: Box<dyn Cartridge>) -> Nes {
        Nes {
            cpu: Cpu::new(concat_u8(
                cartridge.read_prg_rom(0xFFFD),
                cartridge.read_prg_rom(0xFFFC)
            )),
            ppu: Ppu::new(),
            apu: Apu::new(),
            wram: [0; 2048],
            cart: cartridge,
            con1: Default::default(),
            con2: Default::default(),

            // RGBA image (4 channels)
            frame: vec![0u8; 256usize * 240 * 4],
        }
    }

    pub fn run_one_ppu_cycle_in_screen_bounds(&mut self) {
        loop {
            self.run_one_ppu_cycle();
            if (0..=239).contains(&self.ppu.scanline) && (1..=256).contains(&self.ppu.scanline_cycle) {break;}
        }
    }

    pub fn run_one_ppu_cycle(&mut self) {
        match (self.ppu.cycles) % 3 {
            0 => {
                cpu::step_cpu(self);
                ppu::step_ppu(self);
            }
            1 => ppu::step_ppu(self),
            2 => {
                ppu::step_ppu(self);
                apu::step_apu(self);
            }
            _ => unreachable!()
        }
    }

    pub fn run_to_vblank(&mut self) {

        loop {
            cpu::step_cpu(self);

            ppu::step_ppu(self);
            ppu::step_ppu(self);
            ppu::step_ppu(self);

            apu::step_apu(self);

            // At cycle mod 40
            // if self.apu.cycles_since_last_sample == self.apu.target_cycles_per_sample.floor() as u64 {
            //     // If the number of cycles between samples is too large on average,
            //     // sample on the 40th cycle
            //     if self.apu.average_cycles_per_sample >= self.apu.target_cycles_per_sample {
            //         do_sample(self);
            //     }
            // }
            // else if self.apu.cycles_since_last_sample == self.apu.target_cycles_per_sample.ceil() as u64 {
            //     if self.apu.average_cycles_per_sample < self.apu.target_cycles_per_sample {
            //         do_sample(self);
            //     }
            // }

            // So, after 3 ppu cycles, when reaching end of frame, ppu should land somewhere inside
            // the 3 cycle range after the frame ends
            // After 3 more ppu cycles, it should leave this range
            // This way, we don't need a bit to say that we've just entered vblank or whatever
            // and we shouldn't skip any frames

            // self.apu.cycles_since_last_sample += 1;

            if self.ppu.scanline == 239 && (self.ppu.scanline_cycle >= 257 && self.ppu.scanline_cycle <= 259) {break;}

            // if nes.cpu.instruction_count == target {break;}
        }

    }
}
