mod opc;
mod mem;


 
 fn get_addr_mode(opcode: u8) -> Mode {
     
     // Match outliers
     match opcode {
         0x20 => Mode::Absolute,
         0x6C => Mode::AbsoluteIndirect,
     }
 
     // Determine addressing mode from opcode bits
     // https://wiki.nesdev.org/w/index.php/CPU_unofficial_opcodes
     let grp_lsb = (opcode & 0b00000001);
     let grp_msb = (opcode & 0b00000010) >> 1;
     let col     = (opcode & 0b00011100) >> 2;
     let row = (opcode & 0b11100000) >> 5;
 
     match (grp_msb, grp_lsb, col, row) {
         (_,1,2,_) | (_,0,0,4..=7) => Mode::Immediate,
         (_,1,6,_) | (1,_,7,4..=5) => Mode::AbsoluteY,
         (1,_,5,4..=5)             => Mode::ZeroPageY,
         (_,_,3,_)                 => Mode::Absolute,
         (_,_,7,_)                 => Mode::AbsoluteX,
         (_,_,1,_)                 => Mode::ZeroPage,
         (_,_,5,_)                 => Mode::ZeroPageX
         (_,1,0,_)                 => Mode::IndirectX,
         (_,1,4,_)                 => Mode::IndirectY,
         (0,0,4,_)                 => Mode::Relative,
         _                         => Mode::Implied,
     }
 }

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


fn exec_instr(opcode: u8, arg1: u8, arg2: u8, &mut cpu: Cpu, &mut wram: [u8], &cart: Cartridge) {
    addr_mode = get_addr_mode(opcode)
    
    let val_addr = match addr_mode {
        // opc::Implied => 0,
        // opc::Immediate => arg1,
        opc::Absolute  => concat_u8(arg2, arg1),
        opc::AbsoluteX => concat_u8(arg2, arg1).wrapping_add(cpu.x),
        opc::AbsoluteY => concat_u8(arg2, arg1).wrappind_add(cpu.y),
        opc::ZeroPage  => arg1 as u16,
        opc::ZeroPageX => arg1.wrapping_add(cpu.x) as u16,
        opc::ZeroPageY => arg1.wrapping_add(cpu.y) as u16,
        opc::Relative  => cpu.pc.wrapping_add_signed(arg1 as i8),
        opc::IndirectX => {
            let zp_addr_lsb = arg1.wrapping_add(cpu.x);
            let zp_addr_msb = zp_addr_lsb.wrapping_add(1);
            concat_u8(wram[zp_addr_msb], wram[zp_addr_lsb])
        },
        opc::IndirectY => {
            let zp_lsb = wram[arg1];
            let zp_msb = wram[arg1.y.wrapping_add(1)];
            concat_u8(zp_msb, zp_lsb).wrapping_add(cpu.y as u16)
        },
    }

    let instr_val = read_mem(val_addr, &wram, &cart);
    
    let acc_val = match opcode {
        0x6D | 0x7D | 0x79 | 0x69 | 0x61 | 0x71 | 0x65 | 0x75 => { // ADC
            let (sum, carry) = cpu.a.carrying_add(instr_val, cpu.p_c));
            cpu.p_v = was_overflow(cpu.a, instr_val, sum);
            cpu.p_c = carry;
            sum
        },
        0xED | 0xFD | 0xF9 | 0xE9 | 0xE1 | 0xF1 | 0xE5 | 0xF5 => { // SBC
            let (sum, borrow) = cpu.a.borrowing_sub(instr_val, !cpu.p_c));
            cpu.p_v = was_overflow(cpu.a, instr_val, sum);
            cpu.p_c = !borrow;
            sum
        },        
        0x0A => { // ASL
            cpu.p_c = cpu.a > 127;
            cpu.a << 1
        },
        0x4A => { // LSR 
            cpu.p_c = cpu.a & 0x01 == 1;
            cpu.a >> 1
        },
        0x2A => { // ROL
            cpu.p_c = cpu.a > 127;
            cpu.a.rotate_left(1)
        },
        0x6A => { // ROR
            cpu.p_c = cpu.a & 0x01 == 1;
            cpu.a.rotate_right(1)
        },
        0x2D | 0x3D | 0x39 | 0x29 | 0x21 | 0x31 | 0x25 | 0x35 => cpu.a & instr_val, // AND
        0x4D | 0x5D | 0x59 | 0x49 | 0x41 | 0x51 | 0x45 | 0x55 => cpu.a ^ instr_val, // EOR
        0xAD | 0xBD | 0xB9 | 0xA9 | 0xA1 | 0xB1 | 0xA5 | 0xB5 => instr_val, // LDA
        0x8A => cpu.x, // TXA
        0x98 => cpu.y, // TXY
        0x68 => wram[cpu.s] // PLA
        // ORA
        0x0D | 0x1D | 0x19 | 0x09 | 0x01 | 0x11 | 0x05 | 0x15 => {}
    }
    
    update_p_n_z(acc_val, &mut cpu);
    cpu.a = acc_val;

    let x_val = match opcode {
        // DEX
        0xCA => {}
        // LDX
        0xAE | 0xBE | 0xA2 | 0xA6 | 0xB6 => {}
        // INX
        0xE8 => {}
        // TAX
        0xAA => {}
        // TSX
        0xBA => {}
    }

    let y_val = match opcode {
        // DEY
        0x88 => {}
        // LDY
        0xAC | 0xBC | 0xA0 | 0xA4 | 0xB4 => {}   
        // INY
        0xC8 => {}
        // TAY
        0xA8 => {}
    }

    let s_val = match opcode {
        // TXS
        0x9A => {}
        // PHA
        0x48 => {}
        // PHP
        0x08 => {}
    }

    let mem_val = match opcode {
        // ASL
        0x0E | 0x1E | 0x06 | 0x16 => {
            cpu.p_c = instr_val > 127;
            instr_val << 1
        },
        // LSR
        0x4E | 0x5E | 0x46 | 0x56 => {
            cpu.p_c = instr_val & 0x01 == 1;
            instr_val >> 1
        }


        // DEC
        0xCE | 0xDE | 0xC6 | 0xD6 => {}
        // INC
        0xEE | 0xFE | 0xE6 | 0xF6 => {}
        // JMP 
        0x4C | 0x6C => {}
        // JSR
        0x20 => {}
        // NOP
        0xEA => {}


        // ROL
        0x2E | 0x3E | 0x26 | 0x36 => {}
        // ROR
        0x6E | 0x7E | 0x66 | 0x76 => {}
        // RTI
        0x40 => {}
        // RTS
        0x60 => {}
        // STA
        0x8D | 0x9D | 0x99 | 0x81 | 0x91 | 0x85 | 0x95 => {}
        // STX
        0x8E | 0x86 | 0x96 => cpu.x        
        // STY
        0x8C | 0x84 | 0x94 => cpu.y            

        _ => {},
    }

    cpu.pc = match opcode {
        // BCC
        0x90 => if !cpu.p_c {cpu.pc.wrapping_add(arg1 as i16)},
        // BCS
        0xB0 => if cpu.p_c {cpu.pc.wrapping_add(arg1 as i16)}, 
        // BEQ
        0xF0 => if cpu.z {cpu.pc.wrapping_add(arg1 as i16)}, 
        // BMI
        0x30 => if (!cpu.z && cpu.n) {cpu.pc.wrapping_add(arg1 as i16)},
        // BNE
        0xD0 => if !cpu.z {cpu.pc.wrapping_add(arg1 as i16)}, 
        // BPL
        0x10 => if (!cpu.z && !cpu.n) {cpu.pc.wrapping_add(arg1 as i16)},
        // BVC
        0x50 => if !cpu.v {cpu.pc.wrapping_add(arg1 as i16)},
        // BVS
        0x70 => if cpu.v {cpu.pc.wrapping_add(arg1 as i16)},
        // BRK
        0x00 => {}
    }

    match opcode {
        // BIT
        0x2C | 0x24 => {},
        // CLC 
        0x18 => {}
        // CLI 
        0x58 => {}
        // CLV 
        0xB8 => {}
        // SEC
        0x38 => {}
        // SEI
        0x78 => {}
        // PLP
        0x28 => {}
        // CPX
        0xEC | 0xE0 | 0xE4 => {}
        // CPY
        0xCC | 0xC0 | 0xC4 => {}
        // CMP
        0xCD | 0xDD | 0xD9 | 0xC9 | 0xC1 | 0xD1 | 0xC5 | 0xD5 => {
            let (sum, borrow) = cpu.a.borrowing_sub(instr_val, !cpu.p_c));
            cpu.p_v = was_overflow(cpu.a, instr_val, sum);
            cpu.p_c = !borrow;
            update_p_n_z(sum, cpu);
        }
    }
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
        exec_instr(opcode, arg1, arg2, &mut cpu, &mut wram, cart);
    }



