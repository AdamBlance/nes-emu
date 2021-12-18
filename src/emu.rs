mod opc;
mod mem;



fn concat_u8(msb: u8, lsb: u8) -> u16 {
    ((msb as u16) << 8) + (lsb as u16)
}

fn was_overflow(original: u8, operand: u8, result: u8) -> bool {
    ( !(original ^ operand) & (original ^ sum) ) >> 7 == 1;
}

fn update_p_n_z(result: u8, cpu: &Cpu) {
    cpu.p_z = result == 0;
    cpu.p_n = (result as i8).is_negative();
}

pub fn emulate(cart: &Cartridge) {
    
    // Create and init hardware

    let mut cpu = Cpu {
        a: 0, 
        x: 0, 
        y: 0, 
        s: 0, 
        p_n: false,
        p_v: false,
        p_b: false,
        p_i: false,
        p_z: false,
        p_c: false,
        pc: concat_u8(cart.prg_rom[0xFFFD], cart.prg_rom[0xFFFC]),
        cycles: 0, 
    };

    let mut wram: [u8; 2048] = [0; 2048];






    // http://www.oxyron.de/html/opcodes02.html
    // https://wiki.nesdev.org/w/index.php/CPU_unofficial_opcodes

    // wherever N is set, Z is set
    // N and Z are set very often by most instuctions
    // V is set by very few (ADC/SBC/PLP/RTI/BIT/CLV)
    // C is set by more things than V but not too many
    // maybe just write a function "update status reg" or something
    // pass in the cpu and let it do everything




    loop {
        let (opcode, arg1, arg2) = read_three_bytes(cpu.pc, &wram, cart);
        addr_mode = opc::get_addr_mode(opcode)

        // This is the value that the instruction does computation with
        // The instruction's addressing mode determines how the value is read from memory
        // The value can also be an immediate
        let instr_val = match addr_mode {
            opc::Implied => 0,
            opc::Immediate => arg1,
            opc::Absolute  => read_mem(concat_u8(arg2, arg1), &wram, cart),
            opc::AbsoluteX => read_mem(concat_u8(arg2, arg1) + cpu.x, &wram, cart),
            opc::AbsoluteY => read_mem(concat_u8(arg2, arg1) + cpu.y, &wram, cart),
            opc::ZeroPage  => wram[arg1],
            opc::ZeroPageX => wram[arg1.wrapping_add(cpu.x)],
            opc::ZeroPageY => wram[arg1.wrapping_add(cpu.y)],
            opc::Relative  => cpu.pc.wrapping_add_signed(arg1 as i8),
            opc::IndirectX => {
                let zp_addr_lsb = arg1.wrapping_add(cpu.x);
                let zp_addr_msb = zp_addr_lsb.wrapping_add(1);
                read_mem(concat_u8(wram[zp_addr_msb], wram[zp_addr_lsb]), &wram, cart)
            },
            opc::IndirectY => {
                let zp_lsb = wram[arg1];
                let zp_msb = wram[arg1.y.wrapping_add(1)];
                read_mem(concat_u8(zp_msb, zp_lsb) + (cpu.y as u16), &wram, cart)
            },
            opc::AbsoluteIndirect => {
                0
            },
        }

        let acc_val = match opcode {
            // ADC
            0x6D | 0x7D | 0x79 | 0x69 | 0x61 | 0x71 | 0x65 | 0x75 => {
                let (sum, carry) = cpu.a.carrying_add(instr_val, cpu.p_c));
                update_p_n_z(sum, &mut cpu);
                cpu.p_v = was_overflow(cpu.a, instr_val, sum);
                cpu.p_c = carry;
                cpu.a = sum;
            },

            // AND
            0x2D | 0x3D | 0x39 | 0x29 | 0x21 | 0x31 | 0x25 | 0x35 => {
                cpu.a &= instr_val;
                update_p_n_z(cpu.a, &mut cpu);
            },

            // EOR
            0x4D | 0x5D | 0x59 | 0x49 | 0x41 | 0x51 | 0x45 | 0x55 => {
                cpu.a ^= instr_val;
                update_p_n_z(cpu.a, &mut cpu);
            },

            // SBC
            0xED | 0xFD | 0xF9 | 0xE9 | 0xE1 | 0xF1 | 0xE5 | 0xF5 => {
                let (sum, borrow) = cpu.a.borrowing_sub(instr_val, !cpu.p_c));
                update_p_n_z(sum, &mut cpu);
                cpu.p_v = was_overflow(cpu.a, instr_val, sum);
                cpu.p_c = !borrow;
                cpu.a = sum;
            },
            // LDA
            0xAD | 0xBD | 0xB9 | 0xA9 | 0xA1 | 0xB1 | 0xA5 | 0xB5 => {
                
            },
            // TXA
            0x8A => {},
            // TYA
            0x98 => {},
            // PLA
            0x68 => {}
        }

        match opcode {


            // ASL
            0x0E | 0x1E | 0x0A | 0x06 | 0x16 => {}
            // BCC
            0x90 => {}
            // BCS
            0xB0 => {}
            // BEQ
            0xF0 => {}
            // BMI
            0x30 => {}
            // BNE
            0xD0 => {}
            // BPL
            0x10 => {}
            // BVC
            0x50 => {}
            // BVS
            0x70 => {}
            // BIT
            0x2C | 0x24 => {}
            // BRK
            0x00 => {}
            // CLC 
            0x18 => {}
            // CLI 
            0x58 => {}
            // CLV 
            0xB8 => {}
            // CMP
            0xCD | 0xDD | 0xD9 | 0xC9 | 0xC1 | 0xD1 | 0xC5 | 0xD5 => {}
            // CPX
            0xEC | 0xE0 | 0xE4 => {}
            // CPY
            0xCC | 0xC0 | 0xC4 => {}
            // DEC
            0xCE | 0xDE | 0xC6 | 0xD6 => {}
            // DEX
            0xCA => {}
            // DEY
            0x88 => {}

            // INC
            0xEE | 0xFE | 0xE6 | 0xF6 => {}
            // INX
            0xE8 => {}
            // INY
            0xC8 => {}
            // JMP 
            0x4C | 0x6C => {}
            // JSR
            0x20 => {}

            // LDX
            0xAE | 0xBE | 0xA2 | 0xA6 | 0xB6 => {}
            // LDY
            0xAC | 0xBC | 0xA0 | 0xA4 | 0xB4 => {}            
            // LSR
            0x4E | 0x5E | 0x4A | 0x46 | 0x56 => {}
            // NOP
            0xEA => {}
            // ORA
            0x0D | 0x1D | 0x19 | 0x09 | 0x01 | 0x11 | 0x05 | 0x15 => {}
            // PHA
            0x48 => {}
            // PHP
            0x08 => {}

            // PLP
            0x28 => {}
            // ROL
            0x2E | 0x3E | 0x2A | 0x26 | 0x36 => {}
            // ROR
            0x6E | 0x7E | 0x6A | 0x66 | 0x76 => {}
            // RTI
            0x40 => {}
            // RTS
            0x60 => {}

            // SEC
            0x38 => {}
            // SEI
            0x78 => {}
            // STA
            0x8D | 0x9D | 0x99 | 0x81 | 0x91 | 0x85 | 0x95 => {}
            // STX
            0x8E | 0x86 | 0x96 => {}        
            // STY
            0x8C | 0x84 | 0x94 => {}            
            // TAX
            0xAA => {}
            // TAY
            0xA8 => {}
            // TSX
            0xBA => {}

            // TXS
            0x9A => {}


            _ => {},
        }

    }



