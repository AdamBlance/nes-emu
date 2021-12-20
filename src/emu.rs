use crate::hw;
use crate::opc;
use std::io;

pub fn emulate(cart: &hw::Cartridge) {
    
    // Create and init hardware

    let mut wram: [u8; 2048] = [0; 2048];

    let test = concat_u8(read_mem(0xFFFD, &wram, cart), read_mem(0xFFFC, &wram, cart));

    let mut cpu = hw::Cpu {
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
        pc: test,
        cycles: 0, 
    };

    loop {
        let (opcode, byte2, byte3) = read_three_bytes(cpu.pc, &wram, cart);
        println!("{:04X?}:  {:02X?}, {:02X?}, {:02X?}", cpu.pc-0x8000, opcode, byte2, byte3);
        exec_instr(opcode, byte2, byte3, &mut cpu, &mut wram, cart);
        let mut buffer = String::new();
        let mut stdin = io::stdin(); // We get `Stdin` here.
        stdin.read_line(&mut buffer).expect("it broke");
    }
}

fn concat_u8(msb: u8, lsb: u8) -> u16 {
    ((msb as u16) << 8) + (lsb as u16)
}

fn is_neg(val: u8) -> bool {
    val > 0x7F
}

fn get_bit(byte: u8, idx: u8) -> bool {
    (byte & (0x01 << idx)) != 0
}

fn was_overflow(original: u8, operand: u8, result: u8) -> bool {
    ((!(original ^ operand) & (original ^ result)) >> 7) == 1
}

fn p_to_byte(cpu: &hw::Cpu) -> u8 {
    (if cpu.p_n {0b1000_0000} else {0}) & 
    (if cpu.p_v {0b0100_0000} else {0}) & 
             0b0011_1000            &
    (if cpu.p_i {0b0000_0100} else {0}) & 
    (if cpu.p_z {0b0100_0010} else {0}) & 
    (if cpu.p_c {0b0100_0001} else {0})
}

fn read_three_bytes(addr: u16, wram: &[u8], cart: &hw::Cartridge) -> (u8, u8, u8) {
    (
        read_mem(addr, wram, cart),
        read_mem(addr.saturating_add(1), wram, cart),
        read_mem(addr.saturating_add(2), wram, cart)
    )
}

fn read_mem(addr: u16, wram: &[u8], cart: &hw::Cartridge) -> u8 {
    match addr {
        0x0000..=0x1FFF => wram[(addr % 0x800) as usize],
        0x2000..=0x3FFF => 0,
        0x4000..=0x401F => 0,
        0x4020..=0x5FFF => 0,
        0x6000..=0x7FFF => 0,
        0x8000..=0xBFFF => {
            let prg_addr = addr - 0x8000;
            // println!("PRG addr: {:04X?}", prg_addr);
            cart.prg_rom[prg_addr as usize]
        },
        0xC000..=0xFFFF => {
            let prg_addr = addr - 0x8000;
            // println!("PRG addr: {:04X?}", prg_addr);
            cart.prg_rom[prg_addr as usize]
        }
    }
}

fn write_mem(addr: u16, val: u8, wram: &mut [u8]) {
    if addr < 0x2000 {
        wram[(addr % 0x800) as usize] = val;
    }
}

fn exec_instr(opcode: u8, byte2: u8, byte3: u8, cpu: &mut hw::Cpu, wram: &mut [u8], cart: &hw::Cartridge) {

    // Tells us the addressing mode and number of cycles
    let instr_info = opc::INSTRUCTION_INFO[opcode as usize];

    println!("Mode - {:?}", instr_info.mode);
    println!("N={},V={},I={},Z={},C={}", cpu.p_n, cpu.p_v, cpu.p_i, cpu.p_z, cpu.p_c);
    let instr_addr = match instr_info.mode {
        opc::Mode::Absolute  => concat_u8(byte3, byte2),
        opc::Mode::AbsoluteX => concat_u8(byte3, byte2).wrapping_add(cpu.x as u16),
        opc::Mode::AbsoluteY => concat_u8(byte3, byte2).wrapping_add(cpu.y as u16),
        opc::Mode::ZeroPage  => byte2 as u16,
        opc::Mode::ZeroPageX => byte2.wrapping_add(cpu.x) as u16,
        opc::Mode::ZeroPageY => byte2.wrapping_add(cpu.y) as u16,
        opc::Mode::Relative  => cpu.pc.wrapping_add_signed((byte2 as i8) as i16),
        opc::Mode::IndirectX => {
            let zp_addr_lsb = byte2.wrapping_add(cpu.x);
            let zp_addr_msb = zp_addr_lsb.wrapping_add(1);
            concat_u8(wram[zp_addr_msb as usize], wram[zp_addr_lsb as usize])
        },
        opc::Mode::IndirectY => {
            let zp_lsb = wram[byte2 as usize];
            let zp_msb = wram[zp_lsb.wrapping_add(1) as usize];
            concat_u8(zp_msb, zp_lsb).wrapping_add(cpu.y as u16)
        },
        opc::Mode::AbsoluteI => {
            let ind_addr = concat_u8(byte3, byte2);
            let lsb = read_mem(ind_addr, wram, cart);
            let msb = read_mem(ind_addr.wrapping_add(1), wram, cart);
            concat_u8(msb, lsb)
        },
        _ => 0,
    };

    let instr_val = match instr_info.mode {
        opc::Mode::Immediate | opc::Mode::Accumulator => byte2,
        _ => read_mem(instr_addr, wram, cart),
    };

    let instr_len = match instr_info.mode {
        opc::Mode::Immediate => 2,
        opc::Mode::Accumulator => 1,
        opc::Mode::Implied => 1,
        opc::Mode::Absolute  => 3,
        opc::Mode::AbsoluteX => 3,
        opc::Mode::AbsoluteY => 3,
        opc::Mode::ZeroPage  => 2,
        opc::Mode::ZeroPageX => 2,
        opc::Mode::ZeroPageY => 2,
        opc::Mode::Relative  => 2,
        opc::Mode::IndirectX => 2,
        opc::Mode::IndirectY => 2,
        opc::Mode::AbsoluteI => 3,
    };

    cpu.cycles += instr_info.cycles as u64;

    match opcode {
        // LDA
        0xAD | 0xBD | 0xA9 | 0xB9 | 0xA1 | 0xB1 | 0xA5 | 0xB5 => {
            cpu.a = instr_val;
            cpu.p_n = is_neg(cpu.a);
            cpu.p_z = cpu.a == 0;
        },
        // LDX
        0xAE | 0xBE | 0xA2 | 0xA6 | 0xB6 => {
            cpu.x = instr_val;
            cpu.p_n = is_neg(cpu.x);
            cpu.p_z = cpu.x == 0;
        },
        // LDY
        0xAC | 0xBC | 0xA0 | 0xA4 | 0xB4 => {
            cpu.y = instr_val;
            cpu.p_n = is_neg(cpu.y);
            cpu.p_z = cpu.y == 0;
        },
        // STA
        0x8D | 0x9D | 0x99 | 0x81 | 0x91 | 0x85 | 0x95 => {
            write_mem(instr_addr, cpu.a, wram);
        },        
        // STX
        0x8E | 0x86 | 0x96 => {
            write_mem(instr_addr, cpu.x, wram);
        },        
        // STY
        0x8C | 0x84 | 0x94 => {
            write_mem(instr_addr, cpu.y, wram);
        },
        // TAX
        0xAA => {
            cpu.x = cpu.a;
            cpu.p_n = is_neg(cpu.x);
            cpu.p_z = cpu.x == 0;
        },
        // TAY
        0xA8 => {
            cpu.y = cpu.a;
            cpu.p_n = is_neg(cpu.y);
            cpu.p_z = cpu.y == 0;
        },
        // TSX
        0xBA => {
            cpu.x = cpu.s;
            cpu.p_n = is_neg(cpu.x);
            cpu.p_z = cpu.x == 0;
        },
        // TXA
        0x8A => {
            cpu.a = cpu.x;
            cpu.p_n = is_neg(cpu.a);
            cpu.p_z = cpu.a == 0;
        },
        // TXS
        0x9A => {
            cpu.s = cpu.x;
        },
        // TYA
        0x98 => {
            cpu.a = cpu.y;
            cpu.p_n = is_neg(cpu.a);
            cpu.p_z = cpu.a == 0;
        },
        // PHA
        0x48 => {
            wram[0x0100 + (cpu.s as usize)] = cpu.a;
            cpu.s = cpu.s.wrapping_sub(1);
        },
        // PHP
        0x08 => {
            wram[0x0100 + (cpu.s as usize)] = p_to_byte(&cpu);
            cpu.s = cpu.s.wrapping_sub(1);
        },
        // PLA
        0x68 => {
            cpu.a = wram[0x0100 + (cpu.s as usize)];
            cpu.s = cpu.s.wrapping_add(1);
            cpu.p_n = is_neg(cpu.a);
            cpu.p_z = cpu.a == 0;
        },
        // PLP
        0x28 => {
            let p_reg = wram[0x0100 + (cpu.s as usize)];
            cpu.p_n = get_bit(p_reg, 7);
            cpu.p_v = get_bit(p_reg, 6);
            cpu.p_i = get_bit(p_reg, 2);
            cpu.p_z = get_bit(p_reg, 1);
            cpu.p_c = get_bit(p_reg, 0);
            cpu.s = cpu.s.wrapping_add(1);
        },
        // ASL (ACC)
        0x0A => {
            cpu.p_c = cpu.a > 127;
            cpu.a <<= 1;
            cpu.p_n = is_neg(cpu.a);
            cpu.p_z = cpu.a == 0;
        },
        // ASL (RMW)
        0x0E | 0x1E | 0x06 | 0x16 => {
            cpu.p_c = wram[instr_addr as usize] > 127;
            wram[instr_addr as usize] <<= 1;
            cpu.p_n = is_neg(wram[instr_addr as usize]);
            cpu.p_z = wram[instr_addr as usize] == 0;
        },
        // LSR (ACC)
        0x4A => {
            cpu.p_c = (cpu.a & 0x01) == 1;
            cpu.a >>= 1;
            cpu.p_n = false;
            cpu.p_z = cpu.a == 0;
        },
        // LSR (RMW)
        0x4E | 0x5E | 0x46 | 0x56 => {
            cpu.p_c = (wram[instr_addr as usize] & 0x01) == 1;
            wram[instr_addr as usize] >>= 1;
            cpu.p_n = false;
            cpu.p_z = wram[instr_addr as usize] == 0;
        },
        // ROL (ACC)
        0x2A => {
            let initial_carry = cpu.p_c;
            cpu.p_c = cpu.a > 127;
            cpu.a <<= 1;
            cpu.a &= initial_carry as u8;
            cpu.p_n = is_neg(cpu.a);
            cpu.p_z = cpu.a == 0;
        },
        // ROL (RMW)
        0x2E | 0x3E | 0x26 | 0x36 => {
            let initial_carry = cpu.p_c;
            cpu.p_c = wram[instr_addr as usize] > 127;
            wram[instr_addr as usize] <<= 1;
            wram[instr_addr as usize] &= initial_carry as u8;
            cpu.p_n = is_neg(wram[instr_addr as usize]);
            cpu.p_z = wram[instr_addr as usize] == 0;
        },
        // ROR (ACC)
        0x6A => {
            let initial_carry = cpu.p_c;
            cpu.p_c = (cpu.a & 0x01) == 1;
            cpu.a >>= 1;
            cpu.a &= (initial_carry as u8) << 7;
            cpu.p_n = is_neg(cpu.a);
            cpu.p_z = cpu.a == 0;
        },
        // ROR (RMW)
        0x6E | 0x7E | 0x66 | 0x76 => {
            let initial_carry = cpu.p_c;
            cpu.p_c = (wram[instr_addr as usize] & 0x01) == 1;
            wram[instr_addr as usize] >>= 1;
            wram[instr_addr as usize] &= (initial_carry as u8) << 7;
            cpu.p_n = is_neg(wram[instr_addr as usize]);
            cpu.p_z = wram[instr_addr as usize] == 0;
        },
        // AND
        0x2D | 0x3D | 0x39 | 0x29 | 0x21 | 0x31 | 0x25 | 0x35 =>  {
            cpu.a &= instr_val;
            cpu.p_n = is_neg(cpu.a);
            cpu.p_z = cpu.a == 0;
        },
        // BIT
        0x2C | 0x24 => {
            cpu.p_n = get_bit(instr_val, 7);
            cpu.p_v = get_bit(instr_val, 6);
            cpu.p_z = (cpu.a & instr_val) != 0;
        },
        // EOR
        0x4D | 0x5D | 0x59 | 0x49 | 0x41 | 0x51 | 0x45 | 0x55 => { 
            cpu.a ^= instr_val;
            cpu.p_n = is_neg(cpu.a);
            cpu.p_z = cpu.a == 0;
        },
        // ORA
        0x0D | 0x1D | 0x19 | 0x09 | 0x01 | 0x11 | 0x05 | 0x15 => {
            cpu.a |= instr_val;
            cpu.p_n = is_neg(cpu.a);
            cpu.p_z = cpu.a == 0;
        },
        // ADC
        0x6D | 0x7D | 0x79 | 0x69 | 0x61 | 0x71 | 0x65 | 0x75 => {
            let (result, carry) = cpu.a.carrying_add(instr_val, cpu.p_c);
            cpu.p_v = was_overflow(cpu.a, instr_val, result);
            cpu.a = result;
            cpu.p_n = is_neg(cpu.a);
            cpu.p_z = cpu.a == 0;
        },
        // CMP
        0xCD | 0xDD | 0xD9 | 0xC9 | 0xC1 | 0xD1 | 0xC5 | 0xD5 => {
            let (result, borrow) = cpu.a.borrowing_sub(instr_val, !cpu.p_c);
            cpu.p_n = is_neg(result);
            cpu.p_z = result == 0;
            cpu.p_c = !borrow;
        },
        // CPX
        0xEC | 0xE0 | 0xE4 => {
            let (result, borrow) = cpu.a.borrowing_sub(cpu.x, !cpu.p_c);
            cpu.p_n = is_neg(result);
            cpu.p_z = result == 0;
            cpu.p_c = !borrow;
        },
        // CPY
        0xCC | 0xC0 | 0xC4 => {
            let (result, borrow) = cpu.a.borrowing_sub(cpu.y, !cpu.p_c);
            cpu.p_n = is_neg(result);
            cpu.p_z = result == 0;
            cpu.p_c = !borrow;
        },
        // SBC
        0xED | 0xFD | 0xF9 | 0xE9 | 0xE1 | 0xF1 | 0xE5 | 0xF5 => {
            let (result, borrow) = cpu.a.borrowing_sub(instr_val, !cpu.p_c);
            cpu.p_v = was_overflow(cpu.a, instr_val, result);
            cpu.a = result;
            cpu.p_n = is_neg(cpu.a);
            cpu.p_z = cpu.a == 0;
            cpu.p_c = !borrow;
        },
        // DEC
        0xCE | 0xDE | 0xC6 | 0xD6 => {
            wram[instr_addr as usize] -= 1;
            cpu.p_n = is_neg(wram[instr_addr as usize]);
            cpu.p_z = wram[instr_addr as usize] == 0;
        },
        // DEX
        0xCA => {
            cpu.x -= 1;
            cpu.p_n = is_neg(cpu.x);
            cpu.p_z = cpu.x == 0;
        },
        // DEY
        0x88 => {
            cpu.y -= 1;
            cpu.p_n = is_neg(cpu.y);
            cpu.p_z = cpu.y == 0;
        },
        // INC
        0xEE | 0xFE | 0xE6 | 0xF6 => {
            wram[instr_addr as usize] += 1;
            cpu.p_n = is_neg(wram[instr_addr as usize]);
            cpu.p_z = wram[instr_addr as usize] == 0;
        },
        // INX
        0xE8 => {
            cpu.x += 1;
            cpu.p_n = is_neg(cpu.x);
            cpu.p_z = cpu.x == 0;
        },
        // INY
        0xC8 => {
            cpu.y += 1;
            cpu.p_n = is_neg(cpu.y);
            cpu.p_z = cpu.y == 0;
        },
        // BRK
        0x00 => {
            wram[0x0100 + (cpu.s as usize)] = (cpu.pc >> 8) as u8;
            wram[0x0100 + (cpu.s as usize) - 1] = (cpu.pc & 0x00FF) as u8;
            wram[0x0100 + (cpu.s as usize) - 2] = p_to_byte(&cpu);
            cpu.s = cpu.s.wrapping_sub(3);
            cpu.pc = concat_u8(read_mem(0xFFFF, wram, cart), read_mem(0xFFFE, wram, cart));
        },
        // JMP 
        0x4C | 0x6C => {
            cpu.pc = instr_addr;
        },
        // JSR
        0x20 => {
            wram[0x0100 + (cpu.s as usize)] = (cpu.pc >> 8) as u8;
            wram[0x0100 + (cpu.s as usize) - 1] = (cpu.pc & 0x00FF) as u8;
            cpu.pc = instr_addr.wrapping_add(1);
        },
        // RTI
        0x40 => {
            let p_reg = wram[0x0100 + (cpu.s as usize)];
            cpu.p_n = get_bit(p_reg, 7);
            cpu.p_v = get_bit(p_reg, 6);
            cpu.p_i = get_bit(p_reg, 2);
            cpu.p_z = get_bit(p_reg, 1);
            cpu.p_c = get_bit(p_reg, 0);
            let lsb = wram[0x0100 + (cpu.s as usize)];
            let msb = wram[0x0100 + (cpu.s as usize) + 1];
            cpu.s = cpu.s.wrapping_add(3);
            cpu.pc = concat_u8(msb, lsb);
        },
        // RTS 
        0x60 => {
            let lsb = wram[0x0100 + (cpu.s as usize)];
            let msb = wram[0x0100 + (cpu.s as usize) + 1];
            cpu.s = cpu.s.wrapping_add(2);
            cpu.pc = concat_u8(msb, lsb) + 1;
        },
        // BCC
        0x90 => {
            if !cpu.p_c {cpu.pc = instr_addr;}
        },
        // BCS
        0xB0 => {
            if cpu.p_c {cpu.pc = instr_addr;}
        },
        // BEQ
        0xF0 => {
            if cpu.p_z {cpu.pc = instr_addr;}
        },
        // BMI
        0x30 => {
            if !cpu.p_z && cpu.p_n {cpu.pc = instr_addr;}
        },
        // BNE
        0xD0 => {
            if !cpu.p_z {cpu.pc = instr_addr;}
        },
        // BPL
        0x10 => {
            if !cpu.p_z && !cpu.p_n {cpu.pc = instr_addr;}
        },
        // BVC
        0x50 => {
            if !cpu.p_v {cpu.pc = instr_addr;}
        },
        // BVS
        0x70 => {
            if cpu.p_v {cpu.pc = instr_addr;}
        }
        // CLC 
        0x18 => {
            cpu.p_c = false;
        }
        // CLD
        0xD8 => {
            println!("Decimal mode not implemented on the 2A03/7");
        }
        // CLI 
        0x58 => {
            cpu.p_i = false;
        }
        // CLV 
        0xB8 => {
            cpu.p_v = false;
        }
        // SEC
        0x38 => {
            cpu.p_c = true;
        }
        // SED
        0xF8 => {
            println!("Decimal mode not implemented on the 2A03/7");
        }
        // SEI
        0x78 => {
            cpu.p_i = true;
        }
        // NOP
        0xEA => {}
        _ => println!("Illegal opcodes are not implemented")
        
    }
    cpu.pc = cpu.pc.wrapping_add(instr_len);

}
