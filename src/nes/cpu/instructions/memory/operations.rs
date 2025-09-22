use crate::nes::cpu::instructions::common::{update_p_nz, shift_left, shift_right};
use crate::nes::Nes;
use crate::util::get_bit;

pub fn load_a(nes: &mut Nes) {
    nes.cpu.reg.a = nes.cpu.ireg.data;
    update_p_nz(nes, nes.cpu.reg.a);
}
pub fn load_x(nes: &mut Nes) {
    nes.cpu.reg.x = nes.cpu.ireg.data;
    update_p_nz(nes, nes.cpu.reg.x);
}
pub fn load_y(nes: &mut Nes) {
    nes.cpu.reg.y = nes.cpu.ireg.data;
    update_p_nz(nes, nes.cpu.reg.y);
}

pub fn store_a(nes: &mut Nes) {
    nes.cpu.ireg.data = nes.cpu.reg.a;
}
pub fn store_x(nes: &mut Nes) {
    nes.cpu.ireg.data = nes.cpu.reg.x;
}
pub fn store_y(nes: &mut Nes) {
    nes.cpu.ireg.data = nes.cpu.reg.y;
}

pub fn exclusive_or(nes: &mut Nes) {
    nes.cpu.reg.a ^= nes.cpu.ireg.data;
    update_p_nz(nes, nes.cpu.reg.a);
}
pub fn or(nes: &mut Nes) {
    nes.cpu.reg.a |= nes.cpu.ireg.data;
    update_p_nz(nes, nes.cpu.reg.a);
}
pub fn and(nes: &mut Nes) {
    nes.cpu.reg.a &= nes.cpu.ireg.data;
    update_p_nz(nes, nes.cpu.reg.a);
}
pub fn test_bits_in_memory_with_a(nes: &mut Nes) {
    let result = nes.cpu.ireg.data & nes.cpu.reg.a;
    nes.cpu.reg.p_n = get_bit(nes.cpu.ireg.data, 7);
    nes.cpu.reg.p_v = get_bit(nes.cpu.ireg.data, 6);
    nes.cpu.reg.p_z = result == 0;
}

pub fn arithmetic_shift_left(nes: &mut Nes) {
    nes.cpu.ireg.data = shift_left(nes.cpu.ireg.data, false, nes);
    update_p_nz(nes, nes.cpu.ireg.data);
}
pub fn logical_shift_right(nes: &mut Nes) {
    nes.cpu.ireg.data = shift_right(nes.cpu.ireg.data, false, nes);
    update_p_nz(nes, nes.cpu.ireg.data);
}
pub fn rotate_left(nes: &mut Nes) {
    nes.cpu.ireg.data = shift_left(nes.cpu.ireg.data, true, nes);
    update_p_nz(nes, nes.cpu.ireg.data);
}
pub fn rotate_right(nes: &mut Nes) {
    nes.cpu.ireg.data = shift_right(nes.cpu.ireg.data, true, nes);
    update_p_nz(nes, nes.cpu.ireg.data);
}

pub fn add_with_carry(nes: &mut Nes) {
    add_value_to_a_with_carry(nes.cpu.ireg.data, nes);
    update_p_nz(nes, nes.cpu.reg.a);
}
pub fn subtract_with_carry(nes: &mut Nes) {
    add_value_to_a_with_carry(!nes.cpu.ireg.data, nes);
    update_p_nz(nes, nes.cpu.reg.a);
}

pub fn compare_memory_with_a(nes: &mut Nes) {
    compare_data_with_register(nes.cpu.reg.a, nes);
}
pub fn compare_memory_with_x(nes: &mut Nes) {
    compare_data_with_register(nes.cpu.reg.x, nes);
}
pub fn compare_memory_with_y(nes: &mut Nes) {
    compare_data_with_register(nes.cpu.reg.y, nes);
}

pub fn decrement_memory(nes: &mut Nes) {
    nes.cpu.ireg.data = nes.cpu.ireg.data.wrapping_sub(1);
    update_p_nz(nes, nes.cpu.ireg.data);
}

pub fn increment_memory(nes: &mut Nes) {
    nes.cpu.ireg.data = nes.cpu.ireg.data.wrapping_add(1);
    update_p_nz(nes, nes.cpu.ireg.data);
}

fn compare_data_with_register(reg_val: u8, nes: &mut Nes) {
    let result = reg_val.wrapping_sub(nes.cpu.ireg.data);
    nes.cpu.reg.p_z = result == 0;
    nes.cpu.reg.p_n = get_bit(result, 7);
    nes.cpu.reg.p_c = nes.cpu.ireg.data <= reg_val;
}

fn add_value_to_a_with_carry(val: u8, nes: &mut Nes) {
    let (result, carry) = nes.cpu.reg.a.carrying_add(val, nes.cpu.reg.p_c);
    // If the sign bits of A and B are the same
    // but the sign bits of A (or B) and A+B are different,
    // there was an overflow into or out of the sign bit
    nes.cpu.reg.p_v = ((!(nes.cpu.reg.a ^ val) & (nes.cpu.reg.a ^ result)) >> 7) == 1;
    nes.cpu.reg.p_c = carry;
    nes.cpu.reg.a = result;
}















pub fn and_memory_with_s_and_load_into_a_x_s(nes: &mut Nes) {
    let result = nes.cpu.ireg.data & nes.cpu.reg.s;
    nes.cpu.reg.a = result;
    nes.cpu.reg.x = result;
    nes.cpu.reg.s = result;
    update_p_nz(nes, nes.cpu.reg.a);
}

pub fn load_a_and_x(nes: &mut Nes) {
    nes.cpu.reg.a = nes.cpu.ireg.data;
    nes.cpu.reg.x = nes.cpu.ireg.data;
    update_p_nz(nes, nes.cpu.reg.a);
}

pub fn store_a_and_x(nes: &mut Nes) {
    nes.cpu.ireg.data = nes.cpu.reg.a & nes.cpu.reg.x;
}

pub fn decrement_memory_then_compare_with_a(nes: &mut Nes) {
    nes.cpu.ireg.data = nes.cpu.ireg.data.wrapping_sub(1);
    compare_memory_with_a(nes);
}

pub fn increment_memory_then_subtract_from_a(nes: &mut Nes) {
    nes.cpu.ireg.data = nes.cpu.ireg.data.wrapping_add(1);
    add_value_to_a_with_carry(!nes.cpu.ireg.data, nes);
    update_p_nz(nes, nes.cpu.reg.a);
}

pub fn shift_left_then_or_result_with_a(nes: &mut Nes) {
    nes.cpu.ireg.data = shift_left(nes.cpu.ireg.data, false, nes);
    nes.cpu.reg.a |= nes.cpu.ireg.data;
    update_p_nz(nes, nes.cpu.reg.a);
}

pub fn rotate_left_then_and_result_with_a(nes: &mut Nes) {
    nes.cpu.ireg.data = shift_left(nes.cpu.ireg.data, true, nes);
    nes.cpu.reg.a &= nes.cpu.ireg.data;
    update_p_nz(nes, nes.cpu.reg.a);
}

pub fn shift_right_then_xor_result_with_a(nes: &mut Nes) {
    nes.cpu.ireg.data = shift_right(nes.cpu.ireg.data, false, nes);
    nes.cpu.reg.a ^= nes.cpu.ireg.data;
    update_p_nz(nes, nes.cpu.reg.a);
}

pub fn rotate_right_then_and_result_with_a(nes: &mut Nes) {
    nes.cpu.ireg.data = shift_right(nes.cpu.ireg.data, true, nes);
    add_value_to_a_with_carry(nes.cpu.ireg.data, nes);
    update_p_nz(nes, nes.cpu.reg.a);
}

pub fn and_memory_with_a_then_set_carry_flag_to_negative_flag(nes: &mut Nes) {
    nes.cpu.reg.a &= nes.cpu.ireg.data;
    update_p_nz(nes, nes.cpu.reg.a);
    nes.cpu.reg.p_c = nes.cpu.reg.p_n;
}

pub fn and_x_with_a_then_subtract_memory_and_store_in_x(nes: &mut Nes) {
    nes.cpu.reg.x &= nes.cpu.reg.a;
    let (result, carry) = nes.cpu.reg.x.carrying_add(!nes.cpu.ireg.data, true);
    nes.cpu.reg.p_c = carry;
    nes.cpu.reg.x = result;
    update_p_nz(nes, nes.cpu.reg.x);
}

pub fn and_memory_with_a_then_shift_right(nes: &mut Nes) {
    nes.cpu.reg.a &= nes.cpu.ireg.data;
    nes.cpu.reg.a = shift_right(nes.cpu.reg.a, false, nes);
    update_p_nz(nes, nes.cpu.reg.a);
}

pub fn and_memory_with_a_then_rotate_right(nes: &mut Nes) {
    nes.cpu.reg.a &= nes.cpu.ireg.data;
    nes.cpu.reg.a = shift_right(nes.cpu.reg.a, true, nes);
    nes.cpu.reg.p_c = get_bit(nes.cpu.reg.a, 6);
    nes.cpu.reg.p_v = get_bit(nes.cpu.reg.a, 6) ^ get_bit(nes.cpu.reg.a, 5);
    update_p_nz(nes, nes.cpu.reg.a);
}

pub fn and_s_with_upper_address_then_store_in_memory(nes: &mut Nes) {
    nes.cpu.reg.s = nes.cpu.reg.a & nes.cpu.reg.x;
    nes.cpu.ireg.data = (nes.cpu.reg.s & nes.cpu.ireg.upper_address.wrapping_sub(nes.cpu.ireg.carry_out as u8).wrapping_add(1));
    if nes.cpu.ireg.carry_out {
        nes.cpu.ireg.upper_address = nes.cpu.ireg.data;
    }
}

pub fn and_y_with_upper_address_then_store_in_memory(nes: &mut Nes) {
    nes.cpu.ireg.data = nes.cpu.reg.y
        & (nes.cpu.ireg.upper_address.wrapping_sub(nes.cpu.ireg.carry_out as u8).wrapping_add(1));
    if nes.cpu.ireg.carry_out {
        nes.cpu.ireg.upper_address = nes.cpu.ireg.data;
    }
}

pub fn and_x_with_upper_address_then_store_in_memory(nes: &mut Nes) {
    nes.cpu.ireg.data = nes.cpu.reg.x & (nes.cpu.ireg.upper_address.wrapping_sub(nes.cpu.ireg.carry_out as u8).wrapping_add(1));
    if nes.cpu.ireg.carry_out {
        nes.cpu.ireg.upper_address = nes.cpu.ireg.data;
    }
}

pub fn and_a_with_x_with_upper_address_then_store_in_memory(nes: &mut Nes) {
    let val = nes.cpu.reg.a & nes.cpu.reg.x & nes.cpu.ireg.upper_address.wrapping_sub(nes.cpu.ireg.carry_out as u8).wrapping_add(1);
    nes.cpu.ireg.data = val;
    if nes.cpu.ireg.carry_out {
        nes.cpu.ireg.upper_address = nes.cpu.ireg.data;
    }
}

pub fn nondeterministic_nonsense(nes: &mut Nes) {
    // 0xEE - Magic nondeterministic value
    nes.cpu.reg.a = (nes.cpu.reg.a | 0xEE) & nes.cpu.reg.x & nes.cpu.ireg.data;
    update_p_nz(nes, nes.cpu.reg.a);
}