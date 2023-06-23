use crate::nes::Nes;
use crate::util::concat_u8;
use crate::mem::read_mem;
use crate::ppu;
use crate::cpu;
use crate::apu::{self, };

pub fn run_to_vblank(nes: &mut Nes) {

    if nes.cpu.cycles == 0 {
        nes.cpu.pc = concat_u8(read_mem(0xFFFD, nes), read_mem(0xFFFC, nes));
        // nes.cpu.pc = 0xC000;
        nes.cpu.cycles = 8;
        nes.ppu.scanline_cycle = 27;
        nes.cpu.p_i = true;
        nes.cpu.s = 0xFD;
        nes.cpu.target = 100;
        // nes.cpu.pause = true;
    }
    
    loop {
        cpu::step_cpu(nes);

        ppu::step_ppu(nes);
        ppu::step_ppu(nes);
        ppu::step_ppu(nes);
        
        apu::step_apu(nes);

        // At cycle mod 40
        if nes.apu.cycles_since_last_sample == nes.apu.target_cycles_per_sample.floor() as u64 {
            // If the number of cycles between samples is too large on average,
            // sample on the 40th cycle
            if nes.apu.average_cycles_per_sample >= nes.apu.target_cycles_per_sample {
                do_sample(nes);
            }
        }
        else if nes.apu.cycles_since_last_sample == nes.apu.target_cycles_per_sample.ceil() as u64 {
            if nes.apu.average_cycles_per_sample < nes.apu.target_cycles_per_sample {
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

// How far to pan the square waves, 0 being no pan (mono)
const STEREO_PAN: f32 = 0.30;

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
    let sq1_output = apu::square_channel_output(&nes.apu.square1) * 1.0;
    let sq2_output = apu::square_channel_output(&nes.apu.square2) * 1.0;
    let tri_output = apu::triangle_channel_output(&nes.apu.triangle) * 1.0;
    let noise = apu::noise_channel_output(&nes.apu.noise) * 1.0;
    let sample = apu::sample_channel_output(&nes.apu.sample) * 1.0;

    let epsilon = 0.00001;
    let pos_bias = 1.0 + STEREO_PAN;
    let neg_bias = 1.0 - STEREO_PAN;
    let pulse1_out = 95.88 / ((8128.0 / (pos_bias*sq1_output + neg_bias*sq2_output + epsilon)) + 100.0);
    let pulse2_out = 95.88 / ((8128.0 / (pos_bias*sq2_output + neg_bias*sq1_output + epsilon)) + 100.0);
    let other_out = 159.79 / ( ( 1.0 / ((tri_output/8227.0) + (noise/12241.0) + (sample/22638.0) + epsilon) ) + 100.0);
        
    let output_val = (pulse1_out + other_out, pulse2_out + other_out);
        
    nes.apu.audio_queue.send(output_val).expect("something wrong happened when appending to audio queue");
}
