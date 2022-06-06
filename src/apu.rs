use crate::{hw::*, util::get_bit};



/*


    Need a safe way to get the audio samples into a buffer that can be safely read by 
    the audio thread. Need a ring buffer I guess? There is a thing called a 
    "lock-free ring buffer" that doesn't use any mutexes, semaphores, etc. 

    https://kmdreko.github.io/posts/20191003/a-simple-lock-free-ring-buffer/

*/


pub fn step_apu(nes: &mut Nes) {
    
    clock_frame_sequencer(nes);

    clock_pulse_timer(nes);

}


const STEP_1: i16 = 3729;
const STEP_2: i16 = 7457;
const STEP_3: i16 = 11186;
const STEP_4: i16 = 14915;
const STEP_5: i16 = 18641;


fn clock_pulse_timer(nes: &mut Nes) {

    /*
        This should tick period+1 times before clocking
        Basically, it includes t, t-1, t-1, ..., 1, 0
        This is t+1 items
        Only clocked when the transition from 0 to to happens, basically when it should be 
        decremented from 0 to -1

        In this first section of the function, the t should be in 0..=t

    */



    if nes.apu.square_1_timer_value == 0 {
        nes.apu.square_1_timer_value = nes.apu.square_1_timer;
    } else {
        nes.apu.square_1_timer_value -= 1;
    }

}



fn clock_pulse_sequencer(nes: &mut Nes) {
    let bit_select = 7 - nes.apu.square_1_sequencer_stage;
    nes.apu.square_1_output = match nes.apu.square_1_duty_cycle {
        0 => get_bit(0b_0_1_0_0_0_0_0_0, bit_select),
        1 => get_bit(0b_0_1_1_0_0_0_0_0, bit_select),
        2 => get_bit(0b_0_1_1_1_1_0_0_0, bit_select),
        3 => get_bit(0b_1_0_0_1_1_1_1_1, bit_select),
        _ => unreachable!(),
    }
}



fn clock_envelope_and_triangle_counter(nes: &mut Nes) {
    
    // if !nes.apu.square_1_envelope_start {
    //     nes.apu.square_1_envelope_divider -= 1;
    // } else {

    // }

}

fn clock_sweep_and_length_counter(nes: &mut Nes) {
    
}


pub fn clock_frame_sequencer(nes: &mut Nes) {
    
    // definitely think this isn't the most efficient way to do this
    // could easily be accomplished with some normal if statements 
    match nes.apu.frame_sequencer_counter {
        
        STEP_1 => clock_envelope_and_triangle_counter(nes),

        STEP_2 => {
            clock_envelope_and_triangle_counter(nes); 
            clock_sweep_and_length_counter(nes);
        }
        
        STEP_3 => clock_envelope_and_triangle_counter(nes),

        STEP_4 => {
            if nes.apu.frame_sequencer_mode_select == false {
                clock_envelope_and_triangle_counter(nes);
                clock_sweep_and_length_counter(nes);

                if !nes.apu.frame_sequencer_interrupt_inhibit {
                    nes.cpu.interrupt_request = true;
                }
                nes.apu.frame_sequencer_counter = -1;
            }
        }

        STEP_5 => {
            clock_envelope_and_triangle_counter(nes);
            clock_sweep_and_length_counter(nes);
            nes.apu.frame_sequencer_counter = -1;
        }

        _ => unreachable!(),

    }

    nes.apu.frame_sequencer_counter += 1;



}