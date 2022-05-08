use std::io;

use crate::hw::*;
use crate::mem::*;
use crate::cpu;
use crate::ppu;

// PPU shouldn't be touched until 29658 CPU cycles have passed.
// Some registers will work immediately on start up, like PPUSTATUS
// Constantly checking PPUSTATUS bit-7 VBLANK flag with BIT instruction in a loop
// will let 27,000 odd cycles pass
// Doing it again will let 50,000 odd cycles pass
// By this point, everything should work

// PPU needs to report how many cycles are left until it will raise VBLANK NMI
// CPU should run until it either affects the PPU, has run up to VBLANK as reported by PPU
// On NTSC, 1 CPU tick to 3 PPU ticks

// 341 cycles per scanline
// 262 scanlines


pub fn run_to_vblank(nes: &mut Nes) {

    if nes.cpu.cycles == 0 {
        // nes.cpu.pc = read_mem_u16(0xFFFC, nes);
        nes.cpu.pc = 0xC000;
        nes.cpu.p_i = true;
        nes.cpu.s = 0xFD;
    }
    
    let mut input_string = String::new();
    io::stdin().read_line(&mut input_string).unwrap();

    let target: u64 = nes.cpu.instruction_count + input_string.trim_end().parse::<u64>().unwrap_or(1);

    loop {
        cpu::step_cpu(nes);

        ppu::step_ppu(nes);
        ppu::step_ppu(nes);
        ppu::step_ppu(nes);

        // if (nes.ppu.scanline == 241 && nes.ppu.scanline_cycle == 1) {break;}
        // break;
        if (nes.cpu.instruction_count == target) {break;}
    }


}
