use crate::nes::mem::{read_mem, write_mem};
use crate::nes::Nes;
use crate::util::concat_u8;

/*

    Much of this is redundant or unnecessary, will refactor another time.
    Most of this could also be made into methods.
    The advantage of all these methods is the method names document how the instructions work
    cycle-by-cycle in plain English.
    It ended up this way because I was storing these in static arrays previously and needed
    all the functions to have the same signature so I could call them all in the same way at
    runtime using function pointers.

*/

// PC

pub fn increment_pc(nes: &mut Nes) {
    nes.cpu.pc = nes.cpu.pc.wrapping_add(1);
}
pub fn copy_address_to_pc(nes: &mut Nes) {
    nes.cpu.pc = nes.cpu.get_address();
}
pub fn fetch_lower_pc_from_interrupt_vector(nes: &mut Nes) {
    let lower = read_mem(nes.cpu.interrupt_vector, nes);
    nes.cpu.set_lower_pc(lower);
}
pub fn fetch_upper_pc_from_interrupt_vector(nes: &mut Nes) {
    let upper = read_mem(nes.cpu.interrupt_vector + 1, nes);
    nes.cpu.set_upper_pc(upper);
}

// Immediate

pub fn fetch_immediate_from_pc(nes: &mut Nes) {
    nes.cpu.data = read_mem(nes.cpu.pc, nes);
}

// Address

pub fn take_operand_as_low_address_byte(nes: &mut Nes) {
    nes.cpu.lower_address = read_mem(nes.cpu.pc, nes);
}
pub fn take_operand_as_high_address_byte(nes: &mut Nes) {
    nes.cpu.upper_address = read_mem(nes.cpu.pc, nes);
}
pub fn fetch_low_address_byte_using_indirect_address(nes: &mut Nes) {
    nes.cpu.lower_address = read_mem(nes.cpu.get_pointer(), nes);
}
pub fn fetch_high_address_byte_using_indirect_address(nes: &mut Nes) {
    nes.cpu.upper_address = read_mem(
        concat_u8(
            nes.cpu.high_indirect_address,
            nes.cpu.low_indirect_address.wrapping_add(1),
        ),
        nes,
    );
}
fn add_index_to_lower_address_and_set_carry(index: u8, nes: &mut Nes) {
    let (new_val, was_overflow) = nes.cpu.lower_address.overflowing_add(index);
    nes.cpu.lower_address = new_val;
    nes.cpu.internal_carry_out = was_overflow;
}
pub fn add_x_to_low_address_byte(nes: &mut Nes) {
    add_index_to_lower_address_and_set_carry(nes.cpu.x, nes);
}
pub fn add_y_to_low_address_byte(nes: &mut Nes) {
    add_index_to_lower_address_and_set_carry(nes.cpu.y, nes);
}
pub fn add_lower_address_carry_bit_to_upper_address(nes: &mut Nes) {
    let carry_in = nes.cpu.internal_carry_out as u8;
    nes.cpu.upper_address = nes.cpu.upper_address.wrapping_add(carry_in);
}

// Pointer (indirect addressing)

pub fn take_operand_as_low_indirect_address_byte(nes: &mut Nes) {
    nes.cpu.low_indirect_address = read_mem(nes.cpu.pc, nes);
}
pub fn take_operand_as_high_indirect_address_byte(nes: &mut Nes) {
    nes.cpu.high_indirect_address = read_mem(nes.cpu.pc, nes);
}
pub fn add_x_to_low_indirect_address_byte(nes: &mut Nes) {
    nes.cpu.low_indirect_address = nes.cpu.low_indirect_address.wrapping_add(nes.cpu.x);
}

// Data read

pub fn read_from_address(nes: &mut Nes) {
    let addr = nes.cpu.get_address();
    nes.cpu.data = read_mem(addr, nes);
}
pub fn dummy_read_from_address(nes: &mut Nes) {
    let addr = nes.cpu.get_address();
    nes.cpu.data = read_mem(addr, nes);
}

pub fn dummy_read_from_stack(nes: &mut Nes) {
    nes.cpu.data = read_mem(nes.cpu.s as u16, nes);
}

pub fn dummy_read_from_pc_address(nes: &mut Nes) {
    nes.cpu.data = read_mem(nes.cpu.pc, nes);
}
pub fn dummy_read_from_indirect_address(nes: &mut Nes) {
    nes.cpu.data = read_mem(nes.cpu.get_pointer(), nes);
}

// Write data

pub fn write_to_address(nes: &mut Nes) {
    let addr = nes.cpu.get_address();
    write_mem(addr, nes.cpu.data, nes);
}

// Relative addressing (branches)

pub fn fetch_branch_offset_from_pc(nes: &mut Nes) {
    nes.cpu.branch_offset = read_mem(nes.cpu.pc, nes);
}

// Stack push

fn push_to_stack(value: u8, nes: &mut Nes) {
    let stack_addr = 0x0100 + nes.cpu.s as u16;
    write_mem(stack_addr, value, nes);
}
pub fn push_lower_pc_to_stack(nes: &mut Nes) {
    push_to_stack(nes.cpu.pc as u8, nes);
}
pub fn push_upper_pc_to_stack(nes: &mut Nes) {
    push_to_stack((nes.cpu.pc >> 8) as u8, nes);
}
pub fn push_p_to_stack_during_break_or_php(nes: &mut Nes) {
    push_to_stack(nes.cpu.get_p() | 0b0011_0000, nes);
}
pub fn push_p_to_stack_during_interrupt(nes: &mut Nes) {
    push_to_stack(nes.cpu.get_p() | 0b0010_0000, nes);
}
pub fn push_a_to_stack(nes: &mut Nes) {
    push_to_stack(nes.cpu.a, nes);
}

// Stack pull

fn pull_from_stack(nes: &mut Nes) -> u8 {
    let stack_addr = 0x0100 + nes.cpu.s as u16;
    read_mem(stack_addr, nes)
}
pub fn pull_lower_pc_from_stack(nes: &mut Nes) {
    let lower_pc = pull_from_stack(nes);
    nes.cpu.set_lower_pc(lower_pc);
}
pub fn pull_upper_pc_from_stack(nes: &mut Nes) {
    let upper_pc = pull_from_stack(nes);
    nes.cpu.set_upper_pc(upper_pc);
}
pub fn pull_p_from_stack(nes: &mut Nes) {
    let status_reg = pull_from_stack(nes);
    nes.cpu.set_p(status_reg);
}
pub fn pull_a_from_stack(nes: &mut Nes) {
    let a_reg = pull_from_stack(nes);
    nes.cpu.a = a_reg;
}

// Register operations

pub fn increment_s(nes: &mut Nes) {
    nes.cpu.s = nes.cpu.s.wrapping_add(1);
}
pub fn decrement_s(nes: &mut Nes) {
    nes.cpu.s = nes.cpu.s.wrapping_sub(1);
}
