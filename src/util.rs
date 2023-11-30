use std::cmp::{max, min};

pub fn get_bit(byte: u8, idx: u8) -> bool {
        (byte & (0x01 << idx)) != 0
    }
    pub fn get_bit_u16(byte: u16, idx: u8) -> bool {
        (byte & (0x01 << idx)) != 0
    }
    pub fn concat_u8(msb: u8, lsb: u8) -> u16 {
        ((msb as u16) << 8) + (lsb as u16)
    }
    pub fn is_neg(val: u8) -> bool {
        val > 0x7F
    }
    pub fn flip_byte(val: u8) -> u8 {
        ((val & 0b1) << 7) | ((val & 0b10) << 5)
            | ((val & 0b100) << 3)
            | ((val & 0b1000) << 1)
            | ((val & 0b10000) >> 1)
            | ((val & 0b100000) >> 3)
            | ((val & 0b1000000) >> 5)
            | ((val & 0b10000000) >> 7)
    }

    pub fn to_mask(input: bool) -> u8 {
        !((input as u8).wrapping_sub(1))
    }

    pub fn clamp<T: Ord>(x: T, lower: T, upper: T) -> T {
        max(lower, min(x, upper))
    }

