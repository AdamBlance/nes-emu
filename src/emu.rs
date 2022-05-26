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

    let parsed_input = input_string.trim().parse::<u64>().unwrap_or(1);

    let target: u64 = nes.cpu.instruction_count + parsed_input;

    if input_string.trim() == "vram" {
        println!("{:02X?}", &nes.ppu.vram);
    }
    if input_string.trim() == "toggle ppu output" {
        nes.ppu_log_toggle = !nes.ppu_log_toggle;
    }

    loop {
        cpu::step_cpu(nes);
        ppu::step_ppu(nes);
        ppu::step_ppu(nes);
        ppu::step_ppu(nes);

        // if (nes.ppu.scanline == 241 && nes.ppu.scanline_cycle == 1) {break;}

        if nes.cpu.instruction_count == target {break;}
    }




}
