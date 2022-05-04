use crate::hw::*;
use crate::util::*;
use std::{thread, time};

pub static PALETTE: [(u8,u8,u8); 56] = [(84, 84, 84), (0, 30, 116), (8, 16, 144), (48, 0, 136), (68, 0, 100), (92, 0, 48), (84, 4, 0), (60, 24, 0), (32, 42, 0), (8, 58, 0), (0, 64, 0), (0, 60, 0), (0, 50, 60), (0, 0, 0), (152, 150, 152), (8, 76, 196), (48, 50, 236), (92, 30, 228), (136, 20, 176), (160, 20, 100), (152, 34, 32), (120, 60, 0), (84, 90, 0), (40, 114, 0), (8, 124, 0), (0, 118, 40), (0, 102, 120), (0, 0, 0), (236, 238, 236), (76, 154, 236), (120, 124, 236), (176, 98, 236), (228, 84, 236), (236, 88, 180), (236, 106, 100), (212, 136, 32), (160, 170, 0), (116, 196, 0), (76, 208, 32), (56, 204, 108), (56, 180, 204), (60, 60, 60), (236, 238, 236), (168, 204, 236), (188, 188, 236), (212, 178, 236), (236, 174, 236), (236, 174, 212), (236, 180, 176), (228, 196, 144), (204, 210, 120), (180, 222, 120), (168, 226, 144), (152, 226, 180), (160, 214, 228), (160, 162, 160)];

// https://archive.nes.science/nesdev-forums/f3/t13966.xhtml
// https://forums.nesdev.org/viewtopic.php?t=8216
// https://www.nesdev.org/2C02%20technical%20reference.TXT

// This will need a complete overhaul, use the resources above 
// please don't get distracted on this when you've got less than a week until your first exam!

const COARSE_X_MASK: u16  = 0b0_000_00_00000_11111;
const COARSE_Y_MASK: u16  = 0b0_000_00_11111_00000;
const NAMETABLE_MASK: u16 = 0b0_000_11_00000_00000;
const FINE_Y_MASK: u16    = 0b0_111_00_00000_00000;


fn physical_nametable_addr(addr: u16, cart: &Cartridge) -> usize {
    let addr_u = (addr as usize) - 0x2000;
    if cart.v_mirroring {
        match addr {
            0x2000..=0x27FF => addr_u,
            0x2800..=0x2FFF => addr_u - 0x800,
            _ => 0,
        }
    } else {
        match addr {
            0x2000..=0x23FF => addr_u,
            0x2400..=0x27FF => addr_u - 0x400,
            0x2800..=0x2BFF => addr_u - 0x400,
            0x2C00..=0x2FFF => addr_u - 0x800,
            _ => 0,
        }
    }
}

fn increment_v_rendering(nes: &mut Nes) {
    let coarse_x  = nes.ppu.v & COARSE_X_MASK;
    let coarse_y  = (nes.ppu.v & COARSE_Y_MASK) >> 5;
    let nametable = (nes.ppu.v & NAMETABLE_MASK) >> 10;
    let fine_y    = (nes.ppu.v & FINE_Y_MASK) >> 12;

    if coarse_x < 31 {
        nes.ppu.v += 1;
    } else {
        nes.ppu.v &= !COARSE_X_MASK;
        nes.ppu.v ^= 0b0_000_01_00000_00000;
    }
    
    if fine_y < 7 {
        nes.ppu.v += 0b0_001_00_00000_00000;
    } else {
        nes.ppu.v &= !FINE_Y_MASK;
        if coarse_y < 29 {
            nes.ppu.v += 0b0_000_00_00001_00000;
            
        } else {
            nes.ppu.v &= !COARSE_Y_MASK;
            if coarse_y == 29 {
                nes.ppu.v ^= 0b0_000_10_00000_00000;
            }
        }
    }   
}

pub fn read_vram(addr: u16, nes: &mut Nes) -> u8 {
    let a = addr as usize;
    match addr {
        0x0000..=0x1FFF => nes.cart.chr_rom[a],
        0x2000..=0x2FFF => nes.ppu.vram[physical_nametable_addr(addr, &nes.cart)],
        0x3000..=0x3EFF => nes.ppu.vram[physical_nametable_addr(addr - 0x1000, &nes.cart)],
        0x3F00..=0x3FFF => nes.ppu.palette_mem[(a - 0x3F00) % 32],
        _ => 0,
    }
}

pub fn write_vram(addr: u16, val: u8, nes: &mut Nes) {
    let a = addr as usize;
    match addr {
        0x2000..=0x2FFF => nes.ppu.vram[physical_nametable_addr(addr, &nes.cart)] = val,
        0x3000..=0x3EFF => nes.ppu.vram[physical_nametable_addr(addr - 0x1000, &nes.cart)] = val,
        0x3F00..=0x3FFF => nes.ppu.palette_mem[(a - 0x3F00) % 32] = val,
        _ => (),
    }
}

pub fn step_ppu(nes: &mut Nes) {

    // Need to work through the wiki to get this implemented correctly
    // t -> v only at start of frame when set with scroll?
    // v is vram address set by PPUADDR, but also used and incremented when scrolling
    // need to get the reads and writes to ppu correct in here
    // I'm manually reading the chr_rom, vram etc. should be using read_vram

    if nes.ppu.pixel == 257 {
        let t_horizontal_mask = 0b0_000_01_00000_11111;
        let t_horizontal = nes.ppu.t & t_horizontal_mask;
        nes.ppu.v = (nes.ppu.v & !t_horizontal_mask) | t_horizontal;
    }

    if nes.ppu.scanline == 261 && (nes.ppu.pixel >= 280 || nes.ppu.scanline <= 304) {
        let t_vertical_mask = 0b0_111_10_11111_00000;
        let t_vertical = nes.ppu.t & t_vertical_mask;
        nes.ppu.v = (nes.ppu.v & !t_vertical_mask) | t_vertical;
    }


    if (nes.ppu.pixel >= 9 && nes.ppu.pixel <= 257) && (nes.ppu.pixel.wrapping_sub(1) % 8 == 0) {

        let nametable_addr = 0x2000 | (nes.ppu.v & 0x0FFF);

        let nametable_byte = read_vram(nametable_addr, nes);


        let base_addr = (nes.ppu.background_tile_select as u16) << 12;
        let fine_y = nes.ppu.v >> 12;
        let tile_lsb_addr = base_addr +  16 * (nametable_byte as u16) + fine_y;

        let tile_lsb = read_vram(tile_lsb_addr, nes);
        let tile_msb = read_vram(tile_lsb_addr + 8, nes);

        nes.ppu.lsb_pattern_shift_register |= ((tile_lsb as u16) << 8);
        nes.ppu.msb_pattern_shift_register |= ((tile_msb as u16) << 8);

    } 

    if nes.ppu.scanline == 261 && nes.ppu.pixel == 1 {
        nes.ppu.vblank = false;
    }

    if nes.ppu.scanline == 241 && nes.ppu.pixel == 1 {
        if nes.ppu.nmi_enable {
            nes.cpu.nmi_interrupt = true;
        }
        nes.ppu.vblank = true;
    }

    if !(nes.ppu.pixel > 256 && nes.ppu.pixel < 328) && nes.ppu.pixel != 0 && nes.ppu.pixel % 8 == 0 {
        increment_v_rendering(nes);
    }

    

    let pixel_lsb = get_bit(nes.ppu.lsb_pattern_shift_register as u8, nes.ppu.x) as u8;
    let pixel_msb = get_bit(nes.ppu.msb_pattern_shift_register as u8, nes.ppu.x) as u8;

    let pixel_colour = (pixel_msb << 1) | pixel_lsb;

    let attr_addr = 0x23C0 | (nes.ppu.v & 0x0C00) | ((nes.ppu.v >> 4) & 0x38) | ((nes.ppu.v >> 2) & 0x07);

    let attr_table_byte = read_vram(attr_addr, nes);


    let quadrant = ((nes.ppu.scanline / 16) % 2)*2 + ((nes.ppu.pixel / 16) % 2);

    let palette_index = (attr_table_byte >> (quadrant * 2)) & 0b0000_0011;

    let palette_colour = PALETTE[(nes.ppu.palette_mem[(palette_index + pixel_colour) as usize]%56) as usize];
    let frame_idx = ((nes.ppu.scanline*256 + nes.ppu.pixel)*4) as usize;

        let nametable_addr = 0x2000 | (nes.ppu.v & 0x0FFF);

        let nametable_byte = read_vram(nametable_addr, nes);


        let base_addr = (nes.ppu.background_tile_select as u16) << 12;
        let fine_y = nes.ppu.v >> 12;
        let tile_lsb_addr = base_addr +  16 * (nametable_byte as u16) + fine_y;

        let tile_lsb = read_vram(tile_lsb_addr, nes);
        let tile_msb = read_vram(tile_lsb_addr + 8, nes);


    if (nes.ppu.pixel < 256) && (nes.ppu.scanline < 240) {
        nes.frame[frame_idx] = (tile_lsb & 0b0000001) * 255;
        nes.frame[frame_idx+1] = (tile_lsb & 0b0000001) * 255;
        nes.frame[frame_idx+2] = (tile_lsb & 0b0000001) * 255;
        nes.frame[frame_idx+3] = 255;
    }

    nes.ppu.lsb_pattern_shift_register >>= 1;
    nes.ppu.msb_pattern_shift_register >>= 1;

    nes.ppu.cycles += 1;
    nes.ppu.pixel += 1;
    if nes.ppu.pixel == 341 {
        nes.ppu.pixel = 0;
        nes.ppu.scanline += 1;
    }
    if nes.ppu.scanline == 262 {
        nes.ppu.scanline = 0;
    }
}