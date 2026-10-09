use crate::emulator::nes_state::NesState;

impl NesState {
    // Program counter
    pub fn increment_pc(&mut self) {
        self.cpu.reg.pc = self.cpu.reg.pc.wrapping_add(1);
    }
    pub fn copy_address_to_pc(&mut self) {
        self.cpu.reg.pc = self.cpu.ireg.address;
    }
    pub fn fetch_lower_pc_from_interrupt_vector(&mut self) {
        let lower = read_mem(self.cpu.interrupts.interrupt_vector, nes);
        self.cpu.reg.pc = self.cpu.reg.pc.with_lower(lower);
    }
    pub fn fetch_upper_pc_from_interrupt_vector(&mut self) {
        let upper = read_mem(self.cpu.interrupts.interrupt_vector + 1, nes);
        self.cpu.set_upper_pc(upper);
    }

    // Immediate addressing
    pub fn fetch_immediate_from_pc(&mut self) {
        self.cpu.ireg.data = read_mem(self.cpu.reg.pc, nes);
    }

    // Address

    pub fn take_operand_as_low_address_byte(&mut self) {
        self.cpu.ireg.lower_address = read_mem(self.cpu.reg.pc, nes);
    }
    pub fn take_operand_as_high_address_byte(&mut self) {
        self.cpu.ireg.upper_address = read_mem(self.cpu.reg.pc, nes);
    }
    pub fn fetch_low_address_byte_using_indirect_address(&mut self) {
        self.cpu.ireg.lower_address = read_mem(self.cpu.get_pointer(), nes);
    }
    pub fn fetch_high_address_byte_using_indirect_address(&mut self) {
        self.cpu.ireg.upper_address = read_mem(
            concat_u8(
                self.cpu.ireg.high_indirect_address,
                self.cpu.ireg.low_indirect_address.wrapping_add(1),
            ),
            nes,
        );
    }
    fn add_index_to_lower_address_and_set_carry(index: u8, &mut self) {
        let (new_val, was_overflow) = self.cpu.ireg.lower_address.overflowing_add(index);
        self.cpu.ireg.lower_address = new_val;
        self.cpu.ireg.carry_out = was_overflow;
    }
    pub fn add_x_to_low_address_byte(&mut self) {
        add_index_to_lower_address_and_set_carry(self.cpu.reg.x, nes);
    }
    pub fn add_y_to_low_address_byte(&mut self) {
        add_index_to_lower_address_and_set_carry(self.cpu.reg.y, nes);
    }
    pub fn add_lower_address_carry_bit_to_upper_address(&mut self) {
        let carry_in = self.cpu.ireg.carry_out as u8;
        self.cpu.ireg.upper_address = self.cpu.ireg.upper_address.wrapping_add(carry_in);
    }

    // Pointer (indirect addressing)

    pub fn take_operand_as_low_indirect_address_byte(&mut self) {
        self.cpu.ireg.low_indirect_address = read_mem(self.cpu.reg.pc, nes);
    }
    pub fn take_operand_as_high_indirect_address_byte(&mut self) {
        self.cpu.ireg.high_indirect_address = read_mem(self.cpu.reg.pc, nes);
    }
    pub fn add_x_to_low_indirect_address_byte(&mut self) {
        self.cpu.ireg.low_indirect_address = self.cpu.ireg.low_indirect_address.wrapping_add(self.cpu.reg.x);
    }

    // Data read

    pub fn read_from_address(&mut self) {
        let addr = self.cpu.get_address();
        self.cpu.ireg.data = read_mem(addr, nes);
    }
    pub fn dummy_read_from_address(&mut self) {
        let addr = self.cpu.get_address();
        self.cpu.ireg.data = read_mem(addr, nes);
    }

    pub fn dummy_read_from_stack(&mut self) {
        self.cpu.ireg.data = read_mem(self.cpu.reg.s as u16, nes);
    }

    pub fn dummy_read_from_pc_address(&mut self) {
        self.cpu.ireg.data = read_mem(self.cpu.reg.pc, nes);
    }
    pub fn dummy_read_from_indirect_address(&mut self) {
        self.cpu.ireg.data = read_mem(self.cpu.get_pointer(), nes);
    }

    // Write data

    pub fn write_to_address(&mut self) {
        let addr = self.cpu.get_address();
        write_mem(addr, self.cpu.ireg.data, nes);
    }

    pub fn dummy_write_to_address(&mut self) {
        write_to_address(nes);
    }

    // Relative addressing (branches)

    pub fn fetch_branch_offset_from_pc(&mut self) {
        // self.cpu.ireg.branch_offset = read_mem(self.cpu.reg.pc, nes);
    }

    pub fn add_branch_offset_to_lower_pc_and_set_carry(&mut self) {
        let (new_pcl, overflow) = (self.cpu.reg.pc as u8)
            .overflowing_add_signed(self.cpu.ireg.branch_offset as i8);
        self.cpu.set_lower_pc(new_pcl);
        self.cpu.ireg.carry_out = overflow;
    }

    pub fn fix_upper_pc_after_page_crossing_branch(&mut self) {
        if (self.cpu.ireg.branch_offset as i8).is_negative() {
            self.cpu.reg.pc = self.cpu.reg.pc.wrapping_sub(1 << 8);
        } else {
            self.cpu.reg.pc = self.cpu.reg.pc.wrapping_add(1 << 8);
        }
    }

    // Stack push

    fn push_to_stack(value: u8, &mut self) {
        let stack_addr = 0x0100 + self.cpu.reg.s as u16;
        write_mem(stack_addr, value, nes);
    }
    pub fn push_lower_pc_to_stack(&mut self) {
        push_to_stack(self.cpu.reg.pc as u8, nes);
    }
    pub fn push_upper_pc_to_stack(&mut self) {
        push_to_stack((self.cpu.reg.pc >> 8) as u8, nes);
    }
    pub fn push_p_to_stack_during_break_or_php(&mut self) {
        push_to_stack(self.cpu.get_p() | 0b0011_0000, nes);
    }
    pub fn push_p_to_stack_during_interrupt(&mut self) {
        push_to_stack(self.cpu.get_p() | 0b0010_0000, nes);
    }
    pub fn push_a_to_stack(&mut self) {
        push_to_stack(self.cpu.reg.a, nes);
    }

    // Stack pull

    fn pull_from_stack(&mut self) -> u8 {
        let stack_addr = 0x0100 + self.cpu.reg.s as u16;
        read_mem(stack_addr, nes)
    }
    pub fn pull_lower_pc_from_stack(&mut self) {
        let lower_pc = pull_from_stack(nes);
        self.cpu.set_lower_pc(lower_pc);
    }
    pub fn pull_upper_pc_from_stack(&mut self) {
        let upper_pc = pull_from_stack(nes);
        self.cpu.set_upper_pc(upper_pc);
    }
    pub fn pull_p_from_stack(&mut self) {
        let status_reg = pull_from_stack(nes);
        self.cpu.set_p(status_reg);
    }
    pub fn pull_a_from_stack(&mut self) {
        let a_reg = pull_from_stack(nes);
        self.cpu.reg.a = a_reg;
    }

    // Register operations

    pub fn increment_s(&mut self) {
        self.cpu.reg.s = self.cpu.reg.s.wrapping_add(1);
    }
    pub fn decrement_s(&mut self) {
        self.cpu.reg.s = self.cpu.reg.s.wrapping_sub(1);
    }

}

