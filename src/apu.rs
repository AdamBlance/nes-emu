use crate::hw::*;

const STEP_1: u16 = 3729;
const STEP_2: u16 = 7457;
const STEP_3: u16 = 11186;
const STEP_4: u16 = 14915;
const STEP_5: u16 = 18641;

const H: bool = true;
const L: bool = false;
static SQUARE_SEQUENCES: [[bool; 8]; 4] = [
    [L, H, L, L, L, L, L, L],  // 12.5% duty
    [L, H, H, L, L, L, L, L],  // 25.0% duty
    [L, H, H, H, H, L, L, L],  // 50.0% duty
    [H, L, L, H, H, H, H, H],  // 75.0% duty
];

static TRIANGLE_SEQUENCE: [u8; 32] = [
    0xF, 0xE, 0xD, 0xC, 0xB, 0xA, 0x9, 0x8, 0x7, 0x6, 0x5, 0x4, 0x3, 0x2, 0x1, 0x0,
    0x0, 0x1, 0x2, 0x3, 0x4, 0x5, 0x6, 0x7, 0x8, 0x9, 0xA, 0xB, 0xC, 0xD, 0xE, 0xF,
];

pub static LENGTH_TABLE: [u8; 32] = [
    0x0A, 0xFE, 0x14, 0x02, 0x28, 0x04, 0x50, 0x06, 
    0xA0, 0x08, 0x3C, 0x0A, 0x0E, 0x0C, 0x1A, 0x0E, 
    0x0C, 0x10, 0x18, 0x12, 0x30, 0x14, 0x60, 0x16, 
    0xC0, 0x18, 0x48, 0x1A, 0x10, 0x1C, 0x20, 0x1E,
];

pub static NOISE_PERIOD_TABLE: [u16; 16] = [
    0x004, 0x008, 0x010, 0x020, 0x040, 0x060, 0x080, 0x0A0, 
    0x0CA, 0x0FE, 0x17C, 0x1FC, 0x2FA, 0x3F8, 0x7F2, 0xFE4,
];


// I could totally make some linear counter object
// dividers, counters, sequencers are so common here that an "abstract" implementation might be nice


pub fn step_apu(nes: &mut Nes) {
    if nes.cpu.cycles % 2 == 0 {
        clock_frame_sequencer(nes);
        clock_pulse_timer(&mut nes.apu.square1);  // why is this possible? 
        clock_pulse_timer(&mut nes.apu.square2);
        clock_noise_timer(&mut nes.apu.noise);
    }

    clock_triangle_timer(&mut nes.apu.triangle);


}



pub fn clock_frame_sequencer(nes: &mut Nes) {
    match nes.apu.frame_sequencer_counter {
        STEP_1 => {
            clock_envelope_and_triangle_counters(nes);
        }
        STEP_2 => {
            clock_envelope_and_triangle_counters(nes); 
            clock_sweep_and_length_counters(nes);
        }
        STEP_3 => {
            clock_envelope_and_triangle_counters(nes);
        }
        STEP_4 => {
            if nes.apu.frame_sequencer_mode_select == false {
                clock_envelope_and_triangle_counters(nes);
                clock_sweep_and_length_counters(nes);
                if !nes.apu.frame_sequencer_interrupt_inhibit {
                    nes.cpu.interrupt_request = true;
                }
                nes.apu.frame_sequencer_counter = 0;
            }
        }
        STEP_5 => {
            clock_envelope_and_triangle_counters(nes);
            clock_sweep_and_length_counters(nes);
            nes.apu.frame_sequencer_counter = 0;
        }
        _ => (),
    }
    nes.apu.frame_sequencer_counter += 1;
}


fn clock_pulse_timer(sq_wave: &mut SquareWave) {
    if sq_wave.timer_curr_value == 0 {
        // Clock pulse sequencer
        sq_wave.timer_curr_value = sq_wave.timer_init_value;
        sq_wave.sequencer_stage = (sq_wave.sequencer_stage + 1) % 8;
        let duty = sq_wave.duty_cycle as usize;
        let stage = sq_wave.sequencer_stage as usize;
        sq_wave.sequencer_output = SQUARE_SEQUENCES[duty][stage];
    } else {
        sq_wave.timer_curr_value -= 1;
    }
}


fn clock_triangle_timer(tri: &mut TriangleWave) {
    if tri.timer_curr_value == 0 {
        // Clock triangle sequencer
        tri.timer_curr_value = tri.timer_init_value;
        
        // this is just a rule apparently
        if tri.linear_counter_curr_value > 0 && tri.length_counter > 0 {
            tri.sequencer_stage = (tri.sequencer_stage + 1) % 32;
        }
        
        
        tri.sequencer_output = TRIANGLE_SEQUENCE[tri.sequencer_stage as usize];
    } else {
        tri.timer_curr_value -= 1;
    }
}


fn clock_envelope_and_triangle_counters(nes: &mut Nes) {

    clock_square_envelope(&mut nes.apu.square1);
    clock_square_envelope(&mut nes.apu.square2);

    clock_noise_envelope(&mut nes.apu.noise);

    clock_triangle_linear_counter(&mut nes.apu.triangle);

}


fn clock_triangle_linear_counter(tri: &mut TriangleWave) {
    
    if tri.linear_counter_reload_flag {
        tri.linear_counter_curr_value = tri.linear_counter_init_value;
    } else {
        if tri.linear_counter_curr_value > 0 {
            tri.linear_counter_curr_value -= 1;
        }
    }

    // counter reload flag is actually not cleared on clock unconditionally
    // only if the control flag is also clear

    if !tri.length_counter_halt_and_linear_counter_control {
        tri.linear_counter_reload_flag = false;
    }

    tri.linear_counter_mute_signal = tri.linear_counter_curr_value == 0;
    // println!("tri linear counter mute {}", tri.length_counter_mute_signal);
    // println!("lin counter {} reload flag {} length halt {}", tri.linear_counter_curr_value, tri.linear_counter_reload_flag, tri.length_counter_halt_and_linear_counter_control);


}


// This is prime Trait territory, can refactor using them once I have the thing working first

fn clock_triangle_length_counter(tri: &mut TriangleWave) {
    if !tri.length_counter_halt_and_linear_counter_control {
        tri.length_counter = tri.length_counter.saturating_sub(1);
    }
    
    tri.length_counter_mute_signal = tri.length_counter == 0;
    // println!("tri length counter mute {}", tri.length_counter_mute_signal);
    // println!("Tri length counter {}", tri.length_counter);
}

/*

Need to make sure I understand this 

There is a start flag that's set when you finish setting the period of the note (write to 0x4003)

There's a divider which is set by the volume bits in 0x4000 (0-15)

There's a decay counter which is a volume value from 0-15. It starts at 15 and gets decremented
when it's clocked by the divider










*/




fn clock_square_envelope(sqw: &mut SquareWave) {
    
    if sqw.envelope_start_flag {
        sqw.envelope_start_flag = false;
        sqw.envelope_decay_level = 15;
        sqw.envelope_counter_curr_value = sqw.volume_and_envelope_period;
    } else {
        if sqw.envelope_counter_curr_value == 0 {
            // clock decay counter
            sqw.envelope_counter_curr_value = sqw.volume_and_envelope_period;
            // restart the count from 15 if loop is true
            if sqw.envelope_decay_level == 0 && sqw.envelope_loop_and_length_counter_halt {
                sqw.envelope_decay_level = 15;
            } else {
                sqw.envelope_decay_level = sqw.envelope_decay_level.saturating_sub(1);
            }
        } else {
            sqw.envelope_counter_curr_value -= 1;
        }
    }

    sqw.envelope_output = if sqw.constant_volume {
        sqw.volume_and_envelope_period
    } else {
        sqw.envelope_decay_level
    };

}






fn clock_noise_envelope(noise: &mut Noise) {
    
    if noise.envelope_start_flag {
        noise.envelope_start_flag = false;
        noise.envelope_decay_level = 15;
        noise.envelope_counter_curr_value = noise.volume_and_envelope_period;
    } else {
        if noise.envelope_counter_curr_value == 0 {
            // clock decay counter
            noise.envelope_counter_curr_value = noise.volume_and_envelope_period;
            // restart the count from 15 if loop is true
            if noise.envelope_decay_level == 0 && noise.envelope_loop_and_length_counter_halt {
                noise.envelope_decay_level = 15;
            } else {
                noise.envelope_decay_level = noise.envelope_decay_level.saturating_sub(1);
            }


            // ????????          Something is wrong with the decay level? 
            // I think it's not decaying fast enougH? 
        } else {
            noise.envelope_counter_curr_value -= 1;
        }
    }

    noise.envelope_output = if noise.constant_volume {
        noise.volume_and_envelope_period
        
    } else {
        // noise.envelope_decay_level
        0
    };

    // println!("decaylevel {}", noise.envelope_decay_level);


}



fn clock_noise_timer(noise: &mut Noise) {
    if noise.timer_curr_value == 0 {
        // Clock pulse sequencer
        noise.timer_curr_value = noise.timer_init_value;
        noise.sequencer_output = fastrand::bool();
    } else {
        noise.timer_curr_value -= 1;
    }
}

fn clock_noise_length_counters(noise: &mut Noise) {
    if !noise.envelope_loop_and_length_counter_halt {
        noise.length_counter = noise.length_counter.saturating_sub(1);
    }
    
    noise.length_counter_mute_signal = noise.length_counter == 0;
}




fn clock_sweep_and_length_counters(nes: &mut Nes) {
    clock_square_length_counters(&mut nes.apu.square1);
    clock_square_length_counters(&mut nes.apu.square2);

    clock_square_sweep_counter(&mut nes.apu.square1, false);
    clock_square_sweep_counter(&mut nes.apu.square2, true);

    clock_triangle_length_counter(&mut nes.apu.triangle);

    clock_noise_length_counters(&mut nes.apu.noise);
}

fn clock_square_sweep_counter(sq_wave: &mut SquareWave, twos_compliment: bool) {

    let mut change = (sq_wave.timer_init_value >> sq_wave.sweep_shift_amount) as i16;
    let target = sq_wave.timer_init_value.wrapping_add_signed(change);

    sq_wave.sweep_mute_signal = target > 0b111_11111111;

    if sq_wave.sweep_counter_curr_value == 0 && sq_wave.sweep_enabled && !sq_wave.sweep_mute_signal {
        if sq_wave.sweep_negate {
            change *= -1;
            // Pulse 1 only does one's compliment!
            // Have to take 1 off the negative in this case
            if !twos_compliment {
                change -= 1;
            }
        }
        sq_wave.timer_init_value = sq_wave.timer_init_value.wrapping_add_signed(change);  // duplicate
    }

    sq_wave.sweep_mute_signal |= sq_wave.timer_init_value < 8;

    if sq_wave.sweep_counter_curr_value == 0 || sq_wave.sweep_reload_flag {
        sq_wave.sweep_counter_curr_value = sq_wave.sweep_counter_init_value;
        sq_wave.sweep_reload_flag = false;
    } else {
        sq_wave.sweep_counter_curr_value = sq_wave.sweep_counter_curr_value.saturating_sub(1);
    }
}

fn clock_square_length_counters(sq_wave: &mut SquareWave) {
    if !sq_wave.envelope_loop_and_length_counter_halt {
        sq_wave.length_counter = sq_wave.length_counter.saturating_sub(1);
    }
    
    sq_wave.length_counter_mute_signal = sq_wave.length_counter == 0;
}

pub fn square_channel_output(sqw: &SquareWave) -> f32 {
    if !sqw.sweep_mute_signal && sqw.sequencer_output && !sqw.length_counter_mute_signal && sqw.enabled {
        sqw.envelope_output as f32
    } else {
        0.0
    }
}

pub fn triangle_channel_output(tri: &TriangleWave) -> f32 {
    if !tri.linear_counter_mute_signal && !tri.length_counter_mute_signal && tri.enabled {
        tri.sequencer_output as f32
    } else {
        0.0
    }
}

pub fn noise_channel_output(noise: &Noise) -> f32 {
    if noise.sequencer_output && !noise.length_counter_mute_signal && noise.enabled {
        noise.envelope_output as f32
    } else {
        0.0
    }
}
