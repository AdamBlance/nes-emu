use crate::nes::Nes;
use crate::util::concat_u8;
use crate::mem::read_mem;
use crate::ppu;
use crate::cpu;
use crate::apu::{self, };

const CPU_HZ:   f64 = 1_789_773.0;
const SAMPLE_HZ: f64 = 48_000.0;

pub const TARGET_CYCLES_PER_SAMPLE: f64 = CPU_HZ / SAMPLE_HZ;

const TARGET_CYCLES_PER_SAMPLE_FLOOR: u64 = TARGET_CYCLES_PER_SAMPLE as u64;
const TARGET_CYCLES_PER_SAMPLE_CEIL:  u64 = (TARGET_CYCLES_PER_SAMPLE + 1.0) as u64;


pub fn run_to_vblank(nes: &mut Nes) {

    if nes.cpu.cycles == 0 {
        nes.cpu.pc = concat_u8(read_mem(0xFFFD, nes), read_mem(0xFFFC, nes));
        nes.cpu.cycles = 7;
        nes.ppu.scanline_cycle = 21;
        nes.cpu.p_i = true;
        nes.cpu.s = 0xFD;
        nes.apu.sample.init_timer_value = 428;
    }
    

    loop {
        cpu::step_cpu(nes);

        ppu::step_ppu(nes);
        ppu::step_ppu(nes);
        ppu::step_ppu(nes);
        
        apu::step_apu(nes);


        if nes.cpu.cycles % 1000000 == 0 {
            // println!("Actual {:.20} Target {:.20}", nes.apu.average_cycles_per_sample, TARGET_CYCLES_PER_SAMPLE);
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
    let tri_output = apu::triangle_channel_output(&nes.apu.triangle);
    let noise = apu::noise_channel_output(&nes.apu.noise);
    let sample = apu::sample_channel_output(&nes.apu.sample);

    

    let output_val = (sq1_output + sq2_output + tri_output + noise + (sample / 2.3)) / 150.0;
    // let output_val = (sample) / 150.0;
    
    
    
    nes.apu.audio_queue.send(output_val).expect("something wrong happened when appending to audio queue");
}

