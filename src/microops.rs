use crate::hw::*;
use crate::util::*;
use crate::mem::*;

fn add_index_to_address_lsb_and_set_carry(index: u8, nes: &mut Nes) {
    let (new_val, was_overflow) = nes.cpu.byte1.overflowing_add(index);
    nes.cpu.byte1 = new_val; 
    nes.cpu.addr_lsb_carry = was_overflow;
}

pub fn push_to_stack(value: u8, nes: &mut Nes) {
    let stack_addr = 0x0100 + nes.cpu.s as u16;
    write_mem(stack_addr, value, nes);
}

pub fn pull_from_stack(nes: &mut Nes) -> u8 {
    let stack_addr = 0x0100 + nes.cpu.s as u16;
    read_mem(stack_addr, nes)
}

pub fn p_to_byte(nes: &Nes) -> u8 {
    (if nes.cpu.p_n {0b1000_0000} else {0}) | 
    (if nes.cpu.p_v {0b0100_0000} else {0}) | 
                     0b0010_0000            |
    (if nes.cpu.p_d {0b0000_1000} else {0}) | 
    (if nes.cpu.p_i {0b0000_0100} else {0}) | 
    (if nes.cpu.p_z {0b0000_0010} else {0}) | 
    (if nes.cpu.p_c {0b0000_0001} else {0})
}

// https://retrocomputing.stackexchange.com/questions/145/why-does-6502-indexed-lda-take-an-extra-cycle-at-page-boundaries
// 6502 only has 8-bit adder

// this is wasteful, but can maybe use closures later? 
// like write a function that returns a closure that will use x or y as an index depending on what
// is passed

// I totally need to try this, sounds really cool
// https://doc.rust-lang.org/rustc/profile-guided-optimization.html



fn fetch_opcode(nes: &mut Nes) {
    nes.cpu.opcode = read_mem(nes.cpu.pc, nes);
    nes.cpu.inc_pc();
}

fn fetch_byte1(nes: &mut Nes) {
    nes.cpu.byte1 = read_mem(nes.cpu.pc, nes);
    nes.cpu.inc_pc();
}

fn fetch_byte2(nes: &mut Nes) {
    nes.cpu.byte2 = read_mem(nes.cpu.pc, nes);
    nes.cpu.inc_pc();
}

fn add_x_to_address_lsb(nes: &mut Nes) {
    add_index_to_address_lsb_and_set_carry(nes.cpu.x, nes);
}

fn add_y_to_address_lsb(nes: &mut Nes) {
    add_index_to_address_lsb_and_set_carry(nes.cpu.y, nes);
}

fn push_upper_pc_to_stack(nes: &mut Nes) {
    push_to_stack((nes.cpu.pc >> 8) as u8, nes);
    nes.cpu.dec_s();
}

fn push_lower_pc_to_stack(nes: &mut Nes) {
    push_to_stack(nes.cpu.pc as u8, nes);
    nes.cpu.dec_s();
}


fn pull_lower_pc_from_stack(nes: &mut Nes) {
    pull_from_stack(nes);
    nes.cpu.inc_s();
}

fn pull_upper_pc_from_stack(nes: &mut Nes) {
    pull_from_stack(nes);
    // no stack pointer increment because this is the last step
}


const BREAK_FLAG: u8  = 0b0001_0000;
const IRQ_VECTOR: u16 = 0xFFFE;

fn push_p_to_stack_during_break(nes: &mut Nes) {
    push_to_stack(p_to_byte(nes) | BREAK_FLAG, nes);
}

fn fetch_lower_pc_from_irq_vector(nes: &mut Nes) {
    nes.cpu.set_lower_pc(read_mem(IRQ_VECTOR, nes));
}

// https://stackoverflow.com/questions/28587698/whats-the-difference-between-placing-mut-before-a-variable-name-and-after-the
fn fetch_upper_pc_from_irq_vector(nes: &mut Nes) {
    nes.cpu.set_upper_pc(read_mem(IRQ_VECTOR+1, nes));
}

