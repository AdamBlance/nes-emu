pub mod apu;
pub mod cartridge;
pub mod controller;
pub mod cpu;
mod mem;
pub mod ppu;
pub mod mem_consts;

use crate::nes::apu::Apu;
use crate::nes::cartridge::Cartridge;
use crate::nes::controller::Controller;
use crate::nes::cpu::Cpu;
use crate::nes::ppu::Ppu;
use crate::util::concat_u8;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Serialize, Deserialize)]
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
    // #[serde(skip)]
    // #[serde(default = "frame_default")]
    pub frame: Option<Rc<RefCell<Vec<u8>>>>,
}

impl Clone for Nes {
    fn clone(&self) -> Self {
        Nes {
            cpu: self.cpu,
            ppu: self.ppu.clone(),
            apu: self.apu.clone(),
            wram: self.wram.clone(),
            cart: dyn_clone::clone_box(&*self.cart),
            con1: self.con1,
            con2: self.con2,
            frame: Some(Rc::clone(self.frame.as_ref().unwrap())),
        }
    }
}

impl Nes {
    pub fn new(cartridge: Box<dyn Cartridge>, frame: Rc<RefCell<Vec<u8>>>) -> Nes {
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
            frame: Some(frame),
        }
    }
}
