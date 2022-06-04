use crate::hw::*;


pub fn step_apu(nes: &mut Nes) {
    
    

}


const STEP_1: u16 = 3729;
const STEP_2: u16 = 7457;
const STEP_3: u16 = 11186;
const STEP_4: u16 = 14915;
const STEP_5: u16 = 18641;


fn clock_envelope_and_triangle_counter(nes: &mut Nes) {
    
}

fn clock_sweep_and_length_counter(nes: &mut Nes) {
    
}


pub fn clock_frame_sequencer(nes: &mut Nes) {
    
    match nes.apu.sequencer_counter {
        
        STEP_1 => {
            clock_envelope_and_triangle_counter(nes);
        }

        STEP_2 => {
            clock_envelope_and_triangle_counter(nes);
            clock_sweep_and_length_counter(nes);
        }
        
        STEP_3 => {
            clock_envelope_and_triangle_counter(nes);
        }

        STEP_4 => {
            // need to implement normal interrupt!
        }




        _ => unreachable!(),
    }

}