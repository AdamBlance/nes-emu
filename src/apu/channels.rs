
use super::units::{EnvelopeGenerator, SweepUnit, LengthCounter, LinearCounter};
use crate::util::get_bit;
use crate::mem::read_mem;

static SAMPLE_RATE_TABLE: [u16; 16] = [
    428, 380, 340, 320, 286, 254, 226, 214, 
    190, 160, 142, 128, 106,  84,  72,  54,
];

static NOISE_PERIOD_TABLE: [u16; 16] = [
    0x004, 0x008, 0x010, 0x020, 0x040, 0x060, 0x080, 0x0A0, 
    0x0CA, 0x0FE, 0x17C, 0x1FC, 0x2FA, 0x3F8, 0x7F2, 0xFE4,
];

static TRIANGLE_SEQUENCE: [u8; 32] = [
    15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0,
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15,
];

const H: bool = true;
const L: bool = false;
static SQUARE_SEQUENCES: [[bool; 8]; 4] = [
    [L, H, L, L, L, L, L, L],  // 12.5% duty
    [L, H, H, L, L, L, L, L],  // 25.0% duty
    [L, H, H, H, H, L, L, L],  // 50.0% duty
    [H, L, L, H, H, H, H, H],  // 75.0% duty
];










#[derive(Copy, Clone, Default)]
pub struct Square {
    pub envelope_generator: EnvelopeGenerator,
    pub sweep_unit: SweepUnit, 
    pub length_counter: LengthCounter,

    timer: u16,

    duty_cycle: u8,
    sequencer_stage: u8,
}
impl Square {

    pub fn clock_period_timer(&mut self) {
        if self.timer > 0 {
            self.timer -= 1;
        } else {
            self.timer = self.sweep_unit.get_timer_period();
            self.sequencer_stage = (self.sequencer_stage + 1) % 8;
        }
    }

    pub fn set_reg1_from_byte(&mut self, byte: u8) {
        self.duty_cycle = (byte & 0b1100_0000) >> 6;

        self.length_counter.set_halt_flag(get_bit(byte, 5));
        self.envelope_generator.configure_with_byte(byte);
    }
    pub fn set_reg2_from_byte(&mut self, byte: u8) {
        self.sweep_unit.configure_with_byte(byte);
    }
    pub fn set_reg3_from_byte(&mut self, byte: u8) {
        let val = (self.sweep_unit.get_timer_period() & 0b111_0000_0000) | byte as u16;
        self.sweep_unit.set_timer_period(val)
    }
    pub fn set_reg4_from_byte(&mut self, byte: u8) {
        let val = (self.sweep_unit.get_timer_period() & 0b000_1111_1111) 
                | (byte as u16 & 0b0000_0111) << 8;

        self.sweep_unit.set_timer_period(val);
        self.sequencer_stage = 0;
        
        self.length_counter.configure_with_byte(byte);        
        self.envelope_generator.set_start_flag();
    }
}




#[derive(Copy, Clone, Default)]
pub struct Triangle {
    pub linear_counter: LinearCounter,
    pub length_counter: LengthCounter,

    timer_reload: u16,
    timer: u16,

    sequencer_stage: u8,
}
impl Triangle {

    pub fn clock_period_timer(&mut self) {
        if self.timer > 0 {
            self.timer -= 1;
        } else {
            self.timer = self.timer_reload;
            if !self.linear_counter.get_mute_flag() && !self.length_counter.get_mute_flag() && self.timer_reload > 2 {
                self.sequencer_stage = (self.sequencer_stage + 1) % 32;
            }
        }
    }

    pub fn set_reg1_from_byte(&mut self, byte: u8) {
        self.linear_counter.configure_with_byte(byte);
        self.length_counter.set_halt_flag(get_bit(byte, 7));
    }
    pub fn set_reg2_from_byte(&mut self, byte: u8) {
        self.timer_reload &= 0b111_0000_0000;
        self.timer_reload |= byte as u16;
    }
    pub fn set_reg3_from_byte(&mut self, byte: u8) {
        self.timer_reload &= 0b000_1111_1111;
        self.timer_reload |= ((byte as u16) & 0b111) << 8;

        self.length_counter.configure_with_byte(byte);

        self.linear_counter.set_reload_flag();
    }
}




#[derive(Copy, Clone, Default)]
pub struct Noise {
    pub envelope_generator: EnvelopeGenerator,
    pub length_counter: LengthCounter,

    timer_reload: u16,
    timer: u16,

    shift_reg_output: bool,
}
impl Noise {

    pub fn clock_period_timer(&mut self) {
        if self.timer > 0 {
            self.timer -= 1;
        } else {
            self.timer = self.timer_reload;
            self.shift_reg_output = fastrand::bool();
        }
    }

    pub fn set_reg1_from_byte(&mut self, byte: u8) {
        self.length_counter.set_halt_flag(get_bit(byte, 5));
        self.envelope_generator.configure_with_byte(byte);
    }
    pub fn set_reg2_from_byte(&mut self, byte: u8) {
        self.timer_reload = NOISE_PERIOD_TABLE[(byte & 0b0000_1111) as usize];
    }
    pub fn set_reg3_from_byte(&mut self, byte: u8) {
        self.length_counter.configure_with_byte(byte);
        self.envelope_generator.set_start_flag()
    }
}

#[derive(Copy, Clone, Default)]
pub struct Sample {
    pub irq_enabled: bool,
    pub loop_sample: bool,
    pub timer_reload: u16,
    pub timer: u16,

    // 0x4011 - Writes directly to sample channel output
    // Used to playback PCM audio with full 7 bit samples

    // 0x4012 - Sample address
    // C000 + (addr * 64)

    // 0x4013 - Sample length in bytes
    // (length * 16) + 1 bytes

    pub sample_buffer: u8,
    pub buffer_bits_remaining: u8,
    pub sample_length: u16,
    pub remaining_sample_bytes: u16,
    pub init_sample_addr: u16,
    pub curr_sample_addr: u16,

    pub mute_signal: bool,
    pub output: u8,

    pub interrupt_request: bool,
}
impl Sample {

    pub fn clock_period_timer(&mut self) {
        if self.timer > 0 {
            self.timer -= 1;
        } else {
            self.timer = self.timer_reload;
            self.clock_sample_logic();
        }
    }

    pub fn set_reg1_from_byte(&mut self, byte: u8) {
        self.irq_enabled      = (byte & 0b1000_0000) > 0;
        if !self.irq_enabled {self.interrupt_request = false}
        self.loop_sample      = (byte & 0b0100_0000) > 0;
        self.timer_reload = SAMPLE_RATE_TABLE[(byte & 0b0000_1111) as usize];
    }
    pub fn set_reg2_from_byte(&mut self, byte: u8) {
        // println!("Written {:08b} enabled {}", byte, self.enabled);
        self.output = byte & 0b0111_1111;
    }
    pub fn set_reg3_from_byte(&mut self, byte: u8) {
        self.init_sample_addr = 0xC000 + (byte as u16 * 64) as u16;
    }
    pub fn set_reg4_from_byte(&mut self, byte: u8) {
        self.sample_length = (byte as u16 * 16) + 1;
    }

    fn clock_sample_timer(&mut self) {
        if self.timer == 0 {
            self.timer = self.timer_reload;
            

            
        } else {
            self.timer -= 1;
        }
    }

    fn clock_sample_logic(&mut self) {
        if self.buffer_bits_remaining == 0 && self.remaining_sample_bytes > 0 {
                



/*

    Alright, here's the problem. The APU needs to access memory (DMA) to read samples from memory. 
    It will only ever read areas C000-FFFF.
    
    Since I've re-implemented the APU in an OO way with methods, I have been assuming that the APU 
    is self contained and doesn't modify the rest of the system state, which is maybe wrong. 
    
    Since it can't read below C000, it at least can't touch any memory mapped registers.
    It does need to read memory though, so it will need some access to the PRG ROM. 

    My options are to:

        - pass a mutable reference to the NES struct into the APU step function which ruins the 
            point of encapsulation
        - just pass the cartridge in, which is also a bit weird
        - rewrite the whole thing to use functions again
        - just rewrite the sample channel to use functions? 

    It would be nice to keep the methods because writing nes.apu.sample... instead of self. is 
    just really unpleasant to read. 

    I think for the moment I'll just pass the cartridge in, can have a re-think later on. 
    
    Rewriting to use functions instead of methods is fine honestly. It was helpful to refactor this
    with access control though, because there are so many little counters and variables that it
    becomes hard to remember when you should and shouldn't be able to modify a field from outside 
    of the channel or unit.  

*/










            let new_sample_data = read_mem(self.curr_sample_addr, nes);
            self.sample_buffer = new_sample_data;
            self.buffer_bits_remaining = 8;
            // Wrap around 0xC000-0xFFFF
            self.curr_sample_addr = self.curr_sample_addr.wrapping_add(1);
            if self.curr_sample_addr == 0 {self.curr_sample_addr = 0xC000}
            
            self.remaining_sample_bytes -= 1;

            if self.remaining_sample_bytes == 0 {
                if self.loop_sample {
                    self.curr_sample_addr = self.init_sample_addr;
                    self.remaining_sample_bytes = self.sample_length;
                } else if self.irq_enabled {
                    self.interrupt_request = true;
                }
            }
        }

        let delta: i8 = if (self.sample_buffer & 1) == 1 {2} else {-2}; 
        // This is wrong! It doesn't saturate, just doesn't add the offset if it doesn't fit in the range
        self.output = self.output.saturating_add_signed(delta).clamp(0, 0x7F);
        self.sample_buffer >>= 1;
        if self.buffer_bits_remaining > 0 {self.buffer_bits_remaining -= 1;}
    }


}