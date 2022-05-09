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

// This file is a bit unneccesary, might just pull this out into main

// so, I'm going to do the cycle accurate thing I guess
// I know just-in-time is meant to be better for cache but modern cache is massive so it'll be fine

/*

    so, need to run CPU for 1 cycle, then ppu for 3 cycles
    will just make the instructions commit on the last cycle, yeah, that'll be fine
    it's not like the ppu can read ram anyway, so it can't catch an instruction half way done


    cpu will only move 1 cycle at a time, so will need to figure out what instruction is currently
    being processed, and just tick away until the last cycle is reached

    okay, so the number of cycles an instruction takes varies

    most instructions take a set number
    some take +1 cycles if a page boundary is crossed when accessing memory
    branches take +1 when jumping within a page, and +2 when jumping to another page

    save cycles in a field in the cpu struct 
    decrement by one, since a call to tick_cpu should take one cycle
    all instructions are at least 2 cycles long, so this is fine

    next time we enter cpu_tick, check cycles
    if there is 1 cycle left, execute the instruction and decrement

    so, if the instruction has page_penalty set, or is a branch, now that we have
    instr_addr and instr_val, we can figure out if we need to add more cycles

    no, why am i doing this? this is stupid

    just execute all instructions on the first cycle! 
    then get the cycles, figure out any penalty cycles depending on branches or addressing
    
    add them, if any, and just wait out the rest of the cycles
    don't need a flag to say whether there is an instruction "running", just check if the cycle
    count is 0 or not. when it's 0, it's fine to grab the next instruction
    






    alright, just looked it up, cycles are actually quite complicated 
    https://www.nesdev.com/6502_cpu.txt
    https://forums.nesdev.org/viewtopic.php?t=14359

    so basically, instructions take cycles to execute
    each cycle has a specific purpose 

    since PPU and mapper can't read CPU registers, cycle accuracy doesn't matter for cpu register stuff

    cycle accuracy does matter, on the other hand, for memory writes
    mapper and PPU can see these, and since PPU renders 3 pixels for every cpu cycle, a 7 cycle instruction
    spans like 21 pixels, which is quite a lot of variance if something changes in memory that affects
    something in VRAM or v or t or something

    so basically, for memory reads and writes, try to do everything on the correct cycle!
    that document above describes what exactly happens on each cycle


    fetch opcode + increment pc is always the first cycle

    a few easy ones first:

    Accumulator instructions are all 2 cycles; first one is fetch opc, next is a dummy read of next byte 
    
    Immediate instructions are all 2; opcode fetch, value fetch (all do stuff on accumulnator)

    absolute addressing 
        3 JMP - opc, addr lsb, addr msb 

        4 read instructions (ld_, logic, add, sub, cmp, etc)
        opc, lsb, msb, mem read

        6 rmw instructions (asl, lsr, rol, ror, inc, dec, etc.)
        opc, lsb, msb, mem read, write the value back untouched?, write the new value

        4 write instructions (store)
        opc, lsb, msb, write
    
    zero page
        reads
        opc, lsb, read

        rmw
        opc, lsb, read, write, write

        write
        opc, lsb, write
    
    zero 

    at the absolute minimum, there will be an opcode read first, followed by a byte1 read 








*/


pub fn run_to_vblank(nes: &mut Nes) {

    if nes.cpu.cycles == 0 {
        // nes.cpu.pc = read_mem_u16(0xFFFC, nes);
        nes.cpu.pc = 0xC000;
        // alright, apparently interrupts take 7 cycles to process 
        // https://forums.nesdev.org/viewtopic.php?t=18570
        nes.cpu.cycles = 7;
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
