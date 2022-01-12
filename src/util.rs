pub fn get_bit(byte: u8, idx: u8) -> bool {
    (byte & (0x01 << idx)) != 0
}
pub fn concat_u8(msb: u8, lsb: u8) -> u16 {
    ((msb as u16) << 8) + (lsb as u16)
}
pub fn was_signed_overflow(orig: u8, operand: u8, result: u8) -> bool {
    // If the sign bits of A and B are the same
    // and the sign bits of A and A+B are different,
    // sign bit was corrupted (there was signed overflow)
    ((!(orig ^ operand) & (orig ^ result)) >> 7) == 1
}
pub fn is_neg(val: u8) -> bool {
    val > 0x7F
}