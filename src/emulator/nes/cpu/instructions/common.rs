use crate::nes::Nes;
use crate::util::get_bit;

// Used by most instructions
pub fn update_p_nz(nes: &mut Nes, val: u8) {
    nes.cpu.reg.p_n = val > 0x7F;
    nes.cpu.reg.p_z = val == 0;
}

// Used by both nonmemory (accumulator) instructions and memory instructions
pub(crate) fn shift_left(val: u8, rotate: bool, nes: &mut Nes) -> u8 {
    let prev_carry = nes.cpu.reg.p_c;
    nes.cpu.reg.p_c = get_bit(val, 7);
    (val << 1) | ((prev_carry && rotate) as u8)
}
pub(crate) fn shift_right(val: u8, rotate: bool, nes: &mut Nes) -> u8 {
    let prev_carry = nes.cpu.reg.p_c;
    nes.cpu.reg.p_c = get_bit(val, 0);
    (val >> 1) | (((prev_carry && rotate) as u8) << 7)
}
