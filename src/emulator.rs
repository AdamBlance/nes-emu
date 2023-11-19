use crate::nes::cartridge::cartridge::new_cartridge;
use crate::nes::cartridge::Mirroring;
use crate::nes::controller::ButtonState;
use crate::nes::{cartridge, Nes};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use eframe::egui::{ColorImage, Context, TextureFilter, TextureHandle, TextureOptions};
use std::error::Error;
use std::sync::mpsc;
use std::sync::mpsc::{Sender, SyncSender};

// use crate::util::concat_u8;
// use crate::nes::mem::read_mem;
use crate::nes::cpu;
use crate::nes::ppu;
use crate::nes::apu;
// use crate::nes::apu::{self, };
//

pub struct AudioStream {
    pub sender: SyncSender<(f32, f32)>,
    pub sample_rate: f32,
}

pub struct Emulator {
    // The emulator isn't gonna have a NES unless it has a game cartridge
    // The cartridge is hardwired into the address bus so that seems fair
    pub nes: Option<Nes>,
    target_speed: f64,
    pub game_speed: f64,
    paused: bool,
    pub video_output: TextureHandle,
    pub frame: u64,

    pub time: f64,

    audio_output: Option<AudioStream>,
    avg_sample_rate: f64,
    cpu_cycle_at_last_sample: u64,
    cached_cycles_per_sample: f32,
    stereo_pan: f32,
}

// pub enum MirroringConfig {
//     Vertical,
//     Horizontal,
//     Other
// }

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
            // Will take more samples than necessary to build up a buffer on the audio queue
            // until average levels out.
            avg_sample_rate: 1000.0,
            cpu_cycle_at_last_sample: 0,
            cached_cycles_per_sample: init_cycles_per_sample,
            stereo_pan: 0.0,
            frame: 0,
            time: 0.0,
        }
    }

    pub fn load_game(&mut self, rom_data: RomData) {
        self.nes = Some(Nes::new(new_cartridge(rom_data)));



    }

    // pub fn toggle_pause(&mut self) {
    //     self.paused = !self.paused;
    // }
    //


    // I think maybe? To get rid of the weirdness where a surplus of samples builds up in the audio
    // queue we should only pick up the change in speed when we generate a new NES frame, that lets
    // us reset everything without moving the cycles window forward or backwards in the middle of
    // doing stuff


    pub fn set_speed(&mut self, speed: f64) {
        assert!(speed >= 0.0);
        self.target_speed = speed;

        // if let Some(stream) = &self.audio_output {
        //     self.cached_cycles_per_sample = Self::cycles_per_sample(stream.sample_rate, self.game_speed as f32);
        // }

        // let frame_length = 1.0 / (self.game_speed * DEFAULT_FRAMERATE);
        // let frame_number = (self.time / frame_length) as u64;
        //
        // self.frame = frame_number;
        //
        // self.avg_sample_rate = self.cached_cycles_per_sample as f64;

    }

    pub fn update(&mut self, time: f64) -> bool {
        self.time = time;

        if self.paused {
            return false;
        }

        let frame_length = 1.0 / (self.game_speed * DEFAULT_FRAMERATE);
        let frame_number = (self.time / frame_length) as u64;

        if frame_number > self.frame {

            if self.game_speed != self.target_speed {
                self.game_speed = self.target_speed;

                if let Some(stream) = &self.audio_output {
                    self.cached_cycles_per_sample = Self::cycles_per_sample(stream.sample_rate, self.game_speed as f32);
                    self.avg_sample_rate = self.cached_cycles_per_sample as f64;
                }
                let frame_length = 1.0 / (self.game_speed * DEFAULT_FRAMERATE);
                let new_frame_number = (self.time / frame_length) as u64;

                self.frame = new_frame_number;

            } else {
                self.frame = frame_number;
            }

            self.run_to_vblank();

            self.video_output.set(
                ColorImage::from_rgba_unmultiplied(
                    [256, 240],
                    self.nes.as_ref().unwrap().frame.as_slice()
                ),
                TextureOptions {magnification: TextureFilter::Nearest, minification: TextureFilter::Nearest}
            );

            true
        } else {
            false
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
        if let Some(nes) = self.nes.as_mut() {

                let cycle_diff = (nes.cpu.cycles - self.cpu_cycle_at_last_sample);

                if cycle_diff == self.cached_cycles_per_sample.floor() as u64 &&
                    self.avg_sample_rate > self.cached_cycles_per_sample as f64 {
                    self.do_sample();
                } else if cycle_diff >= self.cached_cycles_per_sample.ceil() as u64 {
                    self.do_sample();
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
            self.audio_output.as_mut().unwrap().sender.send(new_sample).unwrap();

            let rolling_average = EXPONENTIAL_MOVING_AVG_BETA * self.avg_sample_rate + (1.0 - EXPONENTIAL_MOVING_AVG_BETA) * (nes.cpu.cycles - self.cpu_cycle_at_last_sample) as f64;

            self.cpu_cycle_at_last_sample = nes.cpu.cycles;

            self.avg_sample_rate = rolling_average;
        }


    }



}

// fn run_one_ppu_cycle(nes: &mut Nes) {
//     match (nes.ppu.cycles) % 3 {
//         0 => {
//             cpu::step_cpu(nes);
//             ppu::step_ppu(nes);
//         }
//         1 => ppu::step_ppu(nes),
//         2 => {
//             ppu::step_ppu(nes);
//             apu::step_apu(nes);
//         }
//         _ => unreachable!()
//     }
// }
//

// let's do exponentially weighted average

// so like avg = beta * prev_avg + (1 - beta) * new_val
// 0 <= beta <= 1, 1 / (1 - beta) gives us an approximation for the number of values we're averaging over

// you can see what it's doing right?
// only thing we have to take note of is the initial value of the average
// Probably we should start the average slightly above the target cycles per sample
// so that the emulator spits out more samples than it needs at the start to get the average down
// that will give us some buffer room so the cpal callback isn't working on the bleeding edge
// of the samples that we just created

//
// // How far to pan the square waves, 0 being no pan (mono)
// const STEREO_PAN: f32 = 0.30;
//
// fn do_sample(nes: &mut Nes) {
//
//     /*
//
//     Basically want to do weighted average
//     So it'll be
//
//     ((average so far * number_of_samples) + cycles since last sample [either 40 or 41])
//     -----------------------------------------------------------------------------------
//                                    number_of_samples + 1
//
//     */
//
//     let numerator = (nes.apu.average_cycles_per_sample * (nes.apu.total_sample_count as f64)) + (nes.apu.cycles_since_last_sample as f64);
//     let denominator = (nes.apu.total_sample_count + 1) as f64;
//
//     nes.apu.average_cycles_per_sample = numerator / denominator;
//
//     nes.apu.total_sample_count += 1;
//     nes.apu.cycles_since_last_sample = 0;
//
//
//     // These will be between 0.0 and 15.0
//     let sq1_output = apu::square_channel_output(&nes.apu.square1) * 1.0;
//     let sq2_output = apu::square_channel_output(&nes.apu.square2) * 1.0;
//     let tri_output = apu::triangle_channel_output(&nes.apu.triangle) * 1.0;
//     let noise = apu::noise_channel_output(&nes.apu.noise) * 1.0;
//     let sample = apu::sample_channel_output(&nes.apu.sample) * 1.0;
//
//     let epsilon = 0.00001;
//     let pos_bias = 1.0 + STEREO_PAN;
//     let neg_bias = 1.0 - STEREO_PAN;
//     let pulse1_out = 95.88 / ((8128.0 / (pos_bias*sq1_output + neg_bias*sq2_output + epsilon)) + 100.0);
//     let pulse2_out = 95.88 / ((8128.0 / (pos_bias*sq2_output + neg_bias*sq1_output + epsilon)) + 100.0);
//     let other_out = 159.79 / ( ( 1.0 / ((tri_output/8227.0) + (noise/12241.0) + (sample/22638.0) + epsilon) ) + 100.0);
//
//     let output_val = (pulse1_out + other_out, pulse2_out + other_out);
//
//     nes.apu.audio_queue.send(output_val).expect("something wrong happened when appending to audio queue");
// }
