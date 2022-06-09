use core::time;
use std::io;
use std::time::Duration;

use crate::apu;
use crate::hw::*;
use crate::cpu;
use crate::mem::read_mem;
use crate::ppu;
use crate::util::concat_u8;

use std::thread;

const CPU_HZ:   f64 = 1_789_773.0;
const SAMPLE_HZ: f64 = 44_100.5;

pub const TARGET_CYCLES_PER_SAMPLE: f64 = CPU_HZ / SAMPLE_HZ;

const TARGET_CYCLES_PER_SAMPLE_FLOOR: u64 = TARGET_CYCLES_PER_SAMPLE as u64;
const TARGET_CYCLES_PER_SAMPLE_CEIL:  u64 = (TARGET_CYCLES_PER_SAMPLE + 1.0) as u64;


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

        if nes.cpu.cycles % 1000000 == 0 {
            println!("Actual {:.20} Target {:.20}", nes.apu.average_cycles_per_sample, TARGET_CYCLES_PER_SAMPLE);
        }

        // Not entirely sure about this averaging

        // At cycle mod 40
        if nes.apu.cycles_since_last_sample == TARGET_CYCLES_PER_SAMPLE_FLOOR {
            // If the number of cycles between samples is too large on average,
            // sample on the 40th cycle
            if nes.apu.average_cycles_per_sample >= TARGET_CYCLES_PER_SAMPLE {
                do_sample(nes);
            }
        }
        else if nes.apu.cycles_since_last_sample == TARGET_CYCLES_PER_SAMPLE_CEIL {
            if nes.apu.average_cycles_per_sample < TARGET_CYCLES_PER_SAMPLE {
                do_sample(nes);
            }
        }



        // So, after 3 ppu cycles, when reaching end of frame, ppu should land somewhere inside 
        // the 3 cycle range after the frame ends
        // After 3 more ppu cycles, it should leave this range
        // This way, we don't need a bit to say that we've just entered vblank or whatever
        // and we shouldn't skip any frames
        
        nes.apu.cycles_since_last_sample += 1;

        if nes.ppu.scanline == 239 && (nes.ppu.scanline_cycle >= 257 && nes.ppu.scanline_cycle <= 259) {break;}

        // if nes.cpu.instruction_count == target {break;}
    }



}

fn do_sample(nes: &mut Nes) {

    /*
    
    Basically want to do weighted average
    So it'll be
    
    ((average so far * number_of_samples) + cycles since last sample [either 40 or 41]) 
    -----------------------------------------------------------------------------------
                                   number_of_samples + 1
    
    */
    
    let numerator = (nes.apu.average_cycles_per_sample * (nes.apu.total_sample_count as f64)) + (nes.apu.cycles_since_last_sample as f64);
    let denominator = (nes.apu.total_sample_count + 1) as f64;
    
    nes.apu.average_cycles_per_sample = numerator / denominator;

    nes.apu.total_sample_count += 1;
    nes.apu.cycles_since_last_sample = 0;


    // These will be between 0.0 and 15.0
    let sq1_output = apu::square_channel_output(&nes.apu.square1);
    let sq2_output = apu::square_channel_output(&nes.apu.square2);

    let output_val = (sq1_output + sq2_output) / 150.0;
    
    
    
    nes.apu.audio_queue.send(output_val).expect("something wrong happened when appending to audio queue");
}

