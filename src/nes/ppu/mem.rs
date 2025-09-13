use crate::nes::cartridge::Mirroring;
use crate::nes::{ppu, Nes};
use crate::nes::mem_consts::*;



pub fn memory_mapped_register_read(addr: u16, nes: &mut Nes) -> u8 {
    match 0x2000 + (addr % 8) {
        PPUCTRL_2000 | PPUMASK_2001 | OAMADDR_2003 | PPUSCROLL_2005 | PPUADDR_2006 => get_dynamic_latch(nes),
        PPUSTATUS_2002 => {
            // Upper 3 bits of PPUSTATUS are open bus
            let status = nes.ppu.get_ppustatus_byte() | (get_dynamic_latch(nes) & 0b0001_1111);
            nes.ppu.in_vblank = false;
            nes.ppu.w = false;
            set_dynamic_latch(status, nes);
            status
        }
        OAMDATA_2004 => {
            set_dynamic_latch(nes.ppu.oam_addr, nes);
            nes.ppu.oam_addr
        }
        PPUDATA_2007 if nes.ppu.addr_bus < 0x3F00=> {
            let existing_data_in_read_buffer = nes.ppu.ppudata_buffer;
            nes.ppu.ppudata_buffer = read_vram(nes.ppu.v, nes);
            increment_v_after_ppudata_access(nes);
            set_dynamic_latch(existing_data_in_read_buffer, nes);
            existing_data_in_read_buffer
        }
        PPUDATA_2007 if nes.ppu.addr_bus >= 0x3F00 => {
            let data_in_memory = read_vram(nes.ppu.v, nes);
            // Some weird behaviour when the PPU reads from palette memory
            nes.ppu.ppudata_buffer = read_vram(nes.ppu.v.wrapping_sub(0x1000), nes);
            increment_v_after_ppudata_access(nes);
            set_dynamic_latch(data_in_memory, nes);
            data_in_memory
        }
        _ => unreachable!(),
    }
}

pub fn memory_mapped_register_write(addr: u16, val: u8, nes: &mut Nes) {
    const PPU_WARMUP: u64 = 29658;
    if matches!(addr, PPUCTRL_2000 | PPUMASK_2001 | PPUSCROLL_2005 | PPUADDR_2006) && nes.cpu.cycles < PPU_WARMUP {
        return
    }
    set_dynamic_latch(val, nes);
    match 0x2000 + (addr % 8) {
        PPUCTRL_2000 => {
            nes.ppu.set_ppuctrl_from_byte(val);
            nes.ppu.t &= !ppu::NAMETABLE;
            nes.ppu.t |= (val as u16 & 0b11) << 10;
        }
        PPUMASK_2001 =>
            nes.ppu.set_ppumask_from_byte(val),
        OAMADDR_2003 =>
            nes.ppu.oam_addr = val,
        OAMDATA_2004 => {
            nes.ppu.oam[nes.ppu.oam_addr as usize] = val;
            nes.ppu.oam_addr = nes.ppu.oam_addr.wrapping_add(1);
        }
        PPUSCROLL_2005 if !nes.ppu.w => {
            // Put x-scroll into t and x after first write
            nes.ppu.t &= !ppu::COARSE_X;
            nes.ppu.t |= val as u16 >> 3;
            nes.ppu.x = val & 0b111;
            nes.ppu.w = !nes.ppu.w;
        }
        PPUSCROLL_2005 if nes.ppu.w => {
            // Put y-scroll into, t after second write
            nes.ppu.t &= !(ppu::COARSE_Y | ppu::FINE_Y);
            nes.ppu.t |= (val as u16 & 0b11111_000) << 2;
            nes.ppu.t |= (val as u16 & 0b00000_111) << 12;
            nes.ppu.w = !nes.ppu.w;
        }
        PPUADDR_2006 if !nes.ppu.w => {
            // Write into upper 6 bits of t
            nes.ppu.t &= 0b000000_11111111;
            nes.ppu.t |= (val as u16 & 0b111111) << 8;
            nes.ppu.w = true;
        }
        PPUADDR_2006 if nes.ppu.w => {
            // Write into lower 8 bits of t
            nes.ppu.t &= 0b111111_00000000;
            nes.ppu.t |= val as u16;
            // Copy t into v
            nes.ppu.v = nes.ppu.t;
            nes.ppu.addr_bus = nes.ppu.v;
            nes.ppu.w = false;
        }
        PPUDATA_2007 => {
            write_vram(nes.ppu.v, val, nes);
            increment_v_after_ppudata_access(nes);
        }
        _ => (),
    }
}

pub fn read_vram(addr: u16, nes: &mut Nes) -> u8 {
    // Colour palette reads don't put anything on the PPU address bus
    if addr < PALETTE_RAM_START_3F00 {
        nes.ppu.addr_bus = addr;
    }
    match addr {
        0x0000..=PATTERN_TABLE_END_1FFF => nes.cart.read_chr(addr),
        VRAM_START_2000..=VRAM_END_3EFF => {
            let mapped_vram_addr = mirroring_mapping(addr, nes.cart.mirroring());
            nes.ppu.vram[mapped_vram_addr as usize]
        },
        PALETTE_RAM_START_3F00..=PALETTE_RAM_END_3FFF => {
            let colour = nes.ppu.palette_mem[map_vram_addr_to_palette_addr(addr)];
            if nes.ppu.greyscale {
                colour & 0b0011_0000
            } else {
                colour
            }
        }
        x => panic!("Invalid PPU address {x:016b}"),
    }
}

pub fn write_vram(addr: u16, val: u8, nes: &mut Nes) {
    if addr < PALETTE_RAM_START_3F00 {
        nes.ppu.addr_bus = addr;
    }
    match addr {
        ..=PATTERN_TABLE_END_1FFF => nes.cart.write_chr(addr, val),
        VRAM_START_2000..=VRAM_END_3EFF => {
            let mapped_vram_addr = mirroring_mapping(addr, nes.cart.mirroring());
            nes.ppu.vram[mapped_vram_addr as usize] = val
        }
        PALETTE_RAM_START_3F00..=PALETTE_RAM_END_3FFF =>
            nes.ppu.palette_mem[map_vram_addr_to_palette_addr(addr)] = val,
        x => panic!("Invalid PPU address {x:016b}")
    }
}

pub fn increment_v_after_ppudata_access(nes: &mut Nes) {
    let increment = if !nes.ppu.increment_select { 1 } else { 32 };
    nes.ppu.v = nes.ppu.v.wrapping_add(increment);
    nes.ppu.addr_bus = nes.ppu.v;
}

fn map_vram_addr_to_palette_addr(addr: u16) -> usize {
    let offset = ((addr - 0x3F00) % 0x20) as usize;
    // Shared first entry between sprites and background
    if offset > 0xF && offset % 4 == 0 { offset - 0x10 } else { offset }
}

pub fn mirroring_mapping(addr: u16, mirroring: Mirroring) -> u16 {
    // The physical nametables sit at 0x2000..=0x23FF and 0x2400..=0x27FF
    let truncated = addr & 0b0000_1111_1111_1111;
    match mirroring {
        Mirroring::Vertical => truncated % 0x800,
        Mirroring::Horizontal => (truncated / 0x800) * 0x400 + (truncated % 0x400),
        Mirroring::SingleScreenLower => truncated % 0x400,
        Mirroring::SingleScreenUpper => 0x400 + (truncated % 0x400),
    }
}

pub fn set_dynamic_latch(val: u8, nes: &mut Nes) {
    nes.ppu.dynamic_latch = val;
    nes.ppu.dynamic_latch_last_set_cycle = nes.ppu.cycles;
}

const PPU_DYNAMIC_LATCH_DECAY_TIME: u64 = 500000;
pub fn get_dynamic_latch(nes: &mut Nes) -> u8 {
    if nes.ppu.cycles - nes.ppu.dynamic_latch_last_set_cycle > PPU_DYNAMIC_LATCH_DECAY_TIME {
        nes.ppu.dynamic_latch = 0;
    }
    nes.ppu.dynamic_latch
}
