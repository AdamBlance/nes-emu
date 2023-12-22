use crate::nes::cartridge::{
    mapper0, mapper1, mapper2, mapper3, mapper4, mapper7, Cartridge, Mirroring,
};
use crate::nes::controller::ButtonState;
use crate::nes::Nes;
use eframe::egui::{ColorImage, TextureFilter, TextureHandle, TextureOptions};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc::SyncSender;

use crate::nes::apu;
use crate::nes::cpu;
use crate::nes::cpu::lookup_table::{Instruction, INSTRUCTIONS, Mode, Name};
use crate::nes::ppu;

use serde::{Deserialize, Serialize};
use crate::util::concat_u8;

/*
    Would be nice to create a state machine diagram to show how the program works when pausing,
    unpausing, opening the debugger, rewinding, scrubbing, stepping forward and backward through
    instructions, etc.
    Will prevent future headaches I think.
    Would also be nice to fix that off-by-one error that happens when you unpause and the game
    lurches a frame.
*/

pub struct AudioStream {
    pub sender: SyncSender<(f32, f32)>,
    pub sample_rate: f32,
}

#[derive(Debug, Copy, Clone)]
pub enum InstrBytes {
    I1(u8),
    I2(u8, u8),
    I3(u8, u8, u8),
}

#[derive(Debug, Copy, Clone)]
pub struct CpuDebuggerInstruction {
    pub opc_addr: u16,
    pub bytes: InstrBytes,
}

impl CpuDebuggerInstruction {
    pub fn debug_string(&self) -> String {
        let instr = match self.bytes {
            InstrBytes::I1(opc) => INSTRUCTIONS[opc as usize],
            InstrBytes::I2(opc, ..) => INSTRUCTIONS[opc as usize],
            InstrBytes::I3(opc, ..) => INSTRUCTIONS[opc as usize],
        };
        match self.bytes {
            InstrBytes::I1(_) => match instr.mode {
                Mode::Accumulator | Mode::Implied => format!("{:04X} {:#?}", self.opc_addr, instr.name),
                _ => unreachable!(),
            },
            InstrBytes::I2(_, arg1) => match instr.mode {
                Mode::Immediate => format!("{:04X} {:#?} {:02X}", self.opc_addr, instr.name, arg1),
                Mode::ZeroPage => format!("{:04X} {:#?} ZP[{:02X}]", self.opc_addr, instr.name, arg1),
                Mode::ZeroPageX => format!("{:04X} {:#?} ZP[{:02X}+X]", self.opc_addr, instr.name, arg1),
                Mode::ZeroPageY => format!("{:04X} {:#?} ZP[{:02X}+Y]", self.opc_addr, instr.name, arg1),
                Mode::IndirectX => format!("{:04X} {:#?} MEM[ ZP16[{:02X}+X] ]", self.opc_addr, instr.name, arg1),
                Mode::IndirectY => format!("{:04X} {:#?} MEM[ ZP16[{:02X}]+Y ]", self.opc_addr, instr.name, arg1),
                Mode::Relative => format!("{:04X} {:#?} (offset)", self.opc_addr, instr.name),
                _ => unreachable!(),
            },
            InstrBytes::I3(_, arg1, arg2) => match instr.mode {
                Mode::Absolute => format!("{:04X} {:#?} MEM[{:04X}]", self.opc_addr, instr.name, concat_u8(arg2, arg1)),
                Mode::AbsoluteX => format!("{:04X} {:#?} MEM[{:04X}+X]", self.opc_addr, instr.name, concat_u8(arg2, arg1)),
                Mode::AbsoluteY => format!("{:04X} {:#?} MEM[{:04X}+Y]", self.opc_addr, instr.name, concat_u8(arg2, arg1)),
                Mode::AbsoluteI => format!("{:04X} {:#?} MEM[ MEM16[{:04X}] ]", self.opc_addr, instr.name, concat_u8(arg2, arg1)),
                _ => unreachable!(),
            }
        }
    }
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

#[derive(Clone, Serialize, Deserialize)]
pub enum ChrMem {
    Rom(Rc<Vec<u8>>), // Could this just be a boolean? I'm not sure if that's less rusty
    Ram(Rc<Vec<u8>>),
}
impl ChrMem {
    pub fn new(rom_data: Option<Vec<u8>>) -> Self {
        match rom_data {
            Some(data) => Self::Rom(Rc::new(data)),
            None => Self::Ram(Rc::new(vec![0u8; 0x2000])),
        }
    }

    pub fn read(&self, addr: usize) -> u8 {
        match self {
            ChrMem::Rom(rom) => rom[addr],
            ChrMem::Ram(ram) => ram[addr],
        }
    }
    pub fn write(&mut self, addr: usize, value: u8) {
        if let ChrMem::Ram(ram) = self {
            // ram[addr] = value;
            Rc::make_mut(ram)[addr] = value
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct RomConfig {
    pub ines_mapper_id: u8,
    pub ines_mirroring: Mirroring,
    pub data: CartMemory,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct CartMemory {
    pub prg_ram: Option<Rc<Vec<u8>>>,
    pub prg_rom: Rc<Vec<u8>>,
    pub chr_mem: ChrMem,
}

impl CartMemory {
    pub fn new(prg_rom: Vec<u8>, chr_rom: Option<Vec<u8>>, has_prg_ram: bool) -> Self {
        CartMemory {
            prg_ram: match has_prg_ram {
                // TODO: Fix this
                /*
                    Mario 3 writes to and reads from 0x6000-0x7FFF where PRG RAM would be
                    but the cartridge doesn't actually have PRG RAM so any reads should just return
                    the open bus. There are no mentions of Mario 3 relying on open bus behaviour
                    to function but weirdly the game crashes at startup if don't enable PRG RAM.
                    Tried to figure out what was going wrong but to no avail. It's difficult without
                    a CPU instruction view, so maybe I should flesh that out first before trying
                    to fix this bug.
                 */
                true | false => Some(Rc::new(vec![0u8; 0x2000])),
                // true => Some(Rc::new(vec![0u8; 0x2000])),
                // false => None,
            },
            prg_rom: Rc::new(prg_rom),
            chr_mem: ChrMem::new(chr_rom),
        }
    }
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

    pub fn update_prg_rom_debug_cache(&mut self) {
        if let Some(nes) = self.nes.as_mut() {
            let prg_rom_snapshot: Vec<u8> = (0x8000..=0xFFFF)
                .map(|i| nes.cart.read_prg_rom(i))
                .collect();
            self.instruction_cache = Emulator::instructions_for_debug(prg_rom_snapshot.as_slice());
        }
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
                if self.rewind_states.len() > 0 {
                    while self.rewind_states.len() - 1 != self.rewind_state_index as usize {
                        self.rewind_states.pop();
                    }
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
            self.nes =
                Some(self.rewind_states[self.rewind_state_index.round() as usize].clone());
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
                if self.rewind_states.len() > 0 {
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
            let new_sample_multiplied = (new_sample.0 * self.volume as f32, new_sample.1 * self.volume as f32);

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

    pub fn instructions_for_debug(prg_rom: &[u8]) -> Vec<CpuDebuggerInstruction> {
        assert_eq!(prg_rom.len(), 0x8000);
        let mut opcodes: Vec<CpuDebuggerInstruction> = Vec::new();
        let mut window = prg_rom.array_windows().enumerate();
        while let Some((index, [opc, arg1, arg2])) = window.next() {
            let instr = INSTRUCTIONS[*opc as usize];
            if !instr.is_unofficial() {
                opcodes.push(
                    CpuDebuggerInstruction {
                        opc_addr: 0x8000 + index as u16,
                        bytes: match instr.number_of_operands() {
                            0 => InstrBytes::I1(*opc),
                            1 => InstrBytes::I2(*opc, *arg1),
                            2 => InstrBytes::I3(*opc, *arg1, *arg2),
                            _ => unreachable!(),
                        }
                    }
                );
                let _ = window.advance_by(instr.number_of_operands() as usize);
            }
        }
        opcodes
    }
}
