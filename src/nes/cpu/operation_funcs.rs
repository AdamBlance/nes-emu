use crate::nes::Nes;
use crate::util::*;

pub fn update_p_nz(nes: &mut Nes, val: u8) {
    nes.cpu.p_n = val > 0x7F;
    nes.cpu.p_z = val == 0;
}

fn shift_left(val: u8, rotate: bool, nes: &mut Nes) -> u8 {
    let prev_carry = nes.cpu.p_c;
    nes.cpu.p_c = get_bit(val, 7);
    (val << 1) | ((prev_carry && rotate) as u8)
}
fn shift_right(val: u8, rotate: bool, nes: &mut Nes) -> u8 {
    let prev_carry = nes.cpu.p_c;
    nes.cpu.p_c = get_bit(val, 0);
    (val >> 1) | (((prev_carry && rotate) as u8) << 7)
}

fn was_signed_overflow(a: u8, b: u8, a_plus_b: u8) -> bool {
    // If the sign bits of A and B are the same
    // and the sign bits of A and A+B are different,
    // sign bit was corrupted (there was signed overflow)
    ((!(a ^ b) & (a ^ a_plus_b)) >> 7) == 1
}

fn add_value_to_a_with_carry(val: u8, nes: &mut Nes) {
    let (result, carry) = nes.cpu.a.carrying_add(val, nes.cpu.p_c);
    nes.cpu.p_v = was_signed_overflow(nes.cpu.a, val, result);
    nes.cpu.p_c = carry;
    nes.cpu.a = result;
}

fn compare_data_with_register(reg_val: u8, nes: &mut Nes) {
    let result = reg_val.wrapping_sub(nes.cpu.data);
    nes.cpu.p_z = result == 0;
    nes.cpu.p_n = get_bit(result, 7);
    nes.cpu.p_c = nes.cpu.data <= reg_val;
}

pub fn load_a(nes: &mut Nes) {
    nes.cpu.a = nes.cpu.data;
    update_p_nz(nes, nes.cpu.a);
}
pub fn load_x(nes: &mut Nes) {
    nes.cpu.x = nes.cpu.data;
    update_p_nz(nes, nes.cpu.x);
}
pub fn load_y(nes: &mut Nes) {
    nes.cpu.y = nes.cpu.data;
    update_p_nz(nes, nes.cpu.y);
}

pub fn store_a(nes: &mut Nes) {
    nes.cpu.data = nes.cpu.a;
}
pub fn store_x(nes: &mut Nes) {
    nes.cpu.data = nes.cpu.x;
}
pub fn store_y(nes: &mut Nes) {
    nes.cpu.data = nes.cpu.y;
}

pub fn xor(nes: &mut Nes) {
    nes.cpu.a ^= nes.cpu.data;
    update_p_nz(nes, nes.cpu.a);
}
pub fn or(nes: &mut Nes) {
    nes.cpu.a |= nes.cpu.data;
    update_p_nz(nes, nes.cpu.a);
}
pub fn and(nes: &mut Nes) {
    nes.cpu.a &= nes.cpu.data;
    update_p_nz(nes, nes.cpu.a);
}
pub fn bit(nes: &mut Nes) {
    let result = nes.cpu.data & nes.cpu.a;
    nes.cpu.p_n = get_bit(nes.cpu.data, 7);
    nes.cpu.p_v = get_bit(nes.cpu.data, 6);
    nes.cpu.p_z = result == 0;
}

pub fn transfer_a_to_x(nes: &mut Nes) {
    nes.cpu.x = nes.cpu.a;
    update_p_nz(nes, nes.cpu.x);
}
pub fn transfer_a_to_y(nes: &mut Nes) {
    nes.cpu.y = nes.cpu.a;
    update_p_nz(nes, nes.cpu.y);
}
pub fn transfer_s_to_x(nes: &mut Nes) {
    nes.cpu.x = nes.cpu.s;
    update_p_nz(nes, nes.cpu.x);
}
pub fn transfer_x_to_a(nes: &mut Nes) {
    nes.cpu.a = nes.cpu.x;
    update_p_nz(nes, nes.cpu.a);
}
pub fn transfer_y_to_a(nes: &mut Nes) {
    nes.cpu.a = nes.cpu.y;
    update_p_nz(nes, nes.cpu.a);
}
pub fn transfer_x_to_s(nes: &mut Nes) {
    nes.cpu.s = nes.cpu.x;
}

pub fn arithmetic_shift_left(nes: &mut Nes) {
    nes.cpu.data = shift_left(nes.cpu.data, false, nes);
    update_p_nz(nes, nes.cpu.data);
}
pub fn logical_shift_right(nes: &mut Nes) {
    nes.cpu.data = shift_right(nes.cpu.data, false, nes);
    update_p_nz(nes, nes.cpu.data);
}
pub fn rotate_left(nes: &mut Nes) {
    nes.cpu.data = shift_left(nes.cpu.data, true, nes);
    update_p_nz(nes, nes.cpu.data);
}
pub fn rotate_right(nes: &mut Nes) {
    nes.cpu.data = shift_right(nes.cpu.data, true, nes);
    update_p_nz(nes, nes.cpu.data);
}
pub fn arithmetic_shift_left_accumulator(nes: &mut Nes) {
    nes.cpu.a = shift_left(nes.cpu.a, false, nes);
    update_p_nz(nes, nes.cpu.a);
}
pub fn logical_shift_right_accumulator(nes: &mut Nes) {
    nes.cpu.a = shift_right(nes.cpu.a, false, nes);
    update_p_nz(nes, nes.cpu.a);
}
pub fn rotate_left_accumulator(nes: &mut Nes) {
    nes.cpu.a = shift_left(nes.cpu.a, true, nes);
    update_p_nz(nes, nes.cpu.a);
}
pub fn rotate_right_accumulator(nes: &mut Nes) {
    nes.cpu.a = shift_right(nes.cpu.a, true, nes);
    update_p_nz(nes, nes.cpu.a);
}

pub fn add_with_carry(nes: &mut Nes) {
    add_value_to_a_with_carry(nes.cpu.data, nes);
    update_p_nz(nes, nes.cpu.a);
}
pub fn subtract_with_carry(nes: &mut Nes) {
    add_value_to_a_with_carry(!nes.cpu.data, nes);
    update_p_nz(nes, nes.cpu.a);
}

pub fn compare_memory_with_a(nes: &mut Nes) {
    compare_data_with_register(nes.cpu.a, nes);
}
pub fn compare_memory_with_x(nes: &mut Nes) {
    compare_data_with_register(nes.cpu.x, nes);
}
pub fn compare_memory_with_y(nes: &mut Nes) {
    compare_data_with_register(nes.cpu.y, nes);
}

pub fn decrement_memory(nes: &mut Nes) {
    nes.cpu.data = nes.cpu.data.wrapping_sub(1);
    update_p_nz(nes, nes.cpu.data);
}
pub fn decrement_x(nes: &mut Nes) {
    nes.cpu.x = nes.cpu.x.wrapping_sub(1);
    update_p_nz(nes, nes.cpu.x);
}
pub fn decrement_y(nes: &mut Nes) {
    nes.cpu.y = nes.cpu.y.wrapping_sub(1);
    update_p_nz(nes, nes.cpu.y);
}

pub fn increment_memory(nes: &mut Nes) {
    nes.cpu.data = nes.cpu.data.wrapping_add(1);
    update_p_nz(nes, nes.cpu.data);
}
pub fn increment_x(nes: &mut Nes) {
    nes.cpu.x = nes.cpu.x.wrapping_add(1);
    update_p_nz(nes, nes.cpu.x);
}
pub fn increment_y(nes: &mut Nes) {
    nes.cpu.y = nes.cpu.y.wrapping_add(1);
    update_p_nz(nes, nes.cpu.y);
}

pub fn clear_carry_flag(nes: &mut Nes) {
    nes.cpu.p_c = false;
}
pub fn clear_decimal_flag(nes: &mut Nes) {
    nes.cpu.p_d = false;
}
pub fn clear_interrupt_flag(nes: &mut Nes) {
    nes.cpu.p_i = false;
}
pub fn clear_overflow_flag(nes: &mut Nes) {
    nes.cpu.p_v = false;
}

pub fn set_carry_flag(nes: &mut Nes) {
    nes.cpu.p_c = true;
}
pub fn set_decimal_flag(nes: &mut Nes) {
    nes.cpu.p_d = true;
}
pub fn set_interrupt_inhibit_flag(nes: &mut Nes) {
    nes.cpu.p_i = true;
}

pub fn none(_nes: &mut Nes) {}

// unofficial

pub fn jam(_nes: &mut Nes) {
    panic!("JAM instruction encountered!");
}

pub fn las(nes: &mut Nes) {
    let result = nes.cpu.data & nes.cpu.s;
    nes.cpu.a = result;
    nes.cpu.x = result;
    nes.cpu.s = result;
    update_p_nz(nes, nes.cpu.a);
}

pub fn load_a_and_x(nes: &mut Nes) {
    nes.cpu.a = nes.cpu.data;
    nes.cpu.x = nes.cpu.data;
    update_p_nz(nes, nes.cpu.a);
}

pub fn store_a_and_x(nes: &mut Nes) {
    nes.cpu.data = nes.cpu.a & nes.cpu.x;
}

pub fn dec_then_compare(nes: &mut Nes) {
    nes.cpu.data = nes.cpu.data.wrapping_sub(1);
    compare_memory_with_a(nes);
}

pub fn isb(nes: &mut Nes) {
    nes.cpu.data = nes.cpu.data.wrapping_add(1);
    add_value_to_a_with_carry(!nes.cpu.data, nes);
    update_p_nz(nes, nes.cpu.a);
}

pub fn slo(nes: &mut Nes) {
    nes.cpu.data = shift_left(nes.cpu.data, false, nes);
    nes.cpu.a |= nes.cpu.data;
    update_p_nz(nes, nes.cpu.a);
}

pub fn rla(nes: &mut Nes) {
    nes.cpu.data = shift_left(nes.cpu.data, true, nes);
    nes.cpu.a &= nes.cpu.data;
    update_p_nz(nes, nes.cpu.a);
}

pub fn sre(nes: &mut Nes) {
    nes.cpu.data = shift_right(nes.cpu.data, false, nes);
    nes.cpu.a ^= nes.cpu.data;
    update_p_nz(nes, nes.cpu.a);
}

pub fn rra(nes: &mut Nes) {
    nes.cpu.data = shift_right(nes.cpu.data, true, nes);
    add_value_to_a_with_carry(nes.cpu.data, nes);
    update_p_nz(nes, nes.cpu.a);
}

pub fn anc(nes: &mut Nes) {
    nes.cpu.a &= nes.cpu.data;
    update_p_nz(nes, nes.cpu.a);
    nes.cpu.p_c = nes.cpu.p_n;
}

pub fn sbx(nes: &mut Nes) {
    nes.cpu.x &= nes.cpu.a;
    let (result, carry) = nes.cpu.x.carrying_add(!nes.cpu.data, true);
    nes.cpu.p_c = carry;
    nes.cpu.x = result;
    update_p_nz(nes, nes.cpu.x);
}

pub fn asr(nes: &mut Nes) {
    nes.cpu.a &= nes.cpu.data;
    nes.cpu.a = shift_right(nes.cpu.a, false, nes);
    update_p_nz(nes, nes.cpu.a);
}

pub fn arr(nes: &mut Nes) {
    nes.cpu.a &= nes.cpu.data;
    nes.cpu.a = shift_right(nes.cpu.a, true, nes);
    nes.cpu.p_c = get_bit(nes.cpu.a, 6);
    nes.cpu.p_v = get_bit(nes.cpu.a, 6) ^ get_bit(nes.cpu.a, 5);
    update_p_nz(nes, nes.cpu.a);
}

pub fn shs(nes: &mut Nes) {
    nes.cpu.s = nes.cpu.a & nes.cpu.x;
    nes.cpu.data = (nes.cpu.s & nes.cpu.upper_address.wrapping_sub(nes.cpu.internal_carry_out as u8).wrapping_add(1));
    if nes.cpu.internal_carry_out {
        nes.cpu.upper_address = nes.cpu.data;
    }
}

pub fn shy(nes: &mut Nes) {
    nes.cpu.data = nes.cpu.y
        & (nes.cpu.upper_address.wrapping_sub(nes.cpu.internal_carry_out as u8).wrapping_add(1));
    if nes.cpu.internal_carry_out {
        nes.cpu.upper_address = nes.cpu.data;
    }
}

pub fn shx(nes: &mut Nes) {
    nes.cpu.data = nes.cpu.x & (nes.cpu.upper_address.wrapping_sub(nes.cpu.internal_carry_out as u8).wrapping_add(1));
    if nes.cpu.internal_carry_out {
        nes.cpu.upper_address = nes.cpu.data;
    }
}

pub fn sha(nes: &mut Nes) {
    let val = nes.cpu.a & nes.cpu.x & nes.cpu.upper_address.wrapping_sub(nes.cpu.internal_carry_out as u8).wrapping_add(1);
    nes.cpu.data = val;
    if nes.cpu.internal_carry_out {
        nes.cpu.upper_address = nes.cpu.data;
    }
}

pub fn xaa(nes: &mut Nes) {
    // 0xEE - Magic nondeterministic value
    nes.cpu.a = (nes.cpu.a | 0xEE) & nes.cpu.x & nes.cpu.data;
    update_p_nz(nes, nes.cpu.a);
}
