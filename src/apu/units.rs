
use crate::util::get_bit;

use super::step;

static LENGTH_TABLE: [u8; 32] = [
    0x0A, 0xFE, 0x14, 0x02, 0x28, 0x04, 0x50, 0x06, 
    0xA0, 0x08, 0x3C, 0x0A, 0x0E, 0x0C, 0x1A, 0x0E, 
    0x0C, 0x10, 0x18, 0x12, 0x30, 0x14, 0x60, 0x16, 
    0xC0, 0x18, 0x48, 0x1A, 0x10, 0x1C, 0x20, 0x1E,
];





/*

    Envelope
    Used by pulse and noise

    both use --LC.VVVV for first register

    both use LLLL.L--- for second register



    Sweep
    Only used by pulse channel

    Length counter

    pulse and noise have the length counter halt flag here --H-.----

    triangle has length counter halt here H---.----





*/




#[derive(Copy, Clone, Default)]
pub struct EnvelopeGenerator {
    divider: u8,
    start_flag: bool,
    loop_flag: bool,
    constant_volume_flag: bool,
    envelope_parameter: u8,
    decay_level: u8,
    output: u8,
}
impl EnvelopeGenerator {
    pub fn clock(&mut self) {
        if self.start_flag {
            self.start_flag = false;
            self.decay_level = 15;
            self.divider = self.envelope_parameter;
        } else {
            self.clock_divider();
        }
    }

    fn clock_divider(&mut self) {
        if self.divider > 0 {
            self.divider -= 1;
        } else {
            self.divider = self.envelope_parameter;
            self.clock_decay_counter();
        }
    }

    fn clock_decay_counter(&mut self) {
        if self.decay_level > 0 {
            self.decay_level -= 1;
        } else if self.loop_flag {
            self.decay_level = 15;
        }
    }

    pub fn configure_with_byte(&mut self, byte: u8) {
        self.constant_volume_flag = get_bit(byte, 4);
        self.envelope_parameter = byte & 0b0000_1111;
        self.loop_flag = get_bit(byte, 5);

        self.output = if self.constant_volume_flag {
            self.envelope_parameter
        } else {
            self.decay_level
        };
    }

    pub fn set_start_flag(&mut self) {
        self.start_flag = true;
    }

    pub fn get_output(&self) -> u8 {
        self.output
    }
}


#[derive(Copy, Clone, Default)]
pub struct SweepUnit {
    divider_period: u8,
    divider: u8,
    enabled_flag: bool,
    reload_flag: bool,
    is_pulse_1: bool,
    pulse_period: u16,
    target_period: u16,
    negate_flag: bool,
    shift_amount: u8,
    mute_flag: bool,
}
impl SweepUnit {

    pub fn clock(&mut self) {
        if self.divider == 0 && self.enabled_flag && !self.mute_flag && self.shift_amount != 0 {
            self.set_period(self.target_period);
        }

        if self.divider == 0 || self.reload_flag {
            self.divider = self.divider_period;
            self.reload_flag = false;
        } else {
            self.divider -= 1;
        }
    }

    pub fn set_period(&mut self, period: u16) {
        self.pulse_period = period;

        let period_change = self.pulse_period >> self.shift_amount;
        self.target_period = match (self.negate_flag, self.is_pulse_1) {
            (false, _) => self.pulse_period + period_change,
            (true, true) => self.pulse_period - period_change - 1,
            (true, false) => self.pulse_period - period_change,
        };

        self.mute_flag = self.pulse_period < 8 || self.target_period > 0x7FF;
    }

    pub fn configure_with_byte(&mut self, byte: u8) {
        self.enabled_flag = get_bit(byte, 7);
        self.divider_period = (byte & 0b0111_0000) >> 4;
        self.negate_flag = get_bit(byte, 3);
        self.shift_amount = byte & 0b0000_0111;
    }

    pub fn get_timer_period(&self) -> u16 {
        self.pulse_period
    }

}




#[derive(Copy, Clone, Default)]
pub struct LengthCounter {
    // When the channel is disabled, the length counter value cannot be changed
    channel_disabled: bool,
    counter: u8,
    halt_flag: bool,
    mute_flag: bool,
}
impl LengthCounter {
    pub fn clock(&mut self) {
        if self.counter > 0 && !self.halt_flag {
            self.counter -= 1;
        }
        self.mute_flag = self.counter == 0;
    }
    
    pub fn update_counter(&mut self, val: u8) {
        if !self.channel_disabled {
            self.counter = step::LENGTH_TABLE[val as usize];
        }
    }

    pub fn set_halt_flag(&mut self, val: bool) {
        self.halt_flag = val;
    }

    pub fn configure_with_byte(&mut self, byte: u8) {
        self.counter = LENGTH_TABLE[((byte & 0b1111_1000) >> 3) as usize];
    }
}


#[derive(Copy, Clone, Default)]
pub struct LinearCounter {
    reload_flag: bool,
    control_flag: bool,
    reload_value: u8,
    counter: u8,
    mute_flag: bool,
}
impl LinearCounter {
    pub fn clock(&mut self) {
        if self.reload_flag {
            self.counter = self.reload_value;
        } else if self.counter > 0 {
            self.counter -= 1;
        }

        if !self.control_flag {
            self.reload_flag = false;
        }

        self.mute_flag = self.counter == 0;
    }

    pub fn configure_with_byte(&mut self, byte: u8) {
        self.control_flag = (byte & 0b1000_0000) > 0;
        self.reload_value = byte & 0b0111_1111;
    }

    pub fn set_reload_flag(&mut self) {
        self.reload_flag = true;
    }
}

pub struct PeriodTimer {
    pub fn clock(&mut self, self.reset_period)
}