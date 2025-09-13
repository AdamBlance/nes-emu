use super::channels::*;
use crate::nes::mem::read_mem;
use crate::nes::Nes;

const STEP_1: u16 = 3729;
const STEP_2: u16 = 7457;
const STEP_3: u16 = 11186;
const STEP_4: u16 = 14915;
const STEP_5: u16 = 18641;

// I could totally make some linear counter object
// dividers, counters, sequencers are so common here that an "abstract" implementation might be nice

pub fn step_apu(nes: &mut Nes) {
    if nes.cpu.cycles % 2 == 0 {
        clock_frame_sequencer(nes);
        clock_pulse_timer(&mut nes.apu.square1); // why is this possible?
        clock_pulse_timer(&mut nes.apu.square2);
        clock_noise_timer(&mut nes.apu.noise);
    }

    clock_triangle_timer(&mut nes.apu.triangle);
    clock_sample_timer(nes);
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
            if !nes.apu.frame_sequencer_mode_1 {
                if !nes.apu.frame_sequencer_interrupt_inhibit {
                    nes.apu.interrupt_request = true;
                }
                nes.apu.frame_sequencer_counter = 0
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

// I have to figure out how to make this less verbose, probably methods
fn clock_sample_timer(nes: &mut Nes) {
    if nes.apu.sample.curr_timer_value == 0 {
        nes.apu.sample.curr_timer_value = nes.apu.sample.init_timer_value;

        // println!("Bytes remaining {}", nes.apu.sample.remaining_sample_bytes);

        if nes.apu.sample.buffer_bits_remaining == 0
            && nes.apu.sample.remaining_sample_bytes > 0
            && nes.apu.sample.enabled
        {
            // DMC DMA

            let new_sample_data = read_mem(nes.apu.sample.curr_sample_addr, nes);
            // println!("Sample curr addr {:04X}", nes.apu.sample.curr_sample_addr);
            // std::thread::sleep(std::time::Duration::from_millis(5));
            nes.apu.sample.sample_buffer = new_sample_data;
            nes.apu.sample.buffer_bits_remaining = 8;
            // Wrap around 0xC000-0xFFFF
            nes.apu.sample.curr_sample_addr = nes.apu.sample.curr_sample_addr.wrapping_add(1);
            if nes.apu.sample.curr_sample_addr == 0 {
                nes.apu.sample.curr_sample_addr = 0xC000
            }

            nes.apu.sample.remaining_sample_bytes -= 1;
            if nes.apu.sample.remaining_sample_bytes == 0 {
                if nes.apu.sample.loop_sample {
                    nes.apu.sample.curr_sample_addr = nes.apu.sample.init_sample_addr;
                    nes.apu.sample.remaining_sample_bytes = nes.apu.sample.sample_length;
                } else if nes.apu.sample.irq_enabled {
                    nes.apu.sample.interrupt_request = true;
                }
            }
        }

        let delta: i8 = if (nes.apu.sample.sample_buffer & 1) == 1 {
            2
        } else {
            -2
        };
        // This is wrong! It doesn't saturate, just doesn't add the offset if it doesn't fit in the range
        nes.apu.sample.output = nes
            .apu
            .sample
            .output
            .saturating_add_signed(delta)
            .clamp(0, 0x7F);
        nes.apu.sample.sample_buffer >>= 1;
        if nes.apu.sample.buffer_bits_remaining > 0 {
            nes.apu.sample.buffer_bits_remaining -= 1;
        }
    } else {
        nes.apu.sample.curr_timer_value -= 1;
    }
}

fn clock_pulse_timer(sq_wave: &mut Square) {
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

fn clock_triangle_timer(tri: &mut Triangle) {
    if tri.timer_curr_value == 0 {
        // Clock triangle sequencer
        tri.timer_curr_value = tri.timer_init_value;

        // When the period is so small that it creates a piercing sound, which is used to silence
        // the triangle channel on real hardware in many games (silver surfer, megaman),
        // we need to maintain the output but not clock the sequencer to avoid popping
        // If we output 0 instead of maintaing the output, if the sequencer is at its highest output
        // level (0xF), the sound jumps from 0 to F instantly, creating a popping sound
        if tri.linear_counter_curr_value > 0
            && tri.length_counter > 0
            && (tri.timer_init_value > 3)
            && tri.enabled
        {
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

fn clock_triangle_linear_counter(tri: &mut Triangle) {
    if tri.linear_counter_reload_flag {
        tri.linear_counter_curr_value = tri.linear_counter_init_value;
    } else if tri.linear_counter_curr_value > 0 {
        tri.linear_counter_curr_value -= 1;
    }

    // counter reload flag is actually not cleared on clock unconditionally
    // only if the control flag is also clear

    if !tri.length_counter_halt_and_linear_counter_control {
        tri.linear_counter_reload_flag = false;
    }

    tri.linear_counter_mute_signal = tri.linear_counter_curr_value == 0;
}

fn clock_triangle_length_counter(tri: &mut Triangle) {
    if !tri.length_counter_halt_and_linear_counter_control {
        tri.length_counter = tri.length_counter.saturating_sub(1);
    }

    tri.length_counter_mute_signal = tri.length_counter == 0;
}

fn clock_square_envelope(sqw: &mut Square) {
    if sqw.envelope_start_flag {
        sqw.envelope_start_flag = false;
        sqw.envelope_decay_level = 15;
        sqw.envelope_counter_curr_value = sqw.volume_and_envelope_period;
    } else if sqw.envelope_counter_curr_value == 0 {
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
    } else if noise.envelope_counter_curr_value == 0 {
        // clock decay counter
        noise.envelope_counter_curr_value = noise.volume_and_envelope_period;
        // restart the count from 15 if loop is true
        if noise.envelope_decay_level == 0 {
            if noise.envelope_loop_and_length_counter_halt {
                noise.envelope_decay_level = 15;
            }
        } else {
            noise.envelope_decay_level -= 1;
        }
    } else {
        noise.envelope_counter_curr_value -= 1;
    }

    noise.envelope_output = if noise.constant_volume {
        noise.volume_and_envelope_period
    } else {
        noise.envelope_decay_level
        // 0
    };
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

fn clock_square_sweep_counter(sq_wave: &mut Square, twos_compliment: bool) {
    let mut change = (sq_wave.timer_init_value >> sq_wave.sweep_shift_amount) as i16;
    let target = sq_wave.timer_init_value.wrapping_add_signed(change);

    sq_wave.sweep_mute_signal = target > 0b111_11111111;

    if sq_wave.sweep_counter_curr_value == 0 && sq_wave.sweep_enabled && !sq_wave.sweep_mute_signal
    {
        if sq_wave.sweep_negate {
            change *= -1;
            // Pulse 1 only does one's compliment!
            // Have to take 1 off the negative in this case
            if !twos_compliment {
                change -= 1;
            }
        }
        sq_wave.timer_init_value = sq_wave.timer_init_value.wrapping_add_signed(change);
        // duplicate
    }

    sq_wave.sweep_mute_signal |= sq_wave.timer_init_value < 8;

    if sq_wave.sweep_counter_curr_value == 0 || sq_wave.sweep_reload_flag {
        sq_wave.sweep_counter_curr_value = sq_wave.sweep_counter_init_value;
        sq_wave.sweep_reload_flag = false;
    } else {
        sq_wave.sweep_counter_curr_value = sq_wave.sweep_counter_curr_value.saturating_sub(1);
    }
}

fn clock_square_length_counters(sq_wave: &mut Square) {
    if !sq_wave.envelope_loop_and_length_counter_halt {
        sq_wave.length_counter = sq_wave.length_counter.saturating_sub(1);
    }

    sq_wave.length_counter_mute_signal = sq_wave.length_counter == 0;
}

pub fn square_channel_output(sqw: &Square) -> f32 {
    if !sqw.sweep_mute_signal
        && sqw.sequencer_output
        && !sqw.length_counter_mute_signal
        && sqw.enabled
        && (sqw.timer_init_value >= 8)
    {
        sqw.envelope_output as f32
    } else {
        0.0
    }
}

pub fn triangle_channel_output(tri: &Triangle) -> f32 {
    /*
        There is a problem with this. Although not clocking the sequencer when the triangle channel
        is muted prevents popping, it distords the levels of the other channels, since, if the
        sequencer stops at a value of 15, all the other instruments get turned down until
        it resumes. Probably a better way would be to ensure the sequencer always starts at step 0
        whenever the triangle channel turns on, although special handling would need to be done
        for when the music mutes the triangle channel by setting the frequency really high.
    */
    tri.sequencer_output as f32
}

pub fn noise_channel_output(noise: &Noise) -> f32 {
    if noise.sequencer_output && !noise.length_counter_mute_signal && noise.enabled {
        // println!("Noise envelope {}", noise.envelope_output);
        noise.envelope_output as f32
    } else {
        0.0
    }
}

pub fn sample_channel_output(sample: &Sample) -> f32 {
    if sample.enabled {
        sample.output as f32
    } else {
        0.0
    }
}
