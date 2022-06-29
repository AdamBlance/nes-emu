
use crate::util::get_bit;

static LENGTH_TABLE: [u8; 32] = [
    10,254, 20,  2, 40,  4, 80,  6, 160,  8, 60, 10, 14, 12, 26, 14,
    12, 16, 24, 18, 48, 20, 96, 22, 192, 24, 72, 26, 16, 28, 32, 30,
];


// This encapsulation is nice and all but will have to go when I want to create a debugger
// There's no way I'm writing getters and setters for no reason



#[derive(Copy, Clone, Default)]
pub struct EnvelopeGenerator {
    divider: u8,
    start_flag: bool,
    loop_flag: bool,
    constant_volume_flag: bool,
    envelope_parameter: u8,
    decay_level: u8,
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

    // should get rid of all this configure with byte stuff, it's just confusing having to go around a bunch of files
    // I should try to minimise the amount of jumping around you have to do, even if that means duplicating some code

    pub fn configure_with_byte(&mut self, byte: u8) {
        self.constant_volume_flag = get_bit(byte, 4);
        self.envelope_parameter = byte & 0b0000_1111;
        self.loop_flag = get_bit(byte, 5);
    }

    pub fn set_start_flag(&mut self) {
        self.start_flag = true;
    }

    pub fn get_output(&self) -> u8 {
        if self.constant_volume_flag {
            self.envelope_parameter
        } else {
            self.decay_level
        }
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
            self.set_timer_period(self.target_period);
        }

        if self.divider == 0 || self.reload_flag {
            self.divider = self.divider_period;
            self.reload_flag = false;
        } else {
            self.divider -= 1;
        }
    }

    pub fn set_timer_period(&mut self, period: u16) {
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

    pub fn is_muting(&self) -> bool {
        self.mute_flag
    }

}







#[derive(Copy, Clone, Default)]
pub struct LengthCounter {
    // When the channel is disabled, the length counter value cannot be changed
    channel_disabled: bool,
    pub counter: u8,  // temp? 
    halt_flag: bool,
}
impl LengthCounter {

    pub fn set_lock(&mut self, status: bool) {
        // println!("Enabled/disabled {status}");
        self.channel_disabled = !status;
        if self.channel_disabled {
            self.counter = 0;
        }
    }

    pub fn clock(&mut self) {
        if self.counter > 0 && !self.halt_flag {
            self.counter -= 1;
        }
        println!("Length counter {}", self.counter);
    }

    pub fn set_halt_flag(&mut self, val: bool) {
        self.halt_flag = val;
        if self.halt_flag {
            println!("Something halting");
        }
    }

    pub fn configure_with_byte(&mut self, byte: u8) {
        if !self.channel_disabled {
            println!("configuring with {}", (byte & 0b1111_1000) >> 3);
            self.counter = LENGTH_TABLE[((byte & 0b1111_1000) >> 3) as usize];
        } else {
            println!("Still disabled");
        }
    }

    pub fn is_muting(&self) -> bool {
        self.counter == 0
    }
}















#[derive(Copy, Clone, Default)]
pub struct LinearCounter {
    reload_flag: bool,
    control_flag: bool,
    reload_value: u8,
    counter: u8,
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
    }

    pub fn configure_with_byte(&mut self, byte: u8) {
        self.control_flag = (byte & 0b1000_0000) > 0;
        self.reload_value = byte & 0b0111_1111;
    }

    pub fn set_reload_flag(&mut self) {
        self.reload_flag = true;
    }

    pub fn is_muting(&self) -> bool {
        self.counter == 0
    }
}
