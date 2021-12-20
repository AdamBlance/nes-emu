#![feature(bigint_helper_methods)]
#![allow(unused_variables)]
#![allow(dead_code)]
#![allow(unused_parens)]
#![feature(mixed_integer_ops)]

use std::fs;
mod emu;
mod hw;
mod opc;


fn main() {

    let ines_data = fs::read("rom.nes").expect("Failed to read rom");

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
    };

    emu::emulate(&cart);

}