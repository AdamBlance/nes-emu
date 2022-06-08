use std::io;

use crate::apu;
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
    
    // let mut input_string = String::new();
    // io::stdin().read_line(&mut input_string).unwrap();

    // let parsed_input = input_string.trim().parse::<u64>().unwrap_or(1);

    // let target: u64 = nes.cpu.instruction_count + parsed_input;

    // if input_string.trim() == "vram" {
    //     println!("{:02X?}", &nes.ppu.vram);
    // }
    // if input_string.trim() == "toggle ppu output" {
    //     nes.ppu_log_toggle = !nes.ppu_log_toggle;
    // }

    loop {
        cpu::step_cpu(nes);
        ppu::step_ppu(nes);
        ppu::step_ppu(nes);
        ppu::step_ppu(nes);

        if nes.cpu.cycles % 2 == 0 {
            apu::step_apu(nes);
        }

        if nes.cpu.cycles % 40 == 0 {
            // https://github.com/RustAudio/cpal/blob/master/examples/synth_tones.rs
            nes.apu.audio_queue.send((nes.apu.square_1_output as u32 as f32) * 10.0 * (nes.apu.square_1_volume_and_envelope_period as f32)).expect("something wrong happened when appending to audio queue");
        }

        // So, after 3 ppu cycles, when reaching end of frame, ppu should land somewhere inside 
        // the 3 cycle range after the frame ends
        // After 3 more ppu cycles, it should leave this range
        // This way, we don't need a bit to say that we've just entered vblank or whatever
        // and we shouldn't skip any frames
        
        if nes.ppu.scanline == 239 && (nes.ppu.scanline_cycle >= 257 && nes.ppu.scanline_cycle <= 259) {break;}

        // if nes.cpu.instruction_count == target {break;}
    }




}
