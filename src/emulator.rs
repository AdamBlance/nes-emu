use std::cell::RefCell;
use std::cmp::{max, min};
use std::rc::Rc;
use crate::nes::cartridge::cartridge::new_cartridge;
use crate::nes::cartridge::Mirroring;
use crate::nes::controller::ButtonState;
use crate::nes::Nes;
use eframe::egui::{ColorImage, TextureFilter, TextureHandle, TextureOptions};
use std::sync::mpsc::SyncSender;

use crate::nes::apu;
use crate::nes::cpu;
use crate::nes::cpu::lookup_table::{Instruction, INSTRUCTIONS};
use crate::nes::ppu;
use crate::util::clamp;

use dyn_clone;

pub struct AudioStream {
    pub sender: SyncSender<(f32, f32)>,
    pub sample_rate: f32,
}

pub struct InstructionAndOperands {
    pub address: u16,
    pub instruction: Instruction,
    pub operand_1: Option<u8>,
    pub operand_2: Option<u8>,
}

pub struct Emulator {
    // The emulator isn't gonna have a NES unless it has a game cartridge
    // The cartridge is hardwired into the address bus so that seems fair 
    nes: Option<Nes>,
    target_speed: f64,
    game_speed: f64,
    paused: bool,
    pub video_output: TextureHandle,  // accessed directly in main.rs, no point in using a getter
    frame: u64,

    time: f64,

    audio_output: Option<AudioStream>,
    avg_sample_rate: f64,
    cpu_cycle_at_last_sample: u64,
    cached_cycles_per_sample: f32,
    stereo_pan: f32,
    rewind_state_index: usize,
    rewind_states: Vec<Nes>,

    nes_frame: Rc<RefCell<Vec<u8>>>,
}

pub struct RomData {
    pub prg_rom: Vec<u8>,
    pub chr_rom: Vec<u8>,
    pub chr_rom_is_ram: bool,
    pub mapper_id: u8, // fine for iNES 1.0 files
    pub mirroring_config: Mirroring,
}

const CPU_CYCLES_PER_FRAME: f32 = 29780.5;
const DEFAULT_FRAMERATE: f64 = 60.0;
const EXPONENTIAL_MOVING_AVG_BETA: f64 = 0.999;

impl Emulator {
    pub fn new(texture_handle: TextureHandle, audio_output: Option<AudioStream>) -> Self {
        let init_cycles_per_sample = match audio_output.as_ref() {
            Some(s) => Self::cycles_per_sample(s.sample_rate, 1.0),
            None => 0.0,
        };

        Emulator {
            nes: None,
            game_speed: 1.0,
            target_speed: 1.0,
            paused: false,
            video_output: texture_handle,
            audio_output,
            avg_sample_rate: 1000.0,
            cpu_cycle_at_last_sample: 0,
            cached_cycles_per_sample: init_cycles_per_sample,
            stereo_pan: 0.0,
            frame: 0,
            time: 0.0,
            rewind_state_index: 0,
            rewind_states: Vec::new(),
            nes_frame: Rc::new(RefCell::new(vec![0u8; 256usize * 240 * 4])),
        }
    }

    pub fn load_game(&mut self, rom_data: RomData) {
        self.nes = Some(Nes::new(new_cartridge(rom_data), Rc::clone(&self.nes_frame)));
    }

    pub fn game_loaded(&self) -> bool {
        self.nes.is_some()
    }

    pub fn get_set_speed(&mut self, speed: Option<f64>) -> f64 {
        if let Some(speed) = speed {
            assert!(speed >= 0.0);
            self.target_speed = speed;
        }
        self.target_speed
    }

    pub fn get_set_pause(&mut self, pause: Option<bool>) -> bool {
        if let Some(pause) = pause {
            if !pause {
                while self.rewind_states.len() - 1 != self.rewind_state_index {
                    self.rewind_states.pop();
                }
            }
            self.paused = pause;
        }
        self.paused
    }

    pub fn scrub_by(&mut self, n_frames: i32) {
        if self.paused && !self.rewind_states.is_empty() {
            self.rewind_state_index = clamp(
                self.rewind_state_index.saturating_add_signed(n_frames as isize),
                0,
                self.rewind_states.len() - 1
            );
        }
    }

    pub fn update(&mut self, time: f64) -> bool {
        self.time = time;

        if self.nes.is_none() {
            return false;
        }

        let frame_length = 1.0 / (self.game_speed * DEFAULT_FRAMERATE);
        let frame_number = (self.time / frame_length) as u64;

        if frame_number > self.frame {

            if self.game_speed != self.target_speed {
                self.game_speed = self.target_speed;

                if let Some(stream) = &self.audio_output {
                    self.cached_cycles_per_sample =
                        Self::cycles_per_sample(stream.sample_rate, self.game_speed as f32);
                    self.avg_sample_rate = self.cached_cycles_per_sample as f64;
                }
                let frame_length = 1.0 / (self.game_speed * DEFAULT_FRAMERATE);
                let new_frame_number = (self.time / frame_length) as u64;

                self.frame = new_frame_number;
            } else {
                self.frame = frame_number;
            }

            if self.paused {
                if !self.rewind_states.is_empty() {
                    self.nes = Some(self.rewind_states[self.rewind_state_index].clone());

                }
                self.run_to_vblank(false);
            } else {
                if !self.rewind_states.is_empty() {
                    self.rewind_state_index = self.rewind_states.len() - 1;
                }
                self.run_to_vblank(true);
            }

            self.video_output.set(
                ColorImage::from_rgba_unmultiplied(
                    [256, 240],
                    self.nes_frame.borrow().as_slice()
                ),
                TextureOptions {
                    magnification: TextureFilter::Nearest,
                    minification: TextureFilter::Nearest,
                },
            );
            true
        } else {
            false
        }
    }

    fn run_to_vblank(&mut self, create_rewind_state: bool) {

        if create_rewind_state {
            self.rewind_states.push(self.nes.as_ref().unwrap().clone());
        }

        loop {
            self.try_audio_sample();
            if let Some(nes) = self.nes.as_mut() {
                cpu::step_cpu(nes);

                ppu::step_ppu(nes);
                ppu::step_ppu(nes);
                ppu::step_ppu(nes);

                apu::step_apu(nes);

                if nes.ppu.scanline == 239
                    && (nes.ppu.scanline_cycle >= 257 && nes.ppu.scanline_cycle <= 259)
                {
                    break;
                }
            }
        }
    }

    fn try_audio_sample(&mut self) {
        if !self.paused {
            if let Some(nes) = self.nes.as_mut() {
                let cycle_diff = (nes.cpu.cycles - self.cpu_cycle_at_last_sample);

                if cycle_diff == self.cached_cycles_per_sample.floor() as u64
                    && self.avg_sample_rate > self.cached_cycles_per_sample as f64
                {
                    self.do_sample();
                } else if cycle_diff >= self.cached_cycles_per_sample.ceil() as u64 {
                    self.do_sample();
                }
            }
        }
    }

    pub fn update_controller(&mut self, num: u8, new_state: ButtonState) {
        if let Some(nes) = self.nes.as_mut() {
            match num {
                1 => nes.con1.update_button_state(new_state),
                2 => nes.con2.update_button_state(new_state),
                x => panic!("Controller {x} doesn't exist"),
            }
        }
    }

    fn cycles_per_sample(sample_rate: f32, game_speed: f32) -> f32 {
        let samples_per_frame = sample_rate / (game_speed * DEFAULT_FRAMERATE as f32);
        CPU_CYCLES_PER_FRAME / samples_per_frame
    }

    fn do_sample(&mut self) {
        if let Some(nes) = self.nes.as_mut() {
            let new_sample = nes.apu.get_sample(self.stereo_pan);
            let _ = self
                .audio_output
                .as_mut()
                .unwrap()
                .sender
                .try_send(new_sample);

            let rolling_average = EXPONENTIAL_MOVING_AVG_BETA * self.avg_sample_rate
                + (1.0 - EXPONENTIAL_MOVING_AVG_BETA)
                    * (nes.cpu.cycles - self.cpu_cycle_at_last_sample) as f64;

            self.cpu_cycle_at_last_sample = nes.cpu.cycles;
            self.avg_sample_rate = rolling_average;
        }
    }

    pub fn instructions_for_debug(&self) -> Vec<InstructionAndOperands> {
        let mut instrs: Vec<InstructionAndOperands> = Vec::with_capacity(0xFFFF - 0x8000);
        if let Some(nes) = self.nes.as_ref() {
            let mut address = 0x8000;
            while address >= 0x8000 {
                let opc = nes.cart.read_prg_rom(address);
                let op1 = nes.cart.read_prg_rom(address.wrapping_add(1));
                let op2 = nes.cart.read_prg_rom(address.wrapping_add(2));

                let instruction = INSTRUCTIONS[opc as usize];
                let len = instruction.number_of_operands();

                assert!(instrs.len() <= 0xFFFF - 0x8000);

                instrs.push(
                    InstructionAndOperands {
                            address,
                            instruction,
                            operand_1: if len > 0 {Some(op1)} else {None},
                            operand_2: if len > 1 {Some(op2)} else {None},
                    }
                );
                address = address.wrapping_add(1 + len as u16);
            }
        }
        instrs
    }
}
