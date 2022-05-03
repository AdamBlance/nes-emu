
// val  - either an immediate or a value pulled from memory
// addr - memory address to write the result into
// both parameters 

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

fn no_op(val: u8, addr: u16, nes: &mut Nes) {}
