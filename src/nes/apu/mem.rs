use crate::nes::mem_consts::*;

pub fn apu_status_read(nes: &mut crate::nes::Nes) -> u8 {
    let result = nes.apu.square1.length_counter.min(1)
        | (nes.apu.square2.length_counter.min(1) << 1)
        | (nes.apu.triangle.length_counter.min(1) << 2)
        | (nes.apu.noise.length_counter.min(1) << 3)
        | ((nes.apu.sample.remaining_sample_bytes.min(1) as u8) << 4)
        | (nes.cpu.open_bus & 0b0010_0000)
        | ((nes.apu.interrupt_request as u8) << 6)
        | ((nes.apu.sample.interrupt_request as u8) << 7);
    nes.apu.interrupt_request = false;
    result
}

pub fn apu_status_write(val: u8, nes: &mut crate::nes::Nes) {
    nes.apu.sample.interrupt_request = false;

    nes.apu.square1.enabled = (val & 0b01) > 0;
    nes.apu.square2.enabled = (val & 0b10) > 0;
    nes.apu.triangle.enabled = (val & 0b100) > 0;
    nes.apu.noise.enabled = (val & 0b1000) > 0;
    nes.apu.sample.enabled = (val & 0b10000) > 0;

    if !nes.apu.square1.enabled {
        nes.apu.square1.length_counter = 0;
    }
    if !nes.apu.square2.enabled {
        nes.apu.square2.length_counter = 0;
    }
    if !nes.apu.triangle.enabled {
        nes.apu.triangle.length_counter = 0;
    }
    if !nes.apu.noise.enabled {
        nes.apu.noise.length_counter = 0;
    }
    if !nes.apu.sample.enabled {
        nes.apu.sample.remaining_sample_bytes = 0;
    } else {
        nes.apu.sample.remaining_sample_bytes = nes.apu.sample.sample_length;
        nes.apu.sample.curr_sample_addr = nes.apu.sample.init_sample_addr;
    } // fix this, add silence flag like other channels
}

pub fn apu_channels_write(addr: u16, val: u8, nes: &mut crate::nes::Nes) {
    match addr {
        PULSE_1_REG_1 => nes.apu.square1.set_reg1_from_byte(val),
        PULSE_1_REG_2 => nes.apu.square1.set_reg2_from_byte(val),
        PULSE_1_REG_3 => nes.apu.square1.set_reg3_from_byte(val),
        PULSE_1_REG_4 => nes.apu.square1.set_reg4_from_byte(val),

        PULSE_2_REG_1 => nes.apu.square2.set_reg1_from_byte(val),
        PULSE_2_REG_2 => nes.apu.square2.set_reg2_from_byte(val),
        PULSE_2_REG_3 => nes.apu.square2.set_reg3_from_byte(val),
        PULSE_2_REG_4 => nes.apu.square2.set_reg4_from_byte(val),

        TRIANGLE_REG_1 => nes.apu.triangle.set_reg1_from_byte(val),
        TRIANGLE_REG_2 => nes.apu.triangle.set_reg2_from_byte(val),
        TRIANGLE_REG_3 => nes.apu.triangle.set_reg3_from_byte(val),

        NOISE_REG_1 => nes.apu.noise.set_reg1_from_byte(val),
        NOISE_REG_2 => nes.apu.noise.set_reg2_from_byte(val),
        NOISE_REG_3 => nes.apu.noise.set_reg3_from_byte(val),

        SAMPLE_REG_1 => nes.apu.sample.set_reg1_from_byte(val),
        SAMPLE_REG_2 => nes.apu.sample.set_reg2_from_byte(val),
        SAMPLE_REG_3 => nes.apu.sample.set_reg3_from_byte(val),
        SAMPLE_REG_4 => nes.apu.sample.set_reg4_from_byte(val),
        _ => {}
    }
}
