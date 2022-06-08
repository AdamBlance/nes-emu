use crate::hw::Nes;
use crate::instr_defs::Mode::*;
use crate::instr_defs::Name::*;
use crate::util::concat_u8;

pub fn log(nes: &Nes) -> String {

    let instr_len = match nes.cpu.instruction.mode {
        Accumulator => 1,
        Implied     => 1,
        Immediate   => 2,
        Absolute    => 3,
        AbsoluteX   => 3,
        AbsoluteY   => 3,
        ZeroPage    => 2,
        ZeroPageX   => 2,
        ZeroPageY   => 2,
        Relative    => 2,
        IndirectX   => 2,
        IndirectY   => 2,
        AbsoluteI   => 3,
    };

    let mut bytes_str = String::new();

    let opcode_str = format!("{:02X} ", nes.cpu.trace_opcode);
    bytes_str.push_str(&opcode_str);

    if instr_len >= 2 {
        let byte2_str = format!("{:02X} ", nes.cpu.trace_byte2);
        bytes_str.push_str(&byte2_str);
    }
    if instr_len == 3 {
        let byte3_str = format!("{:02X}", nes.cpu.trace_byte3);
        bytes_str.push_str(&byte3_str);
    }

    let mut instr_str = format!("{:?} ", nes.cpu.instruction.name);

    let addressing_str = match nes.cpu.instruction.mode {
        Implied => String::new(),
        Accumulator => format!(
            "A"
        ),
        Immediate => format!(
            "#${:02X}", 
            nes.cpu.trace_imm
        ),
        ZeroPage => format!(
            "${:02X} = {:02X}", 
            nes.cpu.trace_byte2, 
            nes.cpu.trace_stored_val
        ),
        ZeroPageX => format!(
            "${:02X},X @ {:02X} = {:02X}", 
            nes.cpu.trace_byte2, 
            nes.cpu.get_address(), 
            nes.cpu.trace_stored_val
        ),
        ZeroPageY => format!(
            "${:02X},Y @ {:02X} = {:02X}", 
            nes.cpu.trace_byte2, 
            nes.cpu.get_address(), 
            nes.cpu.trace_stored_val
        ),
        Absolute => {
            if nes.cpu.instruction.name != JMP && nes.cpu.instruction.name != JSR {
                format!(
                    "${:04X?} = {:02X}", 
                    nes.cpu.get_address(), 
                    nes.cpu.trace_stored_val
                )
            } else {
                format!(
                    "${:04X?}", 
                    nes.cpu.get_address(), 
                )
            }
        }

        AbsoluteX => format!(
            "${:04X?},X @ {:04X} = {:02X}", 
            concat_u8(nes.cpu.trace_byte3, nes.cpu.trace_byte2), 
            nes.cpu.get_address(), 
            nes.cpu.trace_stored_val
        ),
        AbsoluteY => format!(
            "${:04X?},Y @ {:04X} = {:02X}", 
            concat_u8(nes.cpu.trace_byte3, nes.cpu.trace_byte2), 
            nes.cpu.get_address(), 
            nes.cpu.trace_stored_val
        ),
        IndirectX => format!(
            "(${:02X},X) @ {:02X} = {:04X} = {:02X}", 
            nes.cpu.trace_byte2, 
            nes.cpu.trace_byte2.wrapping_add(nes.cpu.x), 
            nes.cpu.get_address(), 
            nes.cpu.trace_stored_val
        ),
        IndirectY => format!(
            "(${:02X}),Y = {:04X} @ {:04X} = {:02X}", 
            nes.cpu.trace_byte2, 
            nes.cpu.get_address().wrapping_sub(nes.cpu.y as u16),
            nes.cpu.get_address(), 
            nes.cpu.trace_stored_val
        ),
        AbsoluteI => format!(
            "(${:04X}) = {:04X}",
            concat_u8(nes.cpu.upper_pointer, nes.cpu.lower_pointer.wrapping_sub(1)),
            nes.cpu.pc,
        ),
        Relative => format!(
            "${:04X}",
            nes.old_cpu_state.pc.wrapping_add_signed(2 + nes.cpu.branch_offset as i8 as i16),
        ),
    };

    instr_str.push_str(&addressing_str);
    

    let register_str = format!(
        "A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X} PPU:{:>3},{:>3} CYC:{}",
        nes.old_cpu_state.a,
        nes.old_cpu_state.x,
        nes.old_cpu_state.y,
        nes.old_cpu_state.get_p(),
        nes.old_cpu_state.s,
        nes.old_ppu_state.scanline,
        nes.old_ppu_state.scanline_cycle,
        nes.old_cpu_state.cycles,
    );
    
    let log_str = format!(
        "{:04X}  {:10}{:32}{}",
        nes.old_cpu_state.pc,
        &bytes_str,
        &instr_str,
        &register_str,
    );

    log_str
}

pub fn print_ppu_log(nes: &Nes) {
    println!("Values at beginning of PPU step:");

    println!("rendering: {}", nes.ppu.show_bg || nes.ppu.show_sprites);
    println!("scanline: {}, cycle: {}", nes.ppu.scanline, nes.ppu.scanline_cycle);
    println!("v: {:016b} ({:04X})", nes.ppu.v, nes.ppu.v);
    println!("t: {:016b} ({:04X}), x: {:08b}", nes.ppu.t, nes.ppu.t, nes.ppu.x);

    println!("pt_lsb_sr: {:016b}", nes.ppu.bg_ptable_lsb_sr);
    println!("pt_msb_sr: {:016b}", nes.ppu.bg_ptable_msb_sr);

    println!("at_lsb_sr: {:08b}", nes.ppu.bg_attr_lsb_sr);
    println!("at_msb_sr: {:08b}", nes.ppu.bg_attr_msb_sr);

    println!("bg_ntable_tmp: {:08b} ({:02X})", nes.ppu.bg_ntable_tmp, nes.ppu.bg_ntable_tmp);
    println!("bg_atable_tmp: {:08b} ({:02X})", nes.ppu.bg_atable_tmp, nes.ppu.bg_atable_tmp);
    println!("bg_ptable_lsb_tmp: {:08b} ({:02X})", nes.ppu.bg_ptable_lsb_tmp, nes.ppu.bg_ptable_lsb_tmp);
    println!("bg_ptable_msb_tmp: {:08b} ({:02X})", nes.ppu.bg_ptable_msb_tmp, nes.ppu.bg_ptable_msb_tmp);

    println!("bg_attr_lsb_latch: {:?}, bg_attr_msb_latch: {:?}", nes.ppu.bg_attr_lsb_latch, nes.ppu.bg_attr_msb_latch);
    println!();
}
