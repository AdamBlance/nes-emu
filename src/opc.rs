/* 
+------+-------------------+---+--------------------------------------------------------------+
| ACC  | Accumulator       | 1 | operates on the accumulator                                  |
| IMM  | Immediate         | 2 | 2nd byte contains operand                                    |
| ABS  | Absolute          | 3 | 2nd and 3rd bytes (lower, higher) encode address             |
| ABSX | Indexed absolute  | 3 | 2nd and 3rd bytes (lower, higher) encode address, X is added |
| ABSY | Indexed absolute  | 3 | 2nd and 3rd bytes (lower, higher) encode address, Y is added |
| ZPG  | Zero page         | 2 | 2nd byte encodes address                                     |
| ZPGX | Indexed zero page | 2 | 2nd byte encodes address, X is added (mod 2^8)               |
| ZPGY | Indexed zero page | 2 | 2nd byte encodes address, Y is added (mod 2^8)               |
| INDX | Indexed indirect  | 2 | 2nd byte encodes address, X is added (mod 2^8),              |
|      |                   |   | location and neighbour contain indirect address              |
| INDY | Indirect indexed  | 2 | 2nd byte encodes address, Y is added to value in address,    |
|      |                   |   | producing new indirect address                               |
| ---- | Implied           | 1 | address is hard coded into instruction                       |
| ---- | Relative          | 2 | used for conditional branch, 2nd byte is an offset for PC    |
| ---- | Absolute indirect | 3 | used for JMP only                                            |
+------+-------------------+---+--------------------------------------------------------------+
*/


use crate::hw::*;
use crate::util::*;
use crate::mem::*;
use Mode::*;
use Type::*;


// INSTRUCTION IMPLEMENTATIONS


fn load_a(val: u8, addr: u16, nes: &mut Nes) {
    nes.cpu.a = val;
    update_p_nz(nes.cpu.a, nes);
}

fn load_x(val: u8, addr: u16, nes: &mut Nes) {
    nes.cpu.x = val;
    update_p_nz(nes.cpu.x, nes);
}

fn load_y(val: u8, addr: u16, nes: &mut Nes) {
    nes.cpu.y = val;
    update_p_nz(nes.cpu.y, nes);
}

fn store_a(val: u8, addr: u16, nes: &mut Nes) {
    write_mem(addr, nes.cpu.a, nes);
}

fn store_x(val: u8, addr: u16, nes: &mut Nes) {
    write_mem(addr, nes.cpu.x, nes);
}

fn store_y(val: u8, addr: u16, nes: &mut Nes) {
    write_mem(addr, nes.cpu.y, nes);
}

fn transfer_a_to_x(val: u8, addr: u16, nes: &mut Nes) {
    nes.cpu.x = nes.cpu.a;
    update_p_nz(nes.cpu.x, nes);
}

fn transfer_a_to_y(val: u8, addr: u16, nes: &mut Nes) {
    nes.cpu.y = nes.cpu.a;
    update_p_nz(nes.cpu.y, nes);
}

fn transfer_s_to_x(val: u8, addr: u16, nes: &mut Nes) {
    nes.cpu.x = nes.cpu.s;
    update_p_nz(nes.cpu.x, nes);
}

fn transfer_x_to_a(val: u8, addr: u16, nes: &mut Nes) {
    nes.cpu.a = nes.cpu.x;
    update_p_nz(nes.cpu.a, nes);
}

fn transfer_x_to_s(val: u8, addr: u16, nes: &mut Nes) {
    nes.cpu.s = nes.cpu.x;
}

fn transfer_y_to_a(val: u8, addr: u16, nes: &mut Nes) {
    nes.cpu.a = nes.cpu.y;
    update_p_nz(nes.cpu.a, nes);
}

fn push_a_to_stack(val: u8, addr: u16, nes: &mut Nes) {
    stack_push(nes.cpu.a, nes);
}

fn push_p_to_stack(val: u8, addr: u16, nes: &mut Nes) {
    stack_push(p_to_byte(nes) | 0b0001_0000, nes);
}

fn pull_a_from_stack(val: u8, addr: u16, nes: &mut Nes) {
    nes.cpu.a = stack_pop(nes);
    update_p_nz(nes.cpu.a, nes);
}

fn pull_p_from_stack(val: u8, addr: u16, nes: &mut Nes) {
    let p_byte = stack_pop(nes);
    byte_to_p(p_byte, nes);
}

fn arithmetic_shift_left_acc(val: u8, addr: u16, nes: &mut Nes) {
    nes.cpu.a = shift_left(nes.cpu.a, false, nes);
    update_p_nz(nes.cpu.a, nes);
}

fn arithmetic_shift_left_rmw(val: u8, addr: u16, nes: &mut Nes) {
    let new_val = shift_left(val, false, nes);
    write_mem(addr, new_val, nes);
    update_p_nz(new_val, nes);
}

fn logical_shift_right_acc(val: u8, addr: u16, nes: &mut Nes) {
    nes.cpu.a = shift_right(nes.cpu.a, false, nes);
    update_p_nz(nes.cpu.a, nes);
}

fn logical_shift_right_rmw(val: u8, addr: u16, nes: &mut Nes) {
    let new_val = shift_right(val, false, nes);
    write_mem(addr, new_val, nes);
    update_p_nz(new_val, nes);
}

fn rotate_left_acc(val: u8, addr: u16, nes: &mut Nes) {
    nes.cpu.a = shift_left(nes.cpu.a, true, nes);
    update_p_nz(nes.cpu.a, nes);
}

fn rotate_left_rmw(val: u8, addr: u16, nes: &mut Nes) {
    let new_val = shift_left(val, true, nes);
    write_mem(addr, new_val, nes);
    update_p_nz(new_val, nes);
}

fn rotate_right_acc(val: u8, addr: u16, nes: &mut Nes) {
    nes.cpu.a = shift_right(nes.cpu.a, true, nes);
    update_p_nz(nes.cpu.a, nes);
}

fn rotate_right_rmw(val: u8, addr: u16, nes: &mut Nes) {
    let new_val = shift_right(val, true, nes);
    write_mem(addr, new_val, nes);
    update_p_nz(new_val, nes);
}

fn and(val: u8, addr: u16, nes: &mut Nes) {
    nes.cpu.a &= val;
    update_p_nz(nes.cpu.a, nes);
}

fn bit(val: u8, addr: u16, nes: &mut Nes) {
    nes.cpu.p_n = get_bit(val, 7);
    nes.cpu.p_v = get_bit(val, 6);
    nes.cpu.p_z = (nes.cpu.a & val) == 0;
}

fn xor(val: u8, addr: u16, nes: &mut Nes) {
    nes.cpu.a ^= val;
    update_p_nz(nes.cpu.a, nes);
}

fn or(val: u8, addr: u16, nes: &mut Nes) {
    nes.cpu.a |= val;
    update_p_nz(nes.cpu.a, nes);
}

fn add(val: u8, addr: u16, nes: &mut Nes) {
    add_with_carry(val, nes);
    update_p_nz(nes.cpu.a, nes);
}

fn compare_with_a(val: u8, addr: u16, nes: &mut Nes) {
    nes.cpu.p_z = nes.cpu.a == val;
    nes.cpu.p_n = is_neg(nes.cpu.a.wrapping_sub(val));
    nes.cpu.p_c = val <= nes.cpu.a;
}

fn compare_with_x(val: u8, addr: u16, nes: &mut Nes) {
    nes.cpu.p_z = nes.cpu.x == val;
    nes.cpu.p_n = is_neg(nes.cpu.x.wrapping_sub(val));
    nes.cpu.p_c = val <= nes.cpu.x;
}

fn compare_with_y(val: u8, addr: u16, nes: &mut Nes) {
    nes.cpu.p_z = nes.cpu.y == val;
    nes.cpu.p_n = is_neg(nes.cpu.y.wrapping_sub(val));
    nes.cpu.p_c = val <= nes.cpu.y;
}

fn subtract(val: u8, addr: u16, nes: &mut Nes) {
    add_with_carry(val ^ 0xFF, nes);
    update_p_nz(nes.cpu.a, nes);
}

fn decrement_rmw(val: u8, addr: u16, nes: &mut Nes) {
    let new_val = val.wrapping_sub(1);
    write_mem(addr, new_val, nes);
    update_p_nz(new_val, nes);
}

fn decrement_x(val: u8, addr: u16, nes: &mut Nes) {
    nes.cpu.x = nes.cpu.x.wrapping_sub(1);
    update_p_nz(nes.cpu.x, nes);
}

fn decrement_y(val: u8, addr: u16, nes: &mut Nes) {
    nes.cpu.y = nes.cpu.y.wrapping_sub(1);
    update_p_nz(nes.cpu.y, nes);
}

fn increment_rmw(val: u8, addr: u16, nes: &mut Nes) {
    let new_val = val.wrapping_add(1);
    write_mem(addr, new_val, nes);
    update_p_nz(new_val, nes);
}

fn increment_x(val: u8, addr: u16, nes: &mut Nes) {
    nes.cpu.x = nes.cpu.x.wrapping_add(1);
    update_p_nz(nes.cpu.x, nes);
}

fn increment_y(val: u8, addr: u16, nes: &mut Nes) {
    nes.cpu.y = nes.cpu.y.wrapping_add(1);
    update_p_nz(nes.cpu.y, nes);
}

fn break_irq(val: u8, addr: u16, nes: &mut Nes) {
    stack_push_u16(nes.cpu.pc, nes);
    stack_push(p_to_byte(nes) | 0b0001_0000, nes);
    nes.cpu.pc = read_mem_u16(0xFFFE, nes);
}

fn jump(val: u8, addr: u16, nes: &mut Nes) {
    nes.cpu.pc = addr;
}

fn jump_to_subroutine(val: u8, addr: u16, nes: &mut Nes) {
    stack_push_u16(nes.cpu.pc.wrapping_add(2), nes);
    nes.cpu.pc = addr;
}

fn return_from_interrupt(val: u8, addr: u16, nes: &mut Nes) {
    let p_reg = stack_pop(nes);
    byte_to_p(p_reg, nes);
    nes.cpu.pc = stack_pop_u16(nes);
    if nes.cpu.nmi_internal_flag {
        nes.cpu.nmi_internal_flag = false;
    }
}

fn return_from_subroutine(val: u8, addr: u16, nes: &mut Nes) {
    nes.cpu.pc = stack_pop_u16(nes).wrapping_add(1);
}

fn branch_if_carry_clear(val: u8, addr: u16, nes: &mut Nes) {
    if !nes.cpu.p_c {nes.cpu.pc = addr};
}

fn branch_if_carry_set(val: u8, addr: u16, nes: &mut Nes) {
    if nes.cpu.p_c {nes.cpu.pc = addr};
}

fn branch_if_overflow_clear(val: u8, addr: u16, nes: &mut Nes) {
    if !nes.cpu.p_v {nes.cpu.pc = addr};
}

fn branch_if_overflow_set(val: u8, addr: u16, nes: &mut Nes) {
    if nes.cpu.p_v {nes.cpu.pc = addr};
}

fn branch_if_equal(val: u8, addr: u16, nes: &mut Nes) {
    if nes.cpu.p_z {nes.cpu.pc = addr};
}

fn branch_if_not_equal(val: u8, addr: u16, nes: &mut Nes) {
    if !nes.cpu.p_z {nes.cpu.pc = addr};
}

fn branch_if_negative(val: u8, addr: u16, nes: &mut Nes) {
    if nes.cpu.p_n {nes.cpu.pc = addr};
}

fn branch_if_positive(val: u8, addr: u16, nes: &mut Nes) {
    if !nes.cpu.p_n {nes.cpu.pc = addr};
}

fn clear_carry_flag(val: u8, addr: u16, nes: &mut Nes) {
    nes.cpu.p_c = false;
}

fn clear_decimal_flag(val: u8, addr: u16, nes: &mut Nes) {
    nes.cpu.p_d = false;
}

fn clear_interrupt_flag(val: u8, addr: u16, nes: &mut Nes) {
    nes.cpu.p_i = false;
}

fn clear_overflow_flag(val: u8, addr: u16, nes: &mut Nes) {
    nes.cpu.p_v = false;
}

fn set_carry_flag(val: u8, addr: u16, nes: &mut Nes) {
    nes.cpu.p_c = true;
}

fn set_decimal_flag(val: u8, addr: u16, nes: &mut Nes) {
    nes.cpu.p_d = true;
}

fn set_interrupt_flag(val: u8, addr: u16, nes: &mut Nes) {
    nes.cpu.p_i = true;
}

fn nop(val: u8, addr: u16, nes: &mut Nes) {}


// HELPER FUNCTIONS


pub fn was_signed_overflow(orig: u8, operand: u8, result: u8) -> bool {
    // If the sign bits of A and B are the same
    // and the sign bits of A and A+B are different,
    // sign bit was corrupted (there was signed overflow)
    ((!(orig ^ operand) & (orig ^ result)) >> 7) == 1
}

pub fn byte_to_p(byte: u8, nes: &mut Nes) {
    nes.cpu.p_n = get_bit(byte, 7);
    nes.cpu.p_v = get_bit(byte, 6);
    nes.cpu.p_d = get_bit(byte, 3);
    nes.cpu.p_i = get_bit(byte, 2);
    nes.cpu.p_z = get_bit(byte, 1);
    nes.cpu.p_c = get_bit(byte, 0);
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

pub fn stack_push(val: u8, nes: &mut Nes) {
    nes.wram[0x0100 + (nes.cpu.s as usize)] = val;
    nes.cpu.s = nes.cpu.s.wrapping_sub(1);
}

pub fn stack_pop(nes: &mut Nes) -> u8 {
    nes.cpu.s = nes.cpu.s.wrapping_add(1);
    nes.wram[0x0100 + (nes.cpu.s as usize)]
}

pub fn stack_push_u16(val: u16, nes: &mut Nes) {
    stack_push((val >> 8)     as u8, nes);
    stack_push((val & 0x00FF) as u8, nes);
}

pub fn stack_pop_u16(nes: &mut Nes) -> u16 {
    let lsb = stack_pop(nes);
    let msb = stack_pop(nes);
    concat_u8(msb, lsb)
}

fn update_p_nz(val: u8, nes: &mut Nes) {
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

fn add_with_carry(val: u8, nes: &mut Nes) {
    let (result, carry) = nes.cpu.a.carrying_add(val, nes.cpu.p_c);
    nes.cpu.p_v = was_signed_overflow(nes.cpu.a, val, result);
    nes.cpu.p_c = carry;
    nes.cpu.a = result;  
}


// INSTRUCTION DEFINITIONS


#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Mode { 
    Accumulator,
    Immediate,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
    Implied,
    Relative,
    IndirectX,
    IndirectY,
    AbsoluteI,
}

impl Mode {
    pub fn num_bytes(&self) -> u16 {
        match self {
            Accumulator => 1,
            Implied     => 1,
            Immediate   => 2,
            Relative    => 2,
            IndirectX   => 2,
            IndirectY   => 2,
            ZeroPage    => 2,
            ZeroPageX   => 2,
            ZeroPageY   => 2,
            Absolute    => 3,
            AbsoluteX   => 3,
            AbsoluteY   => 3,
            AbsoluteI   => 3,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Type {
    Read,
    Write,
    ReadModifyWrite,
    Other,
}

#[derive(Copy, Clone)]
pub struct Instruction { 
    pub name: &'static str,
    pub mode: Mode,
    pub cycles: u8,
    pub page_penalty: bool,
    pub method: Type,
    pub associated_function: fn(u8, u16, &mut Nes),
}

impl Default for Instruction {
    fn default() -> Instruction {
        INSTRUCTIONS[0]
    }
}

// need to add whether 

pub static INSTRUCTIONS: [Instruction; 256] = [
    // 0
    Instruction {name:  "BRK", mode: Implied,     cycles: 7, page_penalty: false, method: Other, associated_function: break_irq},
    Instruction {name:  "ORA", mode: IndirectX,   cycles: 6, page_penalty: false, method: Read, associated_function: or},
    Instruction {name: "*JAM", mode: Implied,     cycles: 0, page_penalty: false, method: Read, associated_function: nop},
    Instruction {name: "*SLO", mode: IndirectX,   cycles: 8, page_penalty: false, method: Read, associated_function: nop},
    Instruction {name: "*NOP", mode: ZeroPage,    cycles: 3, page_penalty: false, method: Read, associated_function: nop},
    Instruction {name:  "ORA", mode: ZeroPage,    cycles: 3, page_penalty: false, method: Read, associated_function: or},
    Instruction {name:  "ASL", mode: ZeroPage,    cycles: 5, page_penalty: false, method: ReadModifyWrite, associated_function: arithmetic_shift_left_rmw},
    Instruction {name: "*SLO", mode: ZeroPage,    cycles: 5, page_penalty: false, method: Read, associated_function: nop},
    Instruction {name:  "PHP", mode: Implied,     cycles: 3, page_penalty: false, method: Other, associated_function: push_p_to_stack},
    Instruction {name:  "ORA", mode: Immediate,   cycles: 2, page_penalty: false, method: Read, associated_function: or},
    Instruction {name:  "ASL", mode: Accumulator, cycles: 2, page_penalty: false, method: Read, associated_function: arithmetic_shift_left_acc},
    Instruction {name: "*ANC", mode: Immediate,   cycles: 2, page_penalty: false, method: Read, associated_function: nop},
    Instruction {name: "*NOP", mode: Absolute,    cycles: 4, page_penalty: false, method: Read, associated_function: nop},
    Instruction {name:  "ORA", mode: Absolute,    cycles: 4, page_penalty: false, method: Read, associated_function: or},
    Instruction {name:  "ASL", mode: Absolute,    cycles: 6, page_penalty: false, method: ReadModifyWrite, associated_function: arithmetic_shift_left_rmw},
    Instruction {name: "*SLO", mode: Absolute,    cycles: 6, page_penalty: false, method: Read, associated_function: nop},
    // 1
    Instruction {name:  "BPL", mode: Relative,    cycles: 2, page_penalty: false, method: Other, associated_function: branch_if_positive},
    Instruction {name:  "ORA", mode: IndirectY,   cycles: 5, page_penalty: true,  method: Read, associated_function: or},
    Instruction {name: "*JAM", mode: Implied,     cycles: 0, page_penalty: false, method: Read, associated_function: nop},
    Instruction {name: "*SLO", mode: IndirectY,   cycles: 8, page_penalty: false, method: Read, associated_function: nop},
    Instruction {name: "*NOP", mode: ZeroPageX,   cycles: 4, page_penalty: false, method: Read, associated_function: nop},
    Instruction {name:  "ORA", mode: ZeroPageX,   cycles: 4, page_penalty: false, method: Read, associated_function: or},
    Instruction {name:  "ASL", mode: ZeroPageX,   cycles: 6, page_penalty: false, method: ReadModifyWrite, associated_function: arithmetic_shift_left_rmw},
    Instruction {name: "*SLO", mode: ZeroPageX,   cycles: 6, page_penalty: false, method: Read, associated_function: nop},
    Instruction {name:  "CLC", mode: Implied,     cycles: 2, page_penalty: false, method: Read, associated_function: clear_carry_flag},
    Instruction {name:  "ORA", mode: AbsoluteY,   cycles: 4, page_penalty: true,  method: Read, associated_function: or},
    Instruction {name: "*NOP", mode: Implied,     cycles: 2, page_penalty: false, method: Read, associated_function: nop},
    Instruction {name: "*SLO", mode: AbsoluteY,   cycles: 7, page_penalty: false, method: Read, associated_function: nop},
    Instruction {name: "*NOP", mode: AbsoluteX,   cycles: 4, page_penalty: false, method: Read, associated_function: nop},
    Instruction {name:  "ORA", mode: AbsoluteX,   cycles: 4, page_penalty: true,  method: Read, associated_function: or},
    Instruction {name:  "ASL", mode: AbsoluteX,   cycles: 7, page_penalty: false, method: ReadModifyWrite, associated_function: arithmetic_shift_left_rmw},
    Instruction {name: "*SLO", mode: AbsoluteX,   cycles: 7, page_penalty: false, method: Read, associated_function: nop},
    // 2
    Instruction {name:  "JSR", mode: Absolute,    cycles: 6, page_penalty: false, method: Read, associated_function: jump_to_subroutine},
    Instruction {name:  "AND", mode: IndirectX,   cycles: 6, page_penalty: false, method: Read, associated_function: and},
    Instruction {name: "*JAM", mode: Implied,     cycles: 0, page_penalty: false, method: Read, associated_function: nop},
    Instruction {name: "*RLA", mode: IndirectX,   cycles: 8, page_penalty: false, method: Read, associated_function: nop},
    Instruction {name:  "BIT", mode: ZeroPage,    cycles: 3, page_penalty: false, method: Read, associated_function: bit},
    Instruction {name:  "AND", mode: ZeroPage,    cycles: 3, page_penalty: false, method: Read, associated_function: and},
    Instruction {name:  "ROL", mode: ZeroPage,    cycles: 5, page_penalty: false, method: ReadModifyWrite, associated_function: rotate_left_rmw},
    Instruction {name: "*RLA", mode: ZeroPage,    cycles: 5, page_penalty: false, method: Read, associated_function: nop},
    Instruction {name:  "PLP", mode: Implied,     cycles: 4, page_penalty: false, method: Read, associated_function: pull_p_from_stack},
    Instruction {name:  "AND", mode: Immediate,   cycles: 2, page_penalty: false, method: Read, associated_function: and},
    Instruction {name:  "ROL", mode: Accumulator, cycles: 2, page_penalty: false, method: Read, associated_function: rotate_left_acc},
    Instruction {name: "*ANC", mode: Immediate,   cycles: 2, page_penalty: false, method: Read, associated_function: nop},
    Instruction {name:  "BIT", mode: Absolute,    cycles: 4, page_penalty: false, method: Read, associated_function: bit},
    Instruction {name:  "AND", mode: Absolute,    cycles: 4, page_penalty: false, method: Read, associated_function: and},
    Instruction {name:  "ROL", mode: Absolute,    cycles: 6, page_penalty: false, method: ReadModifyWrite, associated_function: rotate_left_rmw},
    Instruction {name: "*RLA", mode: Absolute,    cycles: 6, page_penalty: false, method: Read, associated_function: nop},
    // 3
    Instruction {name:  "BMI", mode: Relative,    cycles: 2, page_penalty: false, method: Read, associated_function: branch_if_negative},
    Instruction {name:  "AND", mode: IndirectY,   cycles: 5, page_penalty: true,  method: Read, associated_function: and},
    Instruction {name: "*JAM", mode: Implied,     cycles: 0, page_penalty: false, method: Read, associated_function: nop},
    Instruction {name: "*RLA", mode: IndirectY,   cycles: 8, page_penalty: false, method: Read, associated_function: nop},
    Instruction {name: "*NOP", mode: ZeroPageX,   cycles: 4, page_penalty: false, method: Read, associated_function: nop},
    Instruction {name:  "AND", mode: ZeroPageX,   cycles: 4, page_penalty: false, method: Read, associated_function: and},
    Instruction {name:  "ROL", mode: ZeroPageX,   cycles: 6, page_penalty: false, method: ReadModifyWrite, associated_function: rotate_left_rmw},
    Instruction {name: "*RLA", mode: ZeroPageX,   cycles: 6, page_penalty: false, method: Read, associated_function: nop},
    Instruction {name:  "SEC", mode: Implied,     cycles: 2, page_penalty: false, method: Read, associated_function: set_carry_flag},
    Instruction {name:  "AND", mode: AbsoluteY,   cycles: 4, page_penalty: true,  method: Read, associated_function: and},
    Instruction {name: "*NOP", mode: Implied,     cycles: 2, page_penalty: false, method: Read, associated_function: nop},
    Instruction {name: "*RLA", mode: AbsoluteY,   cycles: 8, page_penalty: false, method: Read, associated_function: nop},
    Instruction {name: "*NOP", mode: AbsoluteX,   cycles: 4, page_penalty: false, method: Read, associated_function: nop},
    Instruction {name:  "AND", mode: AbsoluteX,   cycles: 4, page_penalty: true,  method: Read, associated_function: and},
    Instruction {name:  "ROL", mode: AbsoluteX,   cycles: 7, page_penalty: false, method: ReadModifyWrite, associated_function: rotate_left_rmw},
    Instruction {name: "*RLA", mode: AbsoluteX,   cycles: 7, page_penalty: false, method: Read, associated_function: nop},
    // 4
    Instruction {name:  "RTI", mode: Implied,     cycles: 6, page_penalty: false, method: Read, associated_function: return_from_interrupt},
    Instruction {name:  "EOR", mode: IndirectX,   cycles: 6, page_penalty: false, method: Read, associated_function: xor},
    Instruction {name: "*JAM", mode: Implied,     cycles: 0, page_penalty: false, method: Read, associated_function: nop},
    Instruction {name: "*SRE", mode: IndirectX,   cycles: 8, page_penalty: false, method: Read, associated_function: nop},
    Instruction {name: "*NOP", mode: ZeroPage,    cycles: 3, page_penalty: false, method: Read, associated_function: nop},
    Instruction {name:  "EOR", mode: ZeroPage,    cycles: 3, page_penalty: false, method: Read, associated_function: xor},
    Instruction {name:  "LSR", mode: ZeroPage,    cycles: 5, page_penalty: false, method: ReadModifyWrite, associated_function: logical_shift_right_rmw},
    Instruction {name: "*SRE", mode: ZeroPage,    cycles: 5, page_penalty: false, method: Read, associated_function: nop},
    Instruction {name:  "PHA", mode: Implied,     cycles: 3, page_penalty: false, method: Read, associated_function: push_a_to_stack},
    Instruction {name:  "EOR", mode: Immediate,   cycles: 2, page_penalty: false, method: Read, associated_function: xor},
    Instruction {name:  "LSR", mode: Accumulator, cycles: 2, page_penalty: false, method: Read, associated_function: logical_shift_right_acc},
    Instruction {name: "*ALR", mode: Immediate,   cycles: 2, page_penalty: false, method: Read, associated_function: nop},
    Instruction {name:  "JMP", mode: Absolute,    cycles: 3, page_penalty: false, method: Read, associated_function: jump},
    Instruction {name:  "EOR", mode: Absolute,    cycles: 4, page_penalty: false, method: Read, associated_function: xor},
    Instruction {name:  "LSR", mode: Absolute,    cycles: 6, page_penalty: false, method: ReadModifyWrite, associated_function: logical_shift_right_rmw},
    Instruction {name: "*SRE", mode: Absolute,    cycles: 6, page_penalty: false, method: Read, associated_function: nop},
    // 5
    Instruction {name:  "BVC", mode: Relative,    cycles: 2, page_penalty: false, method: Read, associated_function: branch_if_overflow_clear},
    Instruction {name:  "EOR", mode: IndirectY,   cycles: 5, page_penalty: true,  method: Read, associated_function: xor},
    Instruction {name: "*JAM", mode: Implied,     cycles: 0, page_penalty: false, method: Read, associated_function: nop},
    Instruction {name: "*SRE", mode: IndirectY,   cycles: 8, page_penalty: false, method: Read, associated_function: nop},
    Instruction {name: "*NOP", mode: ZeroPageX,   cycles: 4, page_penalty: false, method: Read, associated_function: nop},
    Instruction {name:  "EOR", mode: ZeroPageX,   cycles: 4, page_penalty: false, method: Read, associated_function: xor},
    Instruction {name:  "LSR", mode: ZeroPageX,   cycles: 6, page_penalty: false, method: ReadModifyWrite, associated_function: logical_shift_right_rmw},
    Instruction {name: "*SRE", mode: ZeroPageX,   cycles: 6, page_penalty: false, method: Read, associated_function: nop},
    Instruction {name:  "CLI", mode: Implied,     cycles: 2, page_penalty: false, method: Read, associated_function: clear_interrupt_flag},
    Instruction {name:  "EOR", mode: AbsoluteY,   cycles: 4, page_penalty: true,  method: Read, associated_function: xor},
    Instruction {name: "*NOP", mode: Implied,     cycles: 2, page_penalty: false, method: Read, associated_function: nop},
    Instruction {name: "*SRE", mode: AbsoluteY,   cycles: 7, page_penalty: false, method: Read, associated_function: nop},
    Instruction {name: "*NOP", mode: AbsoluteX,   cycles: 4, page_penalty: false, method: Read, associated_function: nop},
    Instruction {name:  "EOR", mode: AbsoluteX,   cycles: 4, page_penalty: true,  method: Read, associated_function: xor},
    Instruction {name:  "LSR", mode: AbsoluteX,   cycles: 7, page_penalty: false, method: ReadModifyWrite, associated_function: logical_shift_right_rmw},
    Instruction {name: "*SRE", mode: AbsoluteX,   cycles: 7, page_penalty: false, method: Read, associated_function: nop},
    // 6
    Instruction {name:  "RTS", mode: Implied,     cycles: 6, page_penalty: false, method: Read, associated_function: return_from_subroutine},
    Instruction {name:  "ADC", mode: IndirectX,   cycles: 6, page_penalty: false, method: Read, associated_function: add},
    Instruction {name: "*JAM", mode: Implied,     cycles: 0, page_penalty: false, method: Read, associated_function: nop},
    Instruction {name: "*RRA", mode: IndirectX,   cycles: 8, page_penalty: false, method: Read, associated_function: nop},
    Instruction {name: "*NOP", mode: ZeroPage,    cycles: 3, page_penalty: false, method: Read, associated_function: nop},
    Instruction {name:  "ADC", mode: ZeroPage,    cycles: 4, page_penalty: false, method: Read, associated_function: add},
    Instruction {name:  "ROR", mode: ZeroPage,    cycles: 6, page_penalty: false, method: ReadModifyWrite, associated_function: rotate_right_rmw},
    Instruction {name: "*RRA", mode: ZeroPage,    cycles: 5, page_penalty: false, method: Read, associated_function: nop},
    Instruction {name:  "PLA", mode: Implied,     cycles: 4, page_penalty: false, method: Read, associated_function: pull_a_from_stack},
    Instruction {name:  "ADC", mode: Immediate,   cycles: 2, page_penalty: false, method: Read, associated_function: add},
    Instruction {name:  "ROR", mode: Accumulator, cycles: 2, page_penalty: false, method: Read, associated_function: rotate_right_acc},
    Instruction {name: "*ARR", mode: Immediate,   cycles: 2, page_penalty: false, method: Read, associated_function: nop},
    Instruction {name:  "JMP", mode: AbsoluteI,   cycles: 5, page_penalty: false, method: Read, associated_function: jump},
    Instruction {name:  "ADC", mode: Absolute,    cycles: 4, page_penalty: false, method: Read, associated_function: add},
    Instruction {name:  "ROR", mode: Absolute,    cycles: 6, page_penalty: false, method: ReadModifyWrite, associated_function: rotate_right_rmw},
    Instruction {name: "*RRA", mode: Absolute,    cycles: 6, page_penalty: false, method: Read, associated_function: nop},
    // 7
    Instruction {name:  "BVS", mode: Relative,    cycles: 2, page_penalty: false, method: Read, associated_function: branch_if_overflow_set},
    Instruction {name:  "ADC", mode: IndirectY,   cycles: 5, page_penalty: true,  method: Read, associated_function: add},
    Instruction {name: "*JAM", mode: Implied,     cycles: 0, page_penalty: false, method: Read, associated_function: nop},
    Instruction {name: "*RRA", mode: IndirectY,   cycles: 8, page_penalty: false, method: Read, associated_function: nop},
    Instruction {name: "*NOP", mode: ZeroPageX,   cycles: 4, page_penalty: false, method: Read, associated_function: nop},
    Instruction {name:  "ADC", mode: ZeroPageX,   cycles: 4, page_penalty: false, method: Read, associated_function: add},
    Instruction {name:  "ROR", mode: ZeroPageX,   cycles: 6, page_penalty: false, method: ReadModifyWrite, associated_function: rotate_right_rmw},
    Instruction {name: "*RRA", mode: ZeroPageX,   cycles: 6, page_penalty: false, method: Read, associated_function: nop},
    Instruction {name:  "SEI", mode: Implied,     cycles: 2, page_penalty: false, method: Read, associated_function: set_interrupt_flag},
    Instruction {name:  "ADC", mode: AbsoluteY,   cycles: 4, page_penalty: true,  method: Read, associated_function: add},
    Instruction {name: "*NOP", mode: Implied,     cycles: 2, page_penalty: false, method: Read, associated_function: nop},
    Instruction {name: "*RRA", mode: AbsoluteY,   cycles: 7, page_penalty: false, method: Read, associated_function: nop},
    Instruction {name: "*NOP", mode: AbsoluteX,   cycles: 4, page_penalty: false, method: Read, associated_function: nop},
    Instruction {name:  "ADC", mode: AbsoluteX,   cycles: 4, page_penalty: true,  method: Read, associated_function: add},
    Instruction {name:  "ROR", mode: AbsoluteX,   cycles: 7, page_penalty: false, method: ReadModifyWrite, associated_function: rotate_right_rmw},
    Instruction {name: "*RRA", mode: AbsoluteX,   cycles: 7, page_penalty: false, method: Read, associated_function: nop},
    // 8
    Instruction {name: "*NOP", mode: Immediate,   cycles: 2, page_penalty: false, method: Read, associated_function: nop},
    Instruction {name:  "STA", mode: IndirectX,   cycles: 6, page_penalty: false, method: Write, associated_function: store_a},
    Instruction {name: "*NOP", mode: Immediate,   cycles: 2, page_penalty: false, method: Read, associated_function: nop},
    Instruction {name: "*SAX", mode: IndirectX,   cycles: 6, page_penalty: false, method: Read, associated_function: nop},
    Instruction {name:  "STY", mode: ZeroPage,    cycles: 3, page_penalty: false, method: Write, associated_function: store_y},
    Instruction {name:  "STA", mode: ZeroPage,    cycles: 3, page_penalty: false, method: Write, associated_function: store_a},
    Instruction {name:  "STX", mode: ZeroPage,    cycles: 3, page_penalty: false, method: Write, associated_function: store_x},
    Instruction {name: "*SAX", mode: ZeroPage,    cycles: 3, page_penalty: false, method: Read, associated_function: nop},
    Instruction {name:  "DEY", mode: Implied,     cycles: 2, page_penalty: false, method: Read, associated_function: decrement_y},
    Instruction {name: "*NOP", mode: Immediate,   cycles: 2, page_penalty: false, method: Read, associated_function: nop},
    Instruction {name:  "TXA", mode: Implied,     cycles: 2, page_penalty: false, method: Read, associated_function: transfer_x_to_a},
    Instruction {name: "*XAA", mode: Immediate,   cycles: 2, page_penalty: false, method: Read, associated_function: nop},
    Instruction {name:  "STY", mode: Absolute,    cycles: 4, page_penalty: false, method: Write, associated_function: store_y},
    Instruction {name:  "STA", mode: Absolute,    cycles: 4, page_penalty: false, method: Write, associated_function: store_a},
    Instruction {name:  "STX", mode: Absolute,    cycles: 4, page_penalty: false, method: Write, associated_function: store_x},
    Instruction {name: "*SAX", mode: Absolute,    cycles: 4, page_penalty: false, method: Read, associated_function: nop},
    // 9
    Instruction {name:  "BCC", mode: Relative,    cycles: 2, page_penalty: false, method: Read, associated_function: branch_if_carry_clear},
    Instruction {name:  "STA", mode: IndirectY,   cycles: 6, page_penalty: false, method: Write, associated_function: store_a},
    Instruction {name: "*JAM", mode: Implied,     cycles: 0, page_penalty: false, method: Read, associated_function: nop},
    Instruction {name: "*SHA", mode: IndirectY,   cycles: 6, page_penalty: false, method: Read, associated_function: nop},
    Instruction {name:  "STY", mode: ZeroPageX,   cycles: 4, page_penalty: false, method: Write, associated_function: store_y},
    Instruction {name:  "STA", mode: ZeroPageX,   cycles: 4, page_penalty: false, method: Write, associated_function: store_a},
    Instruction {name:  "STX", mode: ZeroPageY,   cycles: 4, page_penalty: false, method: Write, associated_function: store_x},
    Instruction {name: "*SAX", mode: ZeroPageY,   cycles: 4, page_penalty: false, method: Read, associated_function: nop},
    Instruction {name:  "TYA", mode: Implied,     cycles: 2, page_penalty: false, method: Read, associated_function: transfer_y_to_a},
    Instruction {name:  "STA", mode: AbsoluteY,   cycles: 5, page_penalty: false, method: Write, associated_function: store_a},
    Instruction {name:  "TXS", mode: Implied,     cycles: 2, page_penalty: false, method: Read, associated_function: transfer_x_to_s},
    Instruction {name: "*SHS", mode: AbsoluteY,   cycles: 5, page_penalty: false, method: Read, associated_function: nop},
    Instruction {name: "*SHY", mode: AbsoluteX,   cycles: 5, page_penalty: false, method: Read, associated_function: nop},
    Instruction {name:  "STA", mode: AbsoluteX,   cycles: 5, page_penalty: false, method: Write, associated_function: store_a},
    Instruction {name: "*SHX", mode: AbsoluteY,   cycles: 5, page_penalty: false, method: Read, associated_function: nop},
    Instruction {name: "*SHA", mode: AbsoluteY,   cycles: 5, page_penalty: false, method: Read, associated_function: nop},
    // A
    Instruction {name:  "LDY", mode: Immediate,   cycles: 2, page_penalty: false, method: Read, associated_function: load_y},
    Instruction {name:  "LDA", mode: IndirectX,   cycles: 6, page_penalty: false, method: Read, associated_function: load_a},
    Instruction {name:  "LDX", mode: Immediate,   cycles: 2, page_penalty: false, method: Read, associated_function: load_x},
    Instruction {name: "*LAX", mode: IndirectX,   cycles: 6, page_penalty: false, method: Read, associated_function: nop},
    Instruction {name:  "LDY", mode: ZeroPage,    cycles: 3, page_penalty: false, method: Read, associated_function: load_y},
    Instruction {name:  "LDA", mode: ZeroPage,    cycles: 3, page_penalty: false, method: Read, associated_function: load_a},
    Instruction {name:  "LDX", mode: ZeroPage,    cycles: 3, page_penalty: false, method: Read, associated_function: load_x},
    Instruction {name: "*LAX", mode: ZeroPage,    cycles: 3, page_penalty: false, method: Read, associated_function: nop},
    Instruction {name:  "TAY", mode: Implied,     cycles: 2, page_penalty: false, method: Read, associated_function: transfer_a_to_y},
    Instruction {name:  "LDA", mode: Immediate,   cycles: 2, page_penalty: false, method: Read, associated_function: load_a},
    Instruction {name:  "TAX", mode: Implied,     cycles: 2, page_penalty: false, method: Read, associated_function: transfer_a_to_x},
    Instruction {name: "*LAX", mode: Immediate,   cycles: 2, page_penalty: false, method: Read, associated_function: nop},
    Instruction {name:  "LDY", mode: Absolute,    cycles: 4, page_penalty: false, method: Read, associated_function: load_y},
    Instruction {name:  "LDA", mode: Absolute,    cycles: 4, page_penalty: false, method: Read, associated_function: load_a},
    Instruction {name:  "LDX", mode: Absolute,    cycles: 4, page_penalty: false, method: Read, associated_function: load_x},
    Instruction {name: "*LAX", mode: Absolute,    cycles: 4, page_penalty: false, method: Read, associated_function: nop},
    // B
    Instruction {name:  "BCS", mode: Relative,    cycles: 2, page_penalty: false, method: Read, associated_function: branch_if_carry_set},
    Instruction {name:  "LDA", mode: IndirectY,   cycles: 5, page_penalty: true,  method: Read, associated_function: load_a},
    Instruction {name: "*JAM", mode: Implied,     cycles: 0, page_penalty: false, method: Read, associated_function: nop},
    Instruction {name: "*LAX", mode: IndirectY,   cycles: 4, page_penalty: true,  method: Read, associated_function: nop},
    Instruction {name:  "LDY", mode: ZeroPageX,   cycles: 4, page_penalty: false, method: Read, associated_function: load_y},
    Instruction {name:  "LDA", mode: ZeroPageX,   cycles: 4, page_penalty: false, method: Read, associated_function: load_a},
    Instruction {name:  "LDX", mode: ZeroPageY,   cycles: 4, page_penalty: false, method: Read, associated_function: load_x},
    Instruction {name: "*LAX", mode: ZeroPageY,   cycles: 4, page_penalty: false, method: Read, associated_function: nop},
    Instruction {name:  "CLV", mode: Implied,     cycles: 2, page_penalty: false, method: Read, associated_function: clear_overflow_flag},
    Instruction {name:  "LDA", mode: AbsoluteY,   cycles: 4, page_penalty: true,  method: Read, associated_function: load_a},
    Instruction {name:  "TSX", mode: Implied,     cycles: 2, page_penalty: false, method: Read, associated_function: transfer_s_to_x},
    Instruction {name: "*LAS", mode: AbsoluteY,   cycles: 4, page_penalty: true,  method: Read, associated_function: nop},
    Instruction {name:  "LDY", mode: AbsoluteX,   cycles: 4, page_penalty: true,  method: Read, associated_function: load_y},
    Instruction {name:  "LDA", mode: AbsoluteX,   cycles: 4, page_penalty: true,  method: Read, associated_function: load_a},
    Instruction {name:  "LDX", mode: AbsoluteY,   cycles: 4, page_penalty: true,  method: Read, associated_function: load_x},
    Instruction {name: "*LAX", mode: AbsoluteY,   cycles: 4, page_penalty: true,  method: Read, associated_function: nop},
    // C
    Instruction {name:  "CPY", mode: Immediate,   cycles: 2, page_penalty: false, method: Read, associated_function: compare_with_y},
    Instruction {name:  "CMP", mode: IndirectX,   cycles: 6, page_penalty: false, method: Read, associated_function: compare_with_a},
    Instruction {name: "*NOP", mode: Immediate,   cycles: 2, page_penalty: false, method: Read, associated_function: nop},
    Instruction {name: "*DCP", mode: IndirectX,   cycles: 8, page_penalty: false, method: Read, associated_function: nop},
    Instruction {name:  "CPY", mode: ZeroPage,    cycles: 3, page_penalty: false, method: Read, associated_function: compare_with_y},
    Instruction {name:  "CMP", mode: ZeroPage,    cycles: 3, page_penalty: false, method: Read, associated_function: compare_with_a},
    Instruction {name:  "DEC", mode: ZeroPage,    cycles: 6, page_penalty: false, method: ReadModifyWrite, associated_function: decrement_rmw},
    Instruction {name: "*DCP", mode: ZeroPage,    cycles: 5, page_penalty: false, method: Read, associated_function: nop},
    Instruction {name:  "INY", mode: Implied,     cycles: 2, page_penalty: false, method: Read, associated_function: increment_y},
    Instruction {name:  "CMP", mode: Immediate,   cycles: 2, page_penalty: false, method: Read, associated_function: compare_with_a},
    Instruction {name:  "DEX", mode: Implied,     cycles: 2, page_penalty: false, method: Read, associated_function: decrement_x},
    Instruction {name: "*AXS", mode: Immediate,   cycles: 2, page_penalty: false, method: Read, associated_function: nop},
    Instruction {name:  "CPY", mode: Absolute,    cycles: 4, page_penalty: false, method: Read, associated_function: compare_with_y},
    Instruction {name:  "CMP", mode: Absolute,    cycles: 4, page_penalty: false, method: Read, associated_function: compare_with_a},
    Instruction {name:  "DEC", mode: Absolute,    cycles: 6, page_penalty: false, method: ReadModifyWrite, associated_function: decrement_rmw},
    Instruction {name: "*DCP", mode: Absolute,    cycles: 6, page_penalty: false, method: Read, associated_function: nop},
    // D
    Instruction {name:  "BNE", mode: Relative,    cycles: 2, page_penalty: false, method: Read, associated_function: branch_if_not_equal},
    Instruction {name:  "CMP", mode: IndirectY,   cycles: 5, page_penalty: true,  method: Read, associated_function: compare_with_a},
    Instruction {name: "*JAM", mode: Implied,     cycles: 0, page_penalty: false, method: Read, associated_function: nop},
    Instruction {name: "*DCP", mode: IndirectY,   cycles: 8, page_penalty: false, method: Read, associated_function: nop},
    Instruction {name: "*NOP", mode: ZeroPageX,   cycles: 4, page_penalty: false, method: Read, associated_function: nop},
    Instruction {name:  "CMP", mode: ZeroPageX,   cycles: 4, page_penalty: false, method: Read, associated_function: compare_with_a},
    Instruction {name:  "DEC", mode: ZeroPageX,   cycles: 6, page_penalty: false, method: ReadModifyWrite, associated_function: decrement_rmw},
    Instruction {name: "*DCP", mode: ZeroPageX,   cycles: 6, page_penalty: false, method: Read, associated_function: nop},
    Instruction {name:  "CLD", mode: Implied,     cycles: 2, page_penalty: false, method: Read, associated_function: clear_decimal_flag},
    Instruction {name:  "CMP", mode: AbsoluteY,   cycles: 4, page_penalty: true,  method: Read, associated_function: compare_with_a},
    Instruction {name: "*NOP", mode: Implied,     cycles: 2, page_penalty: false, method: Read, associated_function: nop},
    Instruction {name: "*DCP", mode: AbsoluteY,   cycles: 7, page_penalty: false, method: Read, associated_function: nop},
    Instruction {name: "*NOP", mode: AbsoluteX,   cycles: 4, page_penalty: false, method: Read, associated_function: nop},
    Instruction {name:  "CMP", mode: AbsoluteX,   cycles: 4, page_penalty: true,  method: Read, associated_function: compare_with_a},
    Instruction {name:  "DEC", mode: AbsoluteX,   cycles: 7, page_penalty: false, method: ReadModifyWrite, associated_function: decrement_rmw},
    Instruction {name: "*DCP", mode: AbsoluteX,   cycles: 7, page_penalty: false, method: Read, associated_function: nop},
    // E
    Instruction {name:  "CPX", mode: Immediate,   cycles: 2, page_penalty: false, method: Read, associated_function: compare_with_x},
    Instruction {name:  "SBC", mode: IndirectX,   cycles: 6, page_penalty: false, method: Read, associated_function: subtract},
    Instruction {name: "*NOP", mode: Immediate,   cycles: 2, page_penalty: false, method: Read, associated_function: nop},
    Instruction {name: "*ISB", mode: IndirectX,   cycles: 8, page_penalty: false, method: Read, associated_function: nop},
    Instruction {name:  "CPX", mode: ZeroPage,    cycles: 3, page_penalty: false, method: Read, associated_function: compare_with_x},
    Instruction {name:  "SBC", mode: ZeroPage,    cycles: 3, page_penalty: false, method: Read, associated_function: subtract},
    Instruction {name:  "INC", mode: ZeroPage,    cycles: 5, page_penalty: false, method: ReadModifyWrite, associated_function: increment_rmw},
    Instruction {name: "*ISB", mode: ZeroPage,    cycles: 5, page_penalty: false, method: Read, associated_function: nop},
    Instruction {name:  "INX", mode: Implied,     cycles: 2, page_penalty: false, method: Read, associated_function: increment_x},
    Instruction {name:  "SBC", mode: Immediate,   cycles: 2, page_penalty: false, method: Read, associated_function: subtract},
    Instruction {name:  "NOP", mode: Implied,     cycles: 2, page_penalty: false, method: Read, associated_function: nop},
    Instruction {name: "*SBC", mode: Immediate,   cycles: 2, page_penalty: false, method: Read, associated_function: subtract},
    Instruction {name:  "CPX", mode: Absolute,    cycles: 4, page_penalty: false, method: Read, associated_function: compare_with_x},
    Instruction {name:  "SBC", mode: Absolute,    cycles: 4, page_penalty: false, method: Read, associated_function: subtract},
    Instruction {name:  "INC", mode: Absolute,    cycles: 6, page_penalty: false, method: ReadModifyWrite, associated_function: increment_rmw},
    Instruction {name: "*ISB", mode: Absolute,    cycles: 6, page_penalty: false, method: Read, associated_function: nop},
    // F
    Instruction {name:  "BEQ", mode: Relative,    cycles: 2, page_penalty: false, method: Read, associated_function: branch_if_equal},
    Instruction {name:  "SBC", mode: IndirectY,   cycles: 5, page_penalty: true,  method: Read, associated_function: subtract},
    Instruction {name: "*JAM", mode: Implied,     cycles: 0, page_penalty: false, method: Read, associated_function: nop},
    Instruction {name: "*ISB", mode: IndirectY,   cycles: 8, page_penalty: false, method: Read, associated_function: nop},
    Instruction {name: "*NOP", mode: ZeroPageX,   cycles: 4, page_penalty: false, method: Read, associated_function: nop},
    Instruction {name:  "SBC", mode: ZeroPageX,   cycles: 4, page_penalty: false, method: Read, associated_function: subtract},
    Instruction {name:  "INC", mode: ZeroPageX,   cycles: 6, page_penalty: false, method: ReadModifyWrite, associated_function: increment_rmw},
    Instruction {name: "*ISB", mode: ZeroPageX,   cycles: 6, page_penalty: false, method: Read, associated_function: nop},
    Instruction {name:  "SED", mode: Implied,     cycles: 2, page_penalty: false, method: Read, associated_function: set_decimal_flag},
    Instruction {name:  "SBC", mode: AbsoluteY,   cycles: 4, page_penalty: true,  method: Read, associated_function: subtract},
    Instruction {name: "*NOP", mode: Implied,     cycles: 2, page_penalty: false, method: Read, associated_function: nop},
    Instruction {name: "*ISB", mode: AbsoluteY,   cycles: 8, page_penalty: false, method: Read, associated_function: nop},
    Instruction {name: "*NOP", mode: AbsoluteX,   cycles: 7, page_penalty: false, method: Read, associated_function: nop},
    Instruction {name:  "SBC", mode: AbsoluteX,   cycles: 4, page_penalty: true,  method: Read, associated_function: subtract},
    Instruction {name:  "INC", mode: AbsoluteX,   cycles: 7, page_penalty: false, method: ReadModifyWrite, associated_function: increment_rmw},
    Instruction {name: "*ISB", mode: AbsoluteX,   cycles: 7, page_penalty: false, method: Read, associated_function: nop},
];
