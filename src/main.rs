#![feature(bigint_helper_methods)]
#![feature(mixed_integer_ops)]

use crate::hw::Nes;
use crate::hw::Cartridge;

use std::thread;

use std::sync::mpsc::{self, Receiver, TryIter};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Data, Sample, SampleFormat};

use ggez::{Context, ContextBuilder, GameResult, timer};
use ggez::event::{self, EventHandler, KeyCode, KeyMods};
use ggez::conf::{WindowMode, WindowSetup};
use ggez::graphics::{self, DrawParam, Image};
use ggez::mint::Vector2;

mod emu;
mod hw;
mod cpu; 
mod ppu;
mod apu;
mod mem;
mod instr_defs;
mod instr_funcs;
mod addressing_funcs;
mod util;
mod logging;
mod mappers;

struct Emulator {
    nes: Nes,
    frames: u64,
}

const RIGHT:  u8 = 0b1000_0000;
const LEFT:   u8 = 0b0100_0000;
const DOWN:   u8 = 0b0010_0000;
const UP:     u8 = 0b0001_0000;
const START:  u8 = 0b0000_1000;
const SELECT: u8 = 0b0000_0100;
const B:      u8 = 0b0000_0010;
const A:      u8 = 0b0000_0001;

const FRAMERATE: u32 = 60;
const SCALING:   f32 = 4.0;


impl EventHandler for Emulator {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        if timer::check_update_time(ctx, FRAMERATE) {
            emu::run_to_vblank(&mut self.nes);
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
        let dp = DrawParam::default().scale(Vector2{x: SCALING, y: SCALING});
        graphics::draw(ctx, &image, dp)?;

        // Push image to screen
        graphics::present(ctx)?;

        self.frames += 1;

        Ok(())
    }

    fn key_down_event(&mut self, _ctx: &mut Context, keycode: KeyCode, _keymods: KeyMods, _repeat: bool) {
        match keycode {
            KeyCode::W        => self.nes.controller1.button_state |= UP,
            KeyCode::A        => self.nes.controller1.button_state |= LEFT,
            KeyCode::R        => self.nes.controller1.button_state |= DOWN,
            KeyCode::S        => self.nes.controller1.button_state |= RIGHT,
            KeyCode::N        => self.nes.controller1.button_state |= B,
            KeyCode::E        => self.nes.controller1.button_state |= A,
            KeyCode::LBracket => self.nes.controller1.button_state |= SELECT,
            KeyCode::RBracket => self.nes.controller1.button_state |= START,
            _ => ()
        }   
    }

    fn key_up_event(&mut self, _ctx: &mut Context, keycode: KeyCode, _keymods: KeyMods) {
        match keycode {
            KeyCode::W        => self.nes.controller1.button_state &= !UP,
            KeyCode::A        => self.nes.controller1.button_state &= !LEFT,
            KeyCode::R        => self.nes.controller1.button_state &= !DOWN,
            KeyCode::S        => self.nes.controller1.button_state &= !RIGHT,
            KeyCode::N        => self.nes.controller1.button_state &= !B,
            KeyCode::E        => self.nes.controller1.button_state &= !A,
            KeyCode::LBracket => self.nes.controller1.button_state &= !SELECT,
            KeyCode::RBracket => self.nes.controller1.button_state &= !START,
            _ => ()
        }   
    }
}


fn main() {
    let commandline_args: Vec<String> = std::env::args().collect();

    if commandline_args.is_empty() {
        panic!("No filename provided");
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
    

    // Queue used to send values from the APU to the audio thread
    // Although this is a multiple producer single consumer queue, there is only one producer
    // (the APU)
    let (audio_queue_producer, audio_queue_consumer) = mpsc::channel::<f32>();

    let mut prev_sample = 0.0;

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

                let cpal_sample = cpal::Sample::from::<f32>(&new_sample);

                for sample in frame.iter_mut() {
                    *sample = cpal_sample;
                }


            }
        },
        |_err| {
            panic!();
        },
     ).expect("Problem creating the stream");

    let cartridge = Cartridge::new(ines_data);
    let nes       = Nes::new(cartridge, audio_queue_producer);
    let emulator  = Emulator {nes, frames: 0};

    let cb = ContextBuilder::new("nes-emu", "Adam Blance")
        .window_mode(WindowMode::default().dimensions(256.0*SCALING, 240.0*SCALING))
        .window_setup(WindowSetup::default().title("R-nemUST"));

    let (mut ctx, event_loop) = cb.build().unwrap();

    // Nearest neighbor will prevent the frame from becoming blurry when scaling
    graphics::set_default_filter(&mut ctx, graphics::FilterMode::Nearest);

    stream.play().unwrap();

    event::run(ctx, event_loop, emulator);

}
