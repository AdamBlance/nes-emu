use crate::app::NesButtonState;
use crate::nes::cartridge::{mapper0, mapper1, mapper2, mapper3, mapper4, mapper7, Cartridge};
use crate::nes::Nes;
use eframe::egui::{ColorImage, TextureFilter, TextureHandle, TextureOptions};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc::SyncSender;

use crate::nes::apu;
use crate::nes::cartridge::cartridge_def::RomConfig;
use crate::nes::cpu;
use crate::nes::cpu::lookup_table::INSTRUCTIONS;
use crate::nes::ppu;

use crate::nes::cpu::debugger::{CpuDebuggerInstruction, InstrBytes};

/*
    Would be nice to create a state machine diagram to show how the program works when pausing,
    unpausing, opening the debugger, rewinding, scrubbing, stepping forward and backward through
    instructions, etc.
    Will prevent future headaches I think.
    Would also be nice to fix that off-by-one error that happens when you unpause and the game
    lurches a frame.
*/

const CPU_CYCLES_PER_FRAME: f32 = 29780.5;
const DEFAULT_FRAMERATE: f64 = 60.0;
const EXPONENTIAL_MOVING_AVG_BETA: f64 = 0.999;

pub struct AudioStream {
    pub sender: SyncSender<(f32, f32)>,
    pub sample_rate: f32,
}

pub struct Emulator {
    // The emulator isn't gonna have a NES unless it has a game cartridge
    // The cartridge is hardwired into the address bus so that seems fair
    pub nes: Option<Nes>,
    target_speed: f64,
    game_speed: f64,
    paused: bool,
    pub video_output: TextureHandle, // accessed directly in main.rs, no point in using a getter
    frame: u64,

    time: f64,

    audio_output: Option<AudioStream>,
    volume: f64,
    avg_sample_rate: f64,
    cpu_cycle_at_last_sample: u64,
    cached_cycles_per_sample: f32,
    stereo_pan: f32,
    rewind_state_index: f32,
    rewind_states: Vec<Nes>,

    nes_frame: Rc<RefCell<Vec<u8>>>,

    pub instruction_cache: Vec<CpuDebuggerInstruction>,
}

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
            volume: 1.0,
            avg_sample_rate: 1000.0,
            cpu_cycle_at_last_sample: 0,
            cached_cycles_per_sample: init_cycles_per_sample,
            stereo_pan: 0.0,
            frame: 0,
            time: 0.0,
            rewind_state_index: 0.0,
            rewind_states: Vec::new(),
            nes_frame: Rc::new(RefCell::new(vec![0u8; 256usize * 240 * 4])),
            instruction_cache: Vec::new(),
        }
    }

    pub fn load_game(&mut self, rom_config: RomConfig) {
        let cartridge: Box<dyn Cartridge> = match rom_config.ines_mapper_id {
            0 => Box::new(mapper0::CartridgeM0::new(rom_config)),
            1 => Box::new(mapper1::CartridgeM1::new(rom_config)),
            2 => Box::new(mapper2::CartridgeM2::new(rom_config)),
            3 => Box::new(mapper3::CartridgeM3::new(rom_config)),
            4 => Box::new(mapper4::CartridgeM4::new(rom_config)),
            7 => Box::new(mapper7::CartridgeM7::new(rom_config)),
            id => unimplemented!("Mapper {id} not implemented"),
        };

        self.nes = Some(Nes::new(cartridge, Rc::clone(&self.nes_frame)));
        self.update_prg_rom_debug_cache();
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
            if self.paused && !pause && !self.rewind_states.is_empty() {
                // self.rewind_states.truncate(self.rewind_state_index as usize + 1);
                while self.rewind_states.len() - 1 != self.rewind_state_index as usize {
                    self.rewind_states.pop();
                }
            }
            self.paused = pause;
        }
        self.paused
    }

    pub fn get_set_volume(&mut self, volume: Option<f64>) -> f64 {
        if let Some(v) = volume {
            assert!(v <= 1.0);
            self.volume = v;
        }
        self.volume
    }

    pub fn scrub_by(&mut self, n_frames: f32) {
        if self.paused && !self.rewind_states.is_empty() && n_frames != 0.0 {
            self.rewind_state_index = (self.rewind_state_index + n_frames)
                .clamp(0.0, (self.rewind_states.len() - 1) as f32);
            self.nes = Some(self.rewind_states[self.rewind_state_index.round() as usize].clone());
            self.run_to_vblank();
        }
    }

    fn update_prg_rom_debug_cache(&mut self) {
        if let Some(nes) = self.nes.as_mut() {
            let prg_rom_snapshot: Vec<u8> = (0x8000..=0xFFFF)
                .map(|i| nes.cart.read_prg_rom(i))
                .collect();
            self.instruction_cache = Emulator::instructions_for_debug(prg_rom_snapshot.as_slice());
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

            /*

            Right so CHR RAM and PRG RAM can be behind Rc.
            Stuff behind Rc is usually immutable (because otherwise you could potentially have multiple
            mutable references at the same time to the same part of memory).

            Turns out that you can actually mutate the data behind an Rc using make_mut.
            This works in a copy-on-write way, so if multiple Rc exist that point to the same allocation
            make_mut will clone the existing allocation then mutate it.

            So I think all we need to do is make CHR RAM and PRG RAM Rc and modify them with make_mut

            So when we clone the nes state into the rewind state list, both the rewind state and the
            current nes state should be pointing to the same RAM allocations. The moment the current
            nes modifies the ram it will be cloned and will point to a new allocation

             */

            if !self.paused {
                if !self.rewind_states.is_empty() {
                    self.rewind_state_index = (self.rewind_states.len() - 1) as f32;
                }
                let state = self.nes.as_ref().unwrap().clone();
                self.rewind_states.push(state);
                self.run_to_vblank();
            }

            self.video_output.set(
                ColorImage::from_rgba_unmultiplied([256, 240], self.nes_frame.borrow().as_slice()),
                TextureOptions {
                    magnification: TextureFilter::Nearest,
                    minification: TextureFilter::Nearest,
                    wrap_mode: Default::default(),
                },
            );
            true
        } else {
            false
        }
    }

    pub fn run_one_cpu_instruction(&mut self) {
        if let Some(nes) = self.nes.as_mut() {
            loop {
                let end_of_instr = cpu::step_cpu(nes);

                ppu::step_ppu(nes);
                ppu::step_ppu(nes);
                ppu::step_ppu(nes);

                apu::step_apu(nes);

                if end_of_instr {
                    break;
                }
            }
        }
        self.update_prg_rom_debug_cache();
    }

    fn run_to_vblank(&mut self) {
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
                let cycle_diff = nes.cpu.cycles - self.cpu_cycle_at_last_sample;

                if (cycle_diff == self.cached_cycles_per_sample.floor() as u64
                    && self.avg_sample_rate > self.cached_cycles_per_sample as f64)
                    || cycle_diff >= self.cached_cycles_per_sample.ceil() as u64
                {
                    // TODO: This should technically be (cached_cycles_per_sample + 1).floor() I think
                    self.do_sample();
                }
            }
        }
    }

    pub fn update_controller(&mut self, num: u8, pressed_buttons: NesButtonState) {
        if let Some(nes) = self.nes.as_mut() {
            match num {
                1 => nes.con1.update_button_state(pressed_buttons),
                // 2 => nes.con2.update_button_state(&pressed_buttons),
                _ => panic!("Controller doesn't exist"),
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
            let new_sample_multiplied = (
                new_sample.0 * self.volume as f32,
                new_sample.1 * self.volume as f32,
            );

            let _ = self
                .audio_output
                .as_mut()
                .unwrap()
                .sender
                .try_send(new_sample_multiplied);

            let rolling_average = EXPONENTIAL_MOVING_AVG_BETA * self.avg_sample_rate
                + (1.0 - EXPONENTIAL_MOVING_AVG_BETA)
                    * (nes.cpu.cycles - self.cpu_cycle_at_last_sample) as f64;

            self.cpu_cycle_at_last_sample = nes.cpu.cycles;
            self.avg_sample_rate = rolling_average;
        }
    }

    fn instructions_for_debug(prg_rom: &[u8]) -> Vec<CpuDebuggerInstruction> {
        assert_eq!(prg_rom.len(), 0x8000);
        let mut opcodes: Vec<CpuDebuggerInstruction> = Vec::new();
        let mut window = prg_rom.array_windows().enumerate();
        while let Some((index, [opc, arg1, arg2])) = window.next() {
            let instr = INSTRUCTIONS[*opc as usize];
            if !instr.is_unofficial() {
                opcodes.push(CpuDebuggerInstruction {
                    opc_addr: 0x8000 + index as u16,
                    bytes: match instr.number_of_operands() {
                        0 => InstrBytes::I1(*opc),
                        1 => InstrBytes::I2(*opc, *arg1),
                        2 => InstrBytes::I3(*opc, *arg1, *arg2),
                        _ => unreachable!(),
                    },
                });
                let _ = window.advance_by(instr.number_of_operands() as usize);
            }
        }
        opcodes
    }
}
