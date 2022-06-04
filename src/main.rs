#![feature(bigint_helper_methods)]
#![feature(mixed_integer_ops)]

use ggez::conf::WindowMode;
use ggez::mint::{Point2, Vector2};
use ggez::{Context, ContextBuilder, GameResult, timer};
use ggez::event::{self, EventHandler, KeyCode};
use ggez::graphics::{self, DrawParam, Transform, Mesh, Color};

use crate::hw::*;

mod emu;
mod hw;
mod instr_defs;
mod util;
mod mem;
mod cpu; 
mod ppu;
mod instr_funcs;
mod addressing_funcs;
mod apu;

const WIDTH: u32 = 256;
const HEIGHT: u32 = 240;

const FRAMERATE: u32 = 60;

const RIGHT:  u8 = 0b1000_0000;
const LEFT:   u8 = 0b0100_0000;
const DOWN:   u8 = 0b0010_0000;
const UP:     u8 = 0b0001_0000;
const START:  u8 = 0b0000_1000;
const SELECT: u8 = 0b0000_0100;
const B:      u8 = 0b0000_0010;
const A:      u8 = 0b0000_0001;


struct Emulator {
    frames: u64,
    nes: hw::Nes,
}

impl EventHandler<ggez::GameError> for Emulator {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {

        // ctx shouldn't have underscore here

        // This is in a while loop incase the computer freezes or something and the game has not 
        // produced a frame in over 1/60th of a second. 
        // In this case, the loop will execute twice, catching the game back up again.
        
        while timer::check_update_time(_ctx, FRAMERATE) {
            emu::run_to_vblank(&mut self.nes);  // I think I know what the problem is
            // run_to_vblank maybe overruns? Or misses a vblank? 
            // I am checking for one cycle in particular, but cpu uses cycles and so does ppu, so 
            // it will be skipping over frames I guess
        }

        if self.frames % 100 == 0 {println!("FPS = {}", timer::fps(_ctx));}


        // if self.nes.jammed {
            // let mut sound = Source::new(_ctx, "/jam.mp3")?;

            // sound.play_detached(_ctx)?;

            // self.nes.jammed = false;
        // }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {

        // Create and draw a filled rectangle mesh.
        let rect = graphics::Rect::new(0.0, 0.0, 1024.0, 960.0);
        let r1 =
            Mesh::new_rectangle(ctx, graphics::DrawMode::fill(), rect, Color::WHITE)?;
        graphics::draw(ctx, &r1, DrawParam::default())?;

        self.frames += 1;


        let image = graphics::Image::from_rgba8(ctx, WIDTH as u16, HEIGHT as u16, &self.nes.frame)?;

        let scalev = Vector2{x: 3.0, y: 3.0};
        let destv = Point2{x: 50.0, y: 50.0};
        let offsetv = Point2{x: 0.0, y: 0.0};
        let trans = Transform::Values{dest: destv, rotation: 0.0, scale: scalev, offset: offsetv};
        let mut dp = DrawParam::new();
        dp.trans = trans;
        graphics::draw(ctx, &image, dp)?;
        graphics::present(ctx)?;
        Ok(())
    }

    fn key_down_event(&mut self, _ctx: &mut Context, keycode: event::KeyCode, _keymods: event::KeyMods, _repeat: bool) {
        match keycode {
            KeyCode::W => self.nes.controller_1.button_state |= UP,
            KeyCode::A => self.nes.controller_1.button_state |= LEFT,
            KeyCode::R => self.nes.controller_1.button_state |= DOWN,
            KeyCode::S => self.nes.controller_1.button_state |= RIGHT,
            KeyCode::E => self.nes.controller_1.button_state |= B,
            KeyCode::I => self.nes.controller_1.button_state |= A,
            KeyCode::LBracket => self.nes.controller_1.button_state |= SELECT,
            KeyCode::RBracket => self.nes.controller_1.button_state |= START,
            _ => ()
        }   
    }

    fn key_up_event(&mut self, _ctx: &mut Context, keycode: event::KeyCode, _keymods: event::KeyMods) {
        match keycode {
            KeyCode::W => self.nes.controller_1.button_state &= !UP,
            KeyCode::A => self.nes.controller_1.button_state &= !LEFT,
            KeyCode::R => self.nes.controller_1.button_state &= !DOWN,
            KeyCode::S => self.nes.controller_1.button_state &= !RIGHT,
            KeyCode::E => self.nes.controller_1.button_state &= !B,
            KeyCode::I => self.nes.controller_1.button_state &= !A,
            KeyCode::LBracket => self.nes.controller_1.button_state &= !SELECT,
            KeyCode::RBracket => self.nes.controller_1.button_state &= !START,
            _ => ()
        }   
    }
}

fn main() {

    let args: Vec<String> = std::env::args().collect();

    if args.is_empty() {panic!("No filename provided");}

    let filename = &args[1];

    let ines_data = std::fs::read(filename).expect("Failed to read rom");

    // If the file isn't long enough to contain ines header, quit
    if ines_data.len() < 16 {
        panic!();
    }
    // If file doesn't contain ines magic number, quit
    if &ines_data[0..4] != b"NES\x1A" {
        panic!();
    }
    
    // Program ROM begins immediately after header
    // Fifth header byte defines size of program ROM in 16KB chunks
    let prg_start: usize = 16;
    let prg_end = prg_start + (ines_data[4] as usize) * 0x4000;

    // Character ROM (sprites, graphics) begins immediately after program rom
    // Sixth header byte defines size of character ROM in 8KB chunks
    let chr_end = prg_end + (ines_data[5] as usize) * 0x2000;

    let cart = hw::Cartridge {
        prg_rom: ines_data[prg_start..prg_end].to_vec(),
        chr_rom: ines_data[prg_end..chr_end].to_vec(),
        mapper: (ines_data[7] & 0xF0) | (ines_data[6] >> 4),
        v_mirroring: (ines_data[6] & 0b0000_0001) != 0,
    };

    println!("prgrom: {}", cart.prg_rom.len());

    let nes = hw::Nes {
        cpu:   Default::default(),
        wram:  [0; 2048],
        ppu:   Default::default(),
        apu: Apu::default(),
        frame: vec![0u8; (WIDTH * HEIGHT * 4) as usize], // *4 because of RGBA
        cart,
        skip: 1,
        old_cpu_state: Cpu::default(),
        old_ppu_state: Ppu::default(),
        jammed: false,
        ppu_log_toggle: false,
        controller_1: Controller::default(),
        controller_2: Controller::default(),
    };

    let emulator = Emulator {
        frames: 0,
        nes,
    };

    let window_mode = WindowMode {
        width: 1024.0,
        height: 960.0,
        ..WindowMode::default()
    };

    // Make a Context.
    let cb = ContextBuilder::new("my_game", "Cool Game Author").window_mode(window_mode);
    let (mut ctx, event_loop) = cb.build().expect("hello");
    graphics::set_default_filter(&mut ctx, graphics::FilterMode::Nearest);
    
    event::run(ctx, event_loop, emulator);

}
