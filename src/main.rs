#![feature(bigint_helper_methods)]
#![allow(unused_variables)]
#![allow(dead_code)]
#![allow(unused_parens)]
#![feature(mixed_integer_ops)]

use pixels::{Error, Pixels, SurfaceTexture};
use winit::{
    dpi::LogicalSize,
    event::{Event, VirtualKeyCode},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use winit_input_helper::WinitInputHelper;

use std::fs;
// mod emu;
mod hw;
mod opc;
mod util;

const WIDTH: u32 = 256;
const HEIGHT: u32 = 240;

fn main() {

    let ines_data = fs::read("nestest.nes").expect("Failed to read rom");

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

    println!("size is {}B", std::mem::size_of_val(&opc::INSTRUCTION_INFO));



    emu::nes_main_loop(cart);


    // let event_loop = EventLoop::new();
    // let mut input = WinitInputHelper::new();

    // let window = {
    //     let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
    //     let scaled_size = LogicalSize::new(WIDTH as f64 * 3.0, HEIGHT as f64 * 3.0);
    //     WindowBuilder::new()
    //         .with_title("Balls")
    //         .with_inner_size(scaled_size)
    //         .with_min_inner_size(size)
    //         .build(&event_loop)
    //         .unwrap()
    // };

    // let mut pixels = {
    //     let window_size = window.inner_size();
    //     let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
    //     Pixels::new(WIDTH, HEIGHT, surface_texture)?
    // };

    // event_loop.run(move |event, _, control_flow| {
    //     // The one and only event that winit_input_helper doesn't have for us...
    //     if let Event::RedrawRequested(_) = event {
    //         make_red(pixels.get_frame());
    //         if pixels
    //             .render()
    //             .map_err(|e| panic!("pixels.render() failed: {}", e))
    //             .is_err()
    //         {
    //             *control_flow = ControlFlow::Exit;
    //             return;
    //         }
    //     }

    //     if input.update(&event) {
    //         window.request_redraw();
    //     }
    // });

}

// fn make_red(pix: &mut [u8]) {
//     for (i, pixel) in pix.chunks_exact_mut(4).enumerate() {
//         if i % 2 == 0 {
//             pixel[0] = 0xff; // R
//             pixel[1] = 0xaa; // G
//             pixel[2] = 0x00; // B
//             pixel[3] = 0xff; // A
//         }
//     }
// }