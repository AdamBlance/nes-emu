
use super::units::{EnvelopeGenerator, SweepUnit, LengthCounter, LinearCounter};
use crate::util::get_bit;

pub static NOISE_PERIOD_TABLE: [u16; 16] = [
    0x004, 0x008, 0x010, 0x020, 0x040, 0x060, 0x080, 0x0A0, 
    0x0CA, 0x0FE, 0x17C, 0x1FC, 0x2FA, 0x3F8, 0x7F2, 0xFE4,
];

#[derive(Copy, Clone, Default)]
pub struct Square {
    pub envelope_generator: EnvelopeGenerator,
    pub sweep_unit: SweepUnit, 
    pub length_counter: LengthCounter,

    timer_reload: u16,
    timer: u16,

    duty_cycle: u8,
    sequencer_stage: u8,
}
impl Square {
    pub fn set_reg1_from_byte(&mut self, byte: u8) {
        self.duty_cycle = (byte & 0b1100_0000) >> 6;

        self.length_counter.set_halt_flag(get_bit(byte, 5));
        self.envelope_generator.configure_with_byte(byte);
    }
    pub fn set_reg2_from_byte(&mut self, byte: u8) {
        self.sweep_unit.configure_with_byte(byte);
    }
    pub fn set_reg3_from_byte(&mut self, byte: u8) {
        self.timer_reload &= 0b111_0000_0000;
        self.timer_reload |= byte as u16;
    }
    pub fn set_reg4_from_byte(&mut self, byte: u8) {
        self.timer_reload &= 0b000_1111_1111;
        self.timer_reload |= (byte as u16 & 0b0000_0111) << 8;
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
}
impl Noise {
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
    pub init_timer_value: u16,
    pub curr_timer_value: u16,

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
    pub fn set_reg1_from_byte(&mut self, byte: u8) {
        self.irq_enabled      = (byte & 0b1000_0000) > 0;
        if !self.irq_enabled {self.interrupt_request = false}
        self.loop_sample      = (byte & 0b0100_0000) > 0;
        self.init_timer_value = SAMPLE_RATE_TABLE[(byte & 0b0000_1111) as usize];
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
        if nes.apu.sample.curr_timer_value == 0 {
            nes.apu.sample.curr_timer_value = nes.apu.sample.init_timer_value;
            
            if nes.apu.sample.buffer_bits_remaining == 0 && nes.apu.sample.remaining_sample_bytes > 0 && nes.apu.sample.enabled {
                
                let new_sample_data = read_mem(nes.apu.sample.curr_sample_addr, nes);
                nes.apu.sample.sample_buffer = new_sample_data;
                nes.apu.sample.buffer_bits_remaining = 8;
                // Wrap around 0xC000-0xFFFF
                nes.apu.sample.curr_sample_addr = nes.apu.sample.curr_sample_addr.wrapping_add(1);
                if nes.apu.sample.curr_sample_addr == 0 {nes.apu.sample.curr_sample_addr = 0xC000}
                
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
    
            let delta: i8 = if (nes.apu.sample.sample_buffer & 1) == 1 {2} else {-2}; 
            // This is wrong! It doesn't saturate, just doesn't add the offset if it doesn't fit in the range
            nes.apu.sample.output = nes.apu.sample.output.saturating_add_signed(delta).clamp(0, 0x7F);
            nes.apu.sample.sample_buffer >>= 1;
            if nes.apu.sample.buffer_bits_remaining > 0 {nes.apu.sample.buffer_bits_remaining -= 1;}
            
        } else {
            nes.apu.sample.curr_timer_value -= 1;
        }
    }
}