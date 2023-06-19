#![feature(bigint_helper_methods)]
#![feature(mixed_integer_ops)]

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use ggez::{Context, ContextBuilder, GameResult, timer};
use ggez::event::{self, EventHandler, KeyCode, KeyMods, Button, Axis};
use ggez::conf::{WindowMode, WindowSetup};
use ggez::graphics::{self, DrawParam, Image};
use ggez::mint::{Vector2, Point2};

use std::sync::mpsc;
use std::fs::File;

mod nes;
mod controller;
mod cpu; 
mod ppu;
mod apu;
mod mem;
mod cartridge;
mod emulator;
mod util;

use crate::nes::Nes;
use crate::cartridge::*;

struct Emulator {
    nes: Nes,
    frames: u64,
    scaling: f32,
}

const UP:     u8 = 0b0001_0000;
const DOWN:   u8 = 0b0010_0000;
const LEFT:   u8 = 0b0100_0000;
const RIGHT:  u8 = 0b1000_0000;
const START:  u8 = 0b0000_1000;
const SELECT: u8 = 0b0000_0100;
const A:      u8 = 0b0000_0001;
const B:      u8 = 0b0000_0010;

const FRAMERATE: u32 = 60;
const JOY_DEADZONE: f32 = 0.4;




impl EventHandler for Emulator {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        if timer::check_update_time(ctx, FRAMERATE) {
            emulator::run_to_vblank(&mut self.nes);
        }
        if self.frames % 100 == 0 {
            println!("FPS = {}", timer::fps(ctx));
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        // Draw emulator output
        let image = Image::from_rgba8(
            ctx, 
            256, 
            240, 
            &self.nes.frame
        )?;
        let dp = DrawParam::default()
            .scale(Vector2{x: self.scaling, y: self.scaling})
            .dest(Point2{x: 0.0, y: -8.0*self.scaling});
        graphics::draw(ctx, &image, dp)?;

        // Push image to screen
        graphics::present(ctx)?;

        self.frames += 1;

        Ok(())
    }

    fn key_down_event(&mut self, _ctx: &mut Context, keycode: KeyCode, _keymods: KeyMods, _repeat: bool) {
        match keycode {
            KeyCode::W        => self.nes.con1.button_state |= UP,
            KeyCode::R        => self.nes.con1.button_state |= DOWN,
            KeyCode::A        => self.nes.con1.button_state |= LEFT,
            KeyCode::S        => self.nes.con1.button_state |= RIGHT,
            KeyCode::RBracket => self.nes.con1.button_state |= START,
            KeyCode::LBracket => self.nes.con1.button_state |= SELECT,
            KeyCode::E        => self.nes.con1.button_state |= A,
            KeyCode::N        => self.nes.con1.button_state |= B,
            KeyCode::Space => {
                self.nes.cpu.pause = true;
                self.nes.cpu.target = self.nes.cpu.instruction_count + 10;
            }
            _ => ()
        }   
    }

    fn key_up_event(&mut self, _ctx: &mut Context, keycode: KeyCode, _keymods: KeyMods) {
        match keycode {
            KeyCode::W        => self.nes.con1.button_state &= !UP,
            KeyCode::R        => self.nes.con1.button_state &= !DOWN,
            KeyCode::A        => self.nes.con1.button_state &= !LEFT,
            KeyCode::S        => self.nes.con1.button_state &= !RIGHT,
            KeyCode::RBracket => self.nes.con1.button_state &= !START,
            KeyCode::LBracket => self.nes.con1.button_state &= !SELECT,
            KeyCode::E        => self.nes.con1.button_state &= !A,
            KeyCode::N        => self.nes.con1.button_state &= !B,
            _ => ()
        }   
    }

    fn gamepad_button_down_event(&mut self, _ctx: &mut Context, btn: Button, _id: event::GamepadId) {
        match btn {
            Button::DPadUp    => self.nes.con1.button_state |= UP,
            Button::DPadDown  => self.nes.con1.button_state |= DOWN,
            Button::DPadLeft  => self.nes.con1.button_state |= LEFT,
            Button::DPadRight => self.nes.con1.button_state |= RIGHT,
            Button::Start     => self.nes.con1.button_state |= START,
            Button::Select    => self.nes.con1.button_state |= SELECT,
            Button::South     => self.nes.con1.button_state |= A,
            Button::West      => self.nes.con1.button_state |= B,
            _ => (),
        }
    }

    fn gamepad_button_up_event(&mut self, _ctx: &mut Context, btn: Button, _id: event::GamepadId) {
        match btn {
            Button::DPadUp    => self.nes.con1.button_state &= !UP,
            Button::DPadDown  => self.nes.con1.button_state &= !DOWN,
            Button::DPadLeft  => self.nes.con1.button_state &= !LEFT,
            Button::DPadRight => self.nes.con1.button_state &= !RIGHT,
            Button::Start     => self.nes.con1.button_state &= !START,
            Button::Select    => self.nes.con1.button_state &= !SELECT,
            Button::South     => self.nes.con1.button_state &= !A,
            Button::West      => self.nes.con1.button_state &= !B,
            _ => (),
        }
    }

    fn gamepad_axis_event(&mut self, _ctx: &mut Context, axis: event::Axis, value: f32, _id: event::GamepadId) {
        match axis {
            Axis::LeftStickY => {
                println!("y axis val {value}");
                self.nes.con1.button_state &= !(UP | DOWN);
                if value >= JOY_DEADZONE {
                    self.nes.con1.button_state |= UP;
                }
                else if value <= -JOY_DEADZONE {
                    self.nes.con1.button_state |= DOWN;
                }
            }
            Axis::LeftStickX => {
                println!("x axis val {value}");
                self.nes.con1.button_state &= !(LEFT | RIGHT);
                if value <= -JOY_DEADZONE {
                    self.nes.con1.button_state |= LEFT;
                }
                else if value >= JOY_DEADZONE {
                    self.nes.con1.button_state |= RIGHT;
                }
            }
            _ => (),
        }
    }

}







fn main() {
    let commandline_args: Vec<String> = std::env::args().collect();

    if commandline_args.len() != 3 {
        panic!("Invalid number of arguments");
    }

    let ines_data = std::fs::read(&commandline_args[1])
        .expect("Failed to read rom");

    // If the file isn't long enough to contain iNES header, quit
    if ines_data.len() < 16 {
        panic!();
    }
    
    // If file doesn't contain iNES magic number, quit
    if &ines_data[0..4] != b"NES\x1A" {
        panic!("Not a valid iNES file");
    }



    let scaling = (&commandline_args[2]).parse::<f32>().expect("Invalid scaling value");

    // Queue used to send values from the APU to the audio thread
    // Although this is a multiple producer single consumer queue, there is only one producer
    // (the APU)

    // Would be good to swap this out at somepoint for something that lets me query the 
    // queue length to deal with popping and buffer size and stuff
    let (audio_queue_producer, audio_queue_consumer) = mpsc::channel::<(f32, f32)>();

    let mut prev_sample = (0.0, 0.0);

    // This is WASAPI
    let host = cpal::default_host();
    
    let device = host.default_output_device().unwrap();

    let config = device.default_output_config().unwrap().config();

    let stream = device.build_output_stream(
        &config,
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            // For each left and right sample in the sample list
            for frame in data.chunks_mut(2) {

                let queue_sample = audio_queue_consumer.try_recv();
                
                let new_sample = if queue_sample.is_ok() {
                    let new = queue_sample.unwrap();
                    prev_sample = new;
                    new
                } else {
                    // println!("NOT FED");
                    prev_sample
                };

                let cpal_l_sample = cpal::Sample::from::<f32>(&new_sample.0);
                let cpal_r_sample = cpal::Sample::from::<f32>(&new_sample.1);

                // let cpal_sample = cpal::Sample::from::<f32>(&new_sample);

                frame[0] = cpal_l_sample;
                frame[1] = cpal_r_sample;

                // for sample in frame.iter_mut() {
                    // *sample = cpal_sample;
                // }


            }
        },
        |_err| {
            panic!();
        },
     ).expect("Problem creating the stream");






     let logfile = File::create("emulator.log").unwrap();

     
    let cartridge = new_cartridge(ines_data);
    let nes       = Nes::new(cartridge, audio_queue_producer, config.sample_rate.0, logfile);
    let emulator  = Emulator {nes, frames: 0, scaling};

    let cb = ContextBuilder::new("nes-emu", "Adam Blance")
        .window_mode(WindowMode::default().dimensions(256.0*scaling, 224.0*scaling))
        .window_setup(WindowSetup::default().title("R-nemUST"));

    let (mut ctx, event_loop) = cb.build().unwrap();

    // Nearest neighbor will prevent the frame from becoming blurry when scaling
    graphics::set_default_filter(&mut ctx, graphics::FilterMode::Nearest);

    stream.play().unwrap();

    event::run(ctx, event_loop, emulator);

}

fn new_cartridge(ines_data: Vec<u8>) -> Box<dyn Cartridge> {
    
    // Information extracted from iNES header
    let num_prg_16kb_chunks   = ines_data[4] as usize;
    let num_chr_8kb_chunks    = ines_data[5] as usize;
    let has_prg_ram           = (ines_data[6] & 0b0010) > 0;

    let v_or_h_mirroring    = if (ines_data[6] & 0b0001) > 0 {Mirroring::Vertical} else {Mirroring::Horizontal};
    let four_screen_mirroring = (ines_data[6] & 0b1000) > 0;
    
    let mapper_id             = (ines_data[6] >> 4) 
                                | (ines_data[7] & 0b1111_0000);

    let chr_rom_is_ram = num_chr_8kb_chunks == 0;

    // Program ROM begins immediately after 16 byte header
    let prg_end = 16 + (num_prg_16kb_chunks * 0x4000);
    let chr_end = prg_end + (num_chr_8kb_chunks * 0x2000);

    let chr_rom_is_ram = prg_end == chr_end;

    let prg_rom = ines_data[16..prg_end].to_vec();

    let chr_rom = if !chr_rom_is_ram {
        ines_data[prg_end..chr_end].to_vec()
    } else {
        [0u8; 0x2000].to_vec()
    };

    match mapper_id {
        0 => Box::new(CartridgeM0::new(prg_rom, chr_rom, v_or_h_mirroring)),
        1 => Box::new(CartridgeM1::new(prg_rom, chr_rom, chr_rom_is_ram)),
        2 => Box::new(CartridgeM2::new(prg_rom, chr_rom, chr_rom_is_ram, v_or_h_mirroring)),
        3 => Box::new(CartridgeM3::new(prg_rom, chr_rom, v_or_h_mirroring)),
        4 => Box::new(CartridgeM4::new(prg_rom, chr_rom)),
        7 => Box::new(CartridgeM7::new(prg_rom)),
        _ => unimplemented!("Mapper {} not implemented", mapper_id),
    }



}