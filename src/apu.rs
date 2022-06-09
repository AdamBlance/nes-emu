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

pub static LENGTH_TABLE: [u8; 32] = [
    0x0A, 0xFE, 0x14, 0x02, 0x28, 0x04, 0x50, 0x06, 
    0xA0, 0x08, 0x3C, 0x0A, 0x0E, 0x0C, 0x1A, 0x0E, 
    0x0C, 0x10, 0x18, 0x12, 0x30, 0x14, 0x60, 0x16, 
    0xC0, 0x18, 0x48, 0x1A, 0x10, 0x1C, 0x20, 0x1E,
];



pub fn step_apu(nes: &mut Nes) {
    clock_frame_sequencer(nes);
    clock_pulse_timer(&mut nes.apu.square1);  // why is this possible? 
    clock_pulse_timer(&mut nes.apu.square2);

}



pub fn clock_frame_sequencer(nes: &mut Nes) {
    match nes.apu.frame_sequencer_counter {
        STEP_1 => {
            clock_envelope_and_triangle_counter(nes);
        }
        STEP_2 => {
            clock_envelope_and_triangle_counter(nes); 
            clock_sweep_and_length_counters(nes);
        }
        STEP_3 => {
            clock_envelope_and_triangle_counter(nes);
        }
        STEP_4 => {
            if nes.apu.frame_sequencer_mode_select == false {
                clock_envelope_and_triangle_counter(nes);
                clock_sweep_and_length_counters(nes);
                if !nes.apu.frame_sequencer_interrupt_inhibit {
                    nes.cpu.interrupt_request = true;
                }
                nes.apu.frame_sequencer_counter = 0;
            }
        }
        STEP_5 => {
            clock_envelope_and_triangle_counter(nes);
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
        sq_wave.output = SQUARE_SEQUENCES[duty][stage];
    } else {
        sq_wave.timer_curr_value -= 1;
    }
}



fn clock_envelope_and_triangle_counter(nes: &mut Nes) {}



fn clock_sweep_and_length_counters(nes: &mut Nes) {
    clock_square_length_counters(&mut nes.apu.square1);
    clock_square_length_counters(&mut nes.apu.square2);

    clock_square_sweep_counter(&mut nes.apu.square1, false);
    clock_square_sweep_counter(&mut nes.apu.square2, true);
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
        sq_wave.timer_init_value = target;
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
    if sq_wave.length_counter == 0 {
        sq_wave.mute = true;
    }
}




