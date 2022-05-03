#![feature(bigint_helper_methods)]
#![allow(unused_variables)]
#![allow(dead_code)]
#![allow(unused_parens)]
#![feature(mixed_integer_ops)]

use std::fs;

use ggez::mint::Vector2;
use ggez::{Context, ContextBuilder, GameResult};
use ggez::event::{self, EventHandler};
use ggez::graphics::{self, Color, DrawMode, DrawParam};

mod emu;
mod hw;
mod opc;
mod util;
mod mem;
mod log;
mod cpu; 
mod ppu;

const WIDTH: u32 = 256;
const HEIGHT: u32 = 240;

struct Emulator {
    frames: u64,
    nes: hw::Nes,
}

impl EventHandler<ggez::GameError> for Emulator {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        emu::run_to_vblank(&mut self.nes);
        println!("Frames - {}", self.frames);

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        self.frames += 1;
        // if (self.frames % 60) == 0 {
        //     println!("FPS: {}", ggez::timer::fps(ctx));
        // }

        let image = graphics::Image::from_rgba8(ctx, 256, 240, &self.nes.frame)?;
        // let image = graphics::Image::solid(ctx, 240, Color::from_rgb(255, 0, 255))?;
        let dp = DrawParam::new();
        graphics::draw(ctx, &image, dp)?;
        graphics::present(ctx)?;
        Ok(())
    }
}

fn main() {

    let ines_data = fs::read("color_test.nes").expect("Failed to read rom");

    if ines_data.len() < 16 {
        panic!();
    }
    if &ines_data[0..4] != b"NES\x1A" {
        panic!();
    }
    
    let prg_start: usize = 16;
    let prg_end = prg_start + (ines_data[4] as usize) * 16384;
    println!("prg_end size {}", prg_end-prg_start);
    let chr_end = prg_end + (ines_data[5] as usize) * 8192;



    let cart = hw::Cartridge {
        prg_rom: ines_data[prg_start..prg_end].to_vec(),
        chr_rom: ines_data[prg_end..chr_end].to_vec(),
        mapper: (ines_data[7] & 0xF0) | (ines_data[6] >> 4),
        v_mirroring: (ines_data[6] & 0b0000_0001) != 0,
    };

    let mut frame = vec![0u8; 256*240*4];

    let nes = hw::Nes {
        cpu: Default::default(),
        wram: [0; 2048],
        ppu: Default::default(),
        ppu_written_to: false,
        frame,
        cart,
        skip: 1,
    };

    let emulator = Emulator {
        frames: 0,
        nes,
    };

    // Make a Context.
    let (mut ctx, event_loop) = ContextBuilder::new("my_game", "Cool Game Author").build().expect("Something went wrong");
    event::run(ctx, event_loop, emulator);




}