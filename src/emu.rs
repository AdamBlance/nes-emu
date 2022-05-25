use std::io;

use crate::hw::*;
use crate::cpu;
use crate::mem::read_mem;
use crate::ppu;
use crate::util::concat_u8;


pub fn run_to_vblank(nes: &mut Nes) {

    if nes.cpu.cycles == 0 {
        // nes.cpu.pc = read_mem_u16(0xFFFC, nes);
        nes.cpu.pc = concat_u8(read_mem(0xFFFD, nes), read_mem(0xFFFC, nes));
        // alright, apparently interrupts take 7 cycles to process 
        // https://forums.nesdev.org/viewtopic.php?t=18570
        nes.cpu.cycles = 7;
        nes.ppu.scanline_cycle = 21;
        nes.cpu.p_i = true;
        nes.cpu.s = 0xFD;
    }
    
    let mut input_string = String::new();
    io::stdin().read_line(&mut input_string).unwrap();

    let parsed_input = input_string.trim_end().parse::<u64>().unwrap_or(1);

    let target: u64 = nes.cpu.instruction_count + parsed_input;

    loop {
        cpu::step_cpu(nes);
        ppu::step_ppu(nes);
        ppu::step_ppu(nes);
        ppu::step_ppu(nes);

        // if (nes.ppu.scanline == 241 && nes.ppu.scanline_cycle == 1) {break;}
        // break;
        if parsed_input == 2 {
            for y in 0..=29 {
                println!("{:?}", &nes.ppu.vram[(0x400 + y*30)..(0x400 + y*30 + 32)]);
            }
        }
        if parsed_input == 3 {
            for y in 0..=29 {
                println!("{:?}", &nes.ppu.vram[(0x000 + y*30)..(0x000 + y*30 + 32)]);
            }
        }
        if (nes.cpu.instruction_count == target) {break;}
    }




}
