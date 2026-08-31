use serde::{Deserialize, Serialize};

const H: bool = true;
const L: bool = false;
pub static SQUARE_SEQUENCES: [[bool; 8]; 4] = [
    [L, H, L, L, L, L, L, L], // 12.5% duty
    [L, H, H, L, L, L, L, L], // 25.0% duty
    [L, H, H, H, H, L, L, L], // 50.0% duty
    [H, L, L, H, H, H, H, H], // 75.0% duty
];

pub static TRIANGLE_SEQUENCE: [u8; 32] = [
    0xF, 0xE, 0xD, 0xC, 0xB, 0xA, 0x9, 0x8, 0x7, 0x6, 0x5, 0x4, 0x3, 0x2, 0x1, 0x0, 0x0, 0x1, 0x2,
    0x3, 0x4, 0x5, 0x6, 0x7, 0x8, 0x9, 0xA, 0xB, 0xC, 0xD, 0xE, 0xF,
];

pub static LENGTH_TABLE: [u8; 32] = [
    0x0A, 0xFE, 0x14, 0x02, 0x28, 0x04, 0x50, 0x06, 0xA0, 0x08, 0x3C, 0x0A, 0x0E, 0x0C, 0x1A, 0x0E,
    0x0C, 0x10, 0x18, 0x12, 0x30, 0x14, 0x60, 0x16, 0xC0, 0x18, 0x48, 0x1A, 0x10, 0x1C, 0x20, 0x1E,
];

pub static NOISE_PERIOD_TABLE: [u16; 16] = [
    0x004, 0x008, 0x010, 0x020, 0x040, 0x060, 0x080, 0x0A0, 0x0CA, 0x0FE, 0x17C, 0x1FC, 0x2FA,
    0x3F8, 0x7F2, 0xFE4,
];

pub static SAMPLE_RATE_TABLE: [u16; 16] = [
    428, 380, 340, 320, 286, 254, 226, 214, 190, 160, 142, 128, 106, 84, 72, 54,
];

#[derive(Copy, Clone, Default, Serialize, Deserialize)]
pub struct Square {
    pub enabled: bool,
    pub length_counter_mute_signal: bool,
    pub sequencer_stage: u8,
    pub timer_init_value: u16,
    pub timer_curr_value: u16,
    pub duty_cycle: u8,
    pub length_counter: u8,
    pub constant_volume: bool,
    pub envelope_loop_and_length_counter_halt: bool,
    pub envelope_start_flag: bool,
    pub volume_and_envelope_period: u8,
    pub envelope_counter_curr_value: u8,
    pub envelope_decay_level: u8,
    pub envelope_output: u8,
    pub sweep_enabled: bool,
    pub sweep_counter_init_value: u8,
    pub sweep_counter_curr_value: u8,
    pub sweep_mute_signal: bool,
    pub sweep_negate: bool,
    pub sweep_shift_amount: u8,
    pub sweep_reload_flag: bool,
    pub sequencer_output: bool,
}
impl Square {
    pub fn set_reg1_from_byte(&mut self, byte: u8) {
        self.duty_cycle = byte >> 6;
        self.envelope_loop_and_length_counter_halt = (byte & 0b0010_0000) > 0;
        self.constant_volume = (byte & 0b0001_0000) > 0;
        self.volume_and_envelope_period = byte & 0b0000_1111;
    }
    pub fn set_reg2_from_byte(&mut self, byte: u8) {
        self.sweep_enabled = (byte & 0b1000_0000) > 0;
        self.sweep_counter_init_value = (byte & 0b0111_0000) >> 4;
        self.sweep_negate = (byte & 0b0000_1000) > 0;
        self.sweep_shift_amount = byte & 0b0000_0111;
    }
    pub fn set_reg3_from_byte(&mut self, byte: u8) {
        self.timer_init_value &= 0b111_0000_0000;
        self.timer_init_value |= byte as u16;
    }
    pub fn set_reg4_from_byte(&mut self, byte: u8) {
        self.timer_init_value &= 0b000_1111_1111;
        self.timer_init_value |= ((byte as u16) & 0b111) << 8;
        self.length_counter = LENGTH_TABLE[((byte & 0b11111_000) >> 3) as usize];
        self.envelope_start_flag = true;
    }
}

#[derive(Copy, Clone, Default, Serialize, Deserialize)]
pub struct Triangle {
    pub enabled: bool,
    pub sequencer_stage: u8,
    pub sequencer_output: u8,
    pub timer_init_value: u16,
    pub timer_curr_value: u16,
    pub length_counter: u8,
    pub length_counter_halt_and_linear_counter_control: bool,
    pub length_counter_mute_signal: bool,
    pub linear_counter_reload_flag: bool,
    pub linear_counter_init_value: u8,
    pub linear_counter_curr_value: u8,
    pub linear_counter_mute_signal: bool,
}
impl Triangle {
    pub fn set_reg1_from_byte(&mut self, byte: u8) {
        self.length_counter_halt_and_linear_counter_control = (byte & 0b1000_0000) > 0;
        self.linear_counter_init_value = byte & 0b0111_1111;
    }
    pub fn set_reg2_from_byte(&mut self, byte: u8) {
        self.timer_init_value &= 0b111_0000_0000;
        self.timer_init_value |= byte as u16;
    }
    pub fn set_reg3_from_byte(&mut self, byte: u8) {
        self.timer_init_value &= 0b000_1111_1111;
        self.timer_init_value |= ((byte as u16) & 0b111) << 8;
        self.length_counter = LENGTH_TABLE[((byte & 0b11111_000) >> 3) as usize];
        self.linear_counter_reload_flag = true;
    }
}

#[derive(Copy, Clone, Default, Serialize, Deserialize)]
pub struct Noise {
    pub enabled: bool,
    pub envelope_loop_and_length_counter_halt: bool,
    pub constant_volume: bool,
    pub length_counter: u8,
    pub length_counter_mute_signal: bool,
    pub envelope_start_flag: bool,
    pub envelope_decay_level: u8,
    pub envelope_counter_curr_value: u8,
    pub volume_and_envelope_period: u8,
    pub sequencer_output: bool,
    pub envelope_output: u8,
    pub mode: bool,
    pub timer_init_value: u16,
    pub timer_curr_value: u16,
}
impl Noise {
    pub fn set_reg1_from_byte(&mut self, byte: u8) {
        // println!("Reg 1 set");
        self.envelope_loop_and_length_counter_halt = (byte & 0b0010_0000) > 0;
        self.constant_volume = (byte & 0b0001_0000) > 0;
        self.volume_and_envelope_period = byte & 0b0000_1111;
    }
    pub fn set_reg2_from_byte(&mut self, byte: u8) {
        // println!("Reg 2 set");
        // This will go unused. I'm not convinced that it does anything substantial
        self.mode = (byte & 0b1000_0000) > 0;
        self.timer_init_value = NOISE_PERIOD_TABLE[(byte & 0b0000_1111) as usize];
    }
    pub fn set_reg3_from_byte(&mut self, byte: u8) {
        // println!("Reg 3 set, mute {}", self.length_counter_mute_signal);
        self.length_counter = LENGTH_TABLE[((byte & 0b1111_1000) >> 3) as usize];
        self.envelope_start_flag = true;
    }
}

#[derive(Copy, Clone, Default, Serialize, Deserialize)]
pub struct Sample {
    pub enabled: bool,
    // 0x4010
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
        self.irq_enabled = (byte & 0b1000_0000) > 0;
        if !self.irq_enabled {
            self.interrupt_request = false
        }
        self.loop_sample = (byte & 0b0100_0000) > 0;
        self.init_timer_value = SAMPLE_RATE_TABLE[(byte & 0b0000_1111) as usize];
    }
    pub fn set_reg2_from_byte(&mut self, byte: u8) {
        // println!("Written {:08b} enabled {}", byte, self.enabled);
        self.output = byte & 0b0111_1111;
    }
    pub fn set_reg3_from_byte(&mut self, byte: u8) {
        self.init_sample_addr = 0xC000 + (byte as u16 * 64);
    }
    pub fn set_reg4_from_byte(&mut self, byte: u8) {
        self.sample_length = (byte as u16 * 16) + 1;
    }
}
