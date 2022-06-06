#![feature(bigint_helper_methods)]
#![feature(mixed_integer_ops)]

use crate::hw::Nes;
use crate::hw::Cartridge;

use ggez::{Context, ContextBuilder, GameResult, timer};
use ggez::event::{self, EventHandler, KeyCode, KeyMods};
use ggez::conf::WindowMode;
use ggez::graphics::{self, Rect, Mesh, DrawMode, DrawParam, Color, Image};
use ggez::mint::{Vector2, Point2};

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


impl EventHandler for Emulator {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        while timer::check_update_time(ctx, FRAMERATE) {
            emu::run_to_vblank(&mut self.nes);
        }
        if self.frames % 100 == 0 {
            println!("FPS = {}", timer::fps(ctx));
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        // Draw background
        let bg = Mesh::new_rectangle(
            ctx, 
            DrawMode::fill(), 
            Rect::new(0.0, 0.0, 1024.0, 960.0), 
            Color::WHITE
        )?;
        graphics::draw(ctx, &bg, DrawParam::default());

        // Draw emulator output
        let image = Image::from_rgba8(
            ctx, 
            256, 
            240, 
            &self.nes.frame
        )?;
        let dp = DrawParam::new()
            .scale(Vector2{x: 3.0, y: 3.0})
            .dest(Point2{x: 50.0, y: 50.0});

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
            KeyCode::E        => self.nes.controller1.button_state &= !B,
            KeyCode::I        => self.nes.controller1.button_state &= !A,
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
    
    let cartridge = Cartridge::new(ines_data);
    let nes       = Nes::new(cartridge);
    let emulator  = Emulator {nes, frames: 0};
        
    let cb = ContextBuilder::new("nes-emu", "Adam Blance")
        .window_mode(WindowMode::default().dimensions(1024.0, 960.0));

    let (mut ctx, event_loop) = cb.build().unwrap();

    // Nearest neighbor will prevent the frame from becoming blurry when scaling
    graphics::set_default_filter(&mut ctx, graphics::FilterMode::Nearest);
    
    event::run(ctx, event_loop, emulator);
}
