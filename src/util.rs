
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
