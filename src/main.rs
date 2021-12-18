#![feature(bigint_helper_methods)]
#![allow(unused_variables)]
#![allow(dead_code)]
#![allow(unused_parens)]

use std::fs;

// mod emu;



fn main() {

    let x: u8 = 0b11001110;
    println!("unsigned u8 -> {}", x);

    let y: i8 = (x as i8);
    println!("unsigned u8 to i8 -> {}", y);

    // let ines_data = fs::read("rom.nes").expect("Failed to read rom");

    // if ines_data.len() < 16 {
    //     panic!();
    // }
    // if &ines_data[0..4] != b"NES\x1A" {
    //     panic!();
    // }
    
    // let prg_start: usize = 16;
    // let prg_end = prg_start + (ines_data[4] as usize) * 16384;
    // let chr_end = prg_end + (ines_data[5] as usize) * 8192;

    // let cart = Cartridge {
    //     prg_rom: ines_data[prg_start..prg_end].to_vec(),
    //     chr_rom: ines_data[prg_end..chr_end].to_vec(),
    //     mapper: (ines_data[7] & 0xF0) | (ines_data[6] >> 4),
    // };

    // emu::emulate(&cart);

}