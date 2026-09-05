pub fn get_bit<T: Into<u32>>(integer: T, bit_index: u8) -> bool {
    let mask = 1 << bit_index;
    let masked = integer.into() & mask;
    masked > 0
}

pub fn concat_u8(msb: u8, lsb: u8) -> u16 {
    ((msb as u16) << 8) + (lsb as u16)
}

pub fn to_mask(input: bool) -> u8 {
    !(input as u8).wrapping_sub(1)
}
