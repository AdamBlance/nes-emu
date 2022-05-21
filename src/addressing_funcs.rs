
use crate::mem::*;
use crate::hw::*;

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
    let lower = read_mem(0xFFFE, nes);
    nes.cpu.set_lower_pc(lower);
}
pub fn fetch_upper_pc_from_interrupt_vector(nes: &mut Nes) {
    let upper = read_mem(0xFFFF, nes);
    nes.cpu.set_upper_pc(upper);
}



// Immediate

pub fn fetch_immediate_from_pc(nes: &mut Nes) {
    nes.cpu.data = read_mem(nes.cpu.pc, nes);
    nes.cpu.trace_imm = nes.cpu.data;
}

// Address

pub fn fetch_lower_address_from_pc(nes: &mut Nes) {
    nes.cpu.lower_address = read_mem(nes.cpu.pc, nes);
    nes.cpu.trace_byte2 = nes.cpu.lower_address;
}
pub fn fetch_upper_address_from_pc(nes: &mut Nes) {
    nes.cpu.upper_address = read_mem(nes.cpu.pc, nes);
    nes.cpu.trace_byte3 = nes.cpu.upper_address;
}
pub fn fetch_lower_address_from_pointer(nes: &mut Nes) {
    nes.cpu.lower_address = read_mem(nes.cpu.get_pointer(), nes);
}
pub fn fetch_upper_address_from_pointer(nes: &mut Nes) {
    nes.cpu.upper_address = read_mem(nes.cpu.get_pointer(), nes);
}
fn add_index_to_lower_address_and_set_carry(index: u8, nes: &mut Nes) {
    let (new_val, was_overflow) = nes.cpu.lower_address.overflowing_add(index);
    nes.cpu.lower_address = new_val; 
    nes.cpu.internal_carry_out = was_overflow;
}
pub fn add_x_to_lower_address(nes: &mut Nes) {
    add_index_to_lower_address_and_set_carry(nes.cpu.x, nes);
}
pub fn add_y_to_lower_address(nes: &mut Nes) {
    add_index_to_lower_address_and_set_carry(nes.cpu.y, nes);
}
pub fn add_lower_address_carry_bit_to_upper_address(nes: &mut Nes) {
    let carry_in = nes.cpu.internal_carry_out as u8;
    nes.cpu.upper_address = nes.cpu.upper_address.wrapping_add(carry_in);
}

// Pointer (indirect addressing)

pub fn fetch_lower_pointer_address_from_pc(nes: &mut Nes) {
    nes.cpu.lower_pointer = read_mem(nes.cpu.pc, nes);
}
pub fn fetch_upper_pointer_address_from_pc(nes: &mut Nes) {
    nes.cpu.upper_pointer = read_mem(nes.cpu.pc, nes);
}
pub fn increment_lower_pointer(nes: &mut Nes) {
    nes.cpu.lower_address = nes.cpu.lower_address.wrapping_add(1);
}
pub fn add_x_to_lower_pointer(nes: &mut Nes) {
    nes.cpu.lower_pointer = nes.cpu.lower_pointer.wrapping_add(nes.cpu.x);
}
pub fn add_y_to_lower_pointer(nes: &mut Nes) {
    nes.cpu.lower_pointer = nes.cpu.lower_pointer.wrapping_add(nes.cpu.y);
}

// Data read

pub fn read_from_pc(nes: &mut Nes) {
    nes.cpu.data = read_mem(nes.cpu.pc, nes);
}
pub fn read_from_address(nes: &mut Nes) {
    let addr = nes.cpu.get_address();
    nes.cpu.data = read_mem(addr, nes);
    nes.cpu.trace_stored_val = nes.cpu.data;
}
pub fn read_from_pointer(nes: &mut Nes) {
    let addr = nes.cpu.get_pointer();
    nes.cpu.data = read_mem(addr, nes);
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
pub fn push_p_to_stack(nes: &mut Nes) {
    push_to_stack(nes.cpu.get_p(), nes);
}
pub fn push_p_to_stack_with_brk_flag(nes: &mut Nes) {
    push_to_stack(nes.cpu.get_p() | 0b0001_0000, nes);
}
pub fn push_a_to_stack(nes: &mut Nes) {
    push_to_stack(nes.cpu.a, nes);
}

// Stack pull

fn pull_from_stack(nes: &mut Nes) -> u8 {
    let stack_addr = 0x0100 + nes.cpu.s as u16;
    let value = read_mem(stack_addr, nes);
    value
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
pub fn increment_x(nes: &mut Nes) {
    nes.cpu.x = nes.cpu.x.wrapping_add(1);
}
pub fn increment_y(nes: &mut Nes) {
    nes.cpu.y = nes.cpu.y.wrapping_add(1);
}
pub fn increment_data(nes: &mut Nes) {
    nes.cpu.data = nes.cpu.data.wrapping_add(1);
}
pub fn decrement_s(nes: &mut Nes) {
    nes.cpu.s = nes.cpu.s.wrapping_sub(1);
}
pub fn decrement_x(nes: &mut Nes) {
    nes.cpu.x = nes.cpu.x.wrapping_sub(1);
}
pub fn decrement_y(nes: &mut Nes) {
    nes.cpu.y = nes.cpu.y.wrapping_sub(1);
}
pub fn decrement_data(nes: &mut Nes) {
    nes.cpu.data = nes.cpu.data.wrapping_sub(1);
}

// No op

pub fn none(nes: &mut Nes) {}
