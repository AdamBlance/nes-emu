use crate::nes::cpu::instructions::common::{shift_left, shift_right, update_p_nz};
use crate::nes::Nes;

pub fn transfer_a_to_x(nes: &mut Nes) {
    nes.cpu.reg.x = nes.cpu.reg.a;
    update_p_nz(nes, nes.cpu.reg.x);
}
pub fn transfer_a_to_y(nes: &mut Nes) {
    nes.cpu.reg.y = nes.cpu.reg.a;
    update_p_nz(nes, nes.cpu.reg.y);
}
pub fn transfer_s_to_x(nes: &mut Nes) {
    nes.cpu.reg.x = nes.cpu.reg.s;
    update_p_nz(nes, nes.cpu.reg.x);
}
pub fn transfer_x_to_a(nes: &mut Nes) {
    nes.cpu.reg.a = nes.cpu.reg.x;
    update_p_nz(nes, nes.cpu.reg.a);
}
pub fn transfer_y_to_a(nes: &mut Nes) {
    nes.cpu.reg.a = nes.cpu.reg.y;
    update_p_nz(nes, nes.cpu.reg.a);
}
pub fn transfer_x_to_s(nes: &mut Nes) {
    nes.cpu.reg.s = nes.cpu.reg.x;
}

pub fn arithmetic_shift_left_accumulator(nes: &mut Nes) {
    nes.cpu.reg.a = shift_left(nes.cpu.reg.a, false, nes);
    update_p_nz(nes, nes.cpu.reg.a);
}
pub fn logical_shift_right_accumulator(nes: &mut Nes) {
    nes.cpu.reg.a = shift_right(nes.cpu.reg.a, false, nes);
    update_p_nz(nes, nes.cpu.reg.a);
}
pub fn rotate_left_accumulator(nes: &mut Nes) {
    nes.cpu.reg.a = shift_left(nes.cpu.reg.a, true, nes);
    update_p_nz(nes, nes.cpu.reg.a);
}
pub fn rotate_right_accumulator(nes: &mut Nes) {
    nes.cpu.reg.a = shift_right(nes.cpu.reg.a, true, nes);
    update_p_nz(nes, nes.cpu.reg.a);
}

pub fn increment_x(nes: &mut Nes) {
    nes.cpu.reg.x = nes.cpu.reg.x.wrapping_add(1);
    update_p_nz(nes, nes.cpu.reg.x);
}
pub fn increment_y(nes: &mut Nes) {
    nes.cpu.reg.y = nes.cpu.reg.y.wrapping_add(1);
    update_p_nz(nes, nes.cpu.reg.y);
}

pub fn clear_carry_flag(nes: &mut Nes) {
    nes.cpu.reg.p_c = false;
}
pub fn clear_decimal_flag(nes: &mut Nes) {
    nes.cpu.reg.p_d = false;
}
pub fn clear_interrupt_flag(nes: &mut Nes) {
    nes.cpu.reg.p_i = false;
}
pub fn clear_overflow_flag(nes: &mut Nes) {
    nes.cpu.reg.p_v = false;
}

pub fn set_carry_flag(nes: &mut Nes) {
    nes.cpu.reg.p_c = true;
}
pub fn set_decimal_flag(nes: &mut Nes) {
    nes.cpu.reg.p_d = true;
}
pub fn set_interrupt_inhibit_flag(nes: &mut Nes) {
    nes.cpu.reg.p_i = true;
}

pub fn decrement_x(nes: &mut Nes) {
    nes.cpu.reg.x = nes.cpu.reg.x.wrapping_sub(1);
    update_p_nz(nes, nes.cpu.reg.x);
}
pub fn decrement_y(nes: &mut Nes) {
    nes.cpu.reg.y = nes.cpu.reg.y.wrapping_sub(1);
    update_p_nz(nes, nes.cpu.reg.y);
}