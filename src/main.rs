#![feature(bigint_helper_methods)]
#![allow(unused_variables)]
#![allow(dead_code)]
#![allow(unused_parens)]
#![feature(mixed_integer_ops)]

use std::{fs, io};

use ggez::conf::WindowMode;
use ggez::mint::{Point2, Vector2};
use ggez::{Context, ContextBuilder, GameResult};
use ggez::event::{self, EventHandler};
use ggez::graphics::{self, DrawParam, Transform};
use ggez::audio::{Source, SoundSource};
use util::get_bit;
use crate::hw::*;

mod emu;
mod hw;
mod instr_defs;
mod util;
mod mem;
mod cpu; 
mod ppu;
mod outfile;
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


        if self.nes.jammed {
            let mut sound = Source::new(_ctx, "/jam.mp3")?;

            sound.play_detached(_ctx)?;

            self.nes.jammed = false;
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        self.frames += 1;

        let mut new_frame = vec![0u8; (256*128*4) as usize];

 
        // for y in 0u32..=127 {
        //     for x in 0u32..=255 {
        //         // let mut input_string = String::new();
        //         // io::stdin().read_line(&mut input_string).unwrap();
        //         // let nametable_entry = self.nes.ppu.vram[((y/8)*32 + (x/8)) as usize] as u32;

        //         // let nametable_entry = 

        //         let offset = if x > 127 {256} else {0};

        //         let ptable_index = (y/8)*16 + ((x/8)%16) + offset;

        //         // println!("current nametable entry {}", ptable_index);

        //         let pattern_lsb_byte = self.nes.cart.chr_rom[(ptable_index*16 + (y%8)) as usize];
        //         let pattern_msb_byte = self.nes.cart.chr_rom[((ptable_index*16) + (y%8) + 8) as usize];
        //         let pattern_lsb = get_bit(pattern_lsb_byte, (7 - (x%8)) as u8) as u8;
        //         let pattern_msb = get_bit(pattern_msb_byte, (7 - (x%8)) as u8) as u8;

        //         let this_pixel = ((pattern_msb << 1) | pattern_lsb) & 0b00000011;
                
                

        //         let frame_index: usize = ((y*256 + x)*4) as usize;

        //         // println!("x {} y {} shift {} lsb {:08b} lsb_byte {:08b} msb {:08b} msb_byte {:08b} this_pixel {:08b}", x, y, 7 - (x%8), pattern_lsb, pattern_lsb_byte, pattern_msb, pattern_msb_byte, this_pixel);

        //         /*
                
        //             Going to figure out the layout of the pattern tables
        //             The default mappers have 8kB of chr rom 
        //             8kB is 0x2000 (this is where the nametables start)

        //             One tile is 16B (8B + 8B), so the CHR rom can store 

        //             that is 512 tiles in total

        //             Each tile is 8*8 pixels

        //             pattern table is best represented as two 16*16 tile squares
        //             this is two 128*128 pixel squares
                    
                
        //         */

        //         let colour = match this_pixel {
        //             0b00 => {
        //                 new_frame[frame_index    ] =  76;  // R
        //                 new_frame[frame_index + 1] = 154;  // G
        //                 new_frame[frame_index + 2] = 236;  // B
        //                 new_frame[frame_index + 3] = 255;  // A
        //             }
        //             0b01 => {
        //                 new_frame[frame_index    ] =  92;  // R92  30 228
        //                 new_frame[frame_index + 1] =  30;  // G
        //                 new_frame[frame_index + 2] = 228;  // B
        //                 new_frame[frame_index + 3] = 255;  // A
        //             }
        //             0b10 => {
        //                 new_frame[frame_index    ] =  116;  // R116 196   0   
        //                 new_frame[frame_index + 1] =  196;  // G
        //                 new_frame[frame_index + 2] = 0;  // B
        //                 new_frame[frame_index + 3] = 255;  // A
        //             }
        //             0b11 => {
        //                 new_frame[frame_index    ] =  0;  // R116 196   0   
        //                 new_frame[frame_index + 1] =  30;  // G
        //                 new_frame[frame_index + 2] = 116;  // B
        //                 new_frame[frame_index + 3] = 255;  // A
        //             }
        //             _ => unreachable!(),
        //         };


        //     }
        // }

        // let image = graphics::Image::from_rgba8(ctx, 256, 128, &new_frame)?;

        let image = graphics::Image::from_rgba8(ctx, WIDTH as u16, HEIGHT as u16, &self.nes.frame)?;

        let scalev = Vector2{x: 3., y: 3.};
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

    let ines_data = fs::read("donkeykong.nes").expect("Failed to read rom");

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
        width: 1000.0,
        height: 1000.0,
        ..WindowMode::default()
    };

    // Make a Context.
    let (mut ctx, event_loop) = ContextBuilder::new("my_game", "Cool Game Author").window_mode(window_mode).build().expect("Something went wrong");
    
    event::run(ctx, event_loop, emulator);

}
