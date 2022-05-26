#![feature(bigint_helper_methods)]
#![feature(mixed_integer_ops)]

use ggez::conf::WindowMode;
use ggez::mint::{Point2, Vector2};
use ggez::{Context, ContextBuilder, GameResult};
use ggez::event::{self, EventHandler};
use ggez::graphics::{self, DrawParam, Transform};

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

const WIDTH: u32 = 256;
const HEIGHT: u32 = 256;

struct Emulator {
    frames: u64,
    nes: hw::Nes,
}

impl EventHandler<ggez::GameError> for Emulator {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        emu::run_to_vblank(&mut self.nes);


        // if self.nes.jammed {
            // let mut sound = Source::new(_ctx, "/jam.mp3")?;

            // sound.play_detached(_ctx)?;

            // self.nes.jammed = false;
        // }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        self.frames += 1;


        let image = graphics::Image::from_rgba8(ctx, WIDTH as u16, HEIGHT as u16, &self.nes.frame)?;

        let scalev = Vector2{x: 4.0, y: 4.0};
        let destv = Point2{x: 0.0, y: 0.0};
        let offsetv = Point2{x: 0.0, y: 0.0};
        let trans = Transform::Values{dest: destv, rotation: 0.0, scale: scalev, offset: offsetv};
        let mut dp = DrawParam::new();
        dp.trans = trans;
        graphics::draw(ctx, &image, dp)?;
        graphics::present(ctx)?;
        Ok(())
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
        frame: vec![0u8; (WIDTH * HEIGHT * 4) as usize], // *4 because of RGBA
        cart,
        skip: 1,
        old_cpu_state: Cpu::default(),
        old_ppu_state: Ppu::default(),
        jammed: false,
        ppu_log_toggle: false,
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
