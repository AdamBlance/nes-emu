use crate::hw::*;
use std::{thread, time};

pub static PALETTE: [(u8,u8,u8); 56] = [(84, 84, 84), (0, 30, 116), (8, 16, 144), (48, 0, 136), (68, 0, 100), (92, 0, 48), (84, 4, 0), (60, 24, 0), (32, 42, 0), (8, 58, 0), (0, 64, 0), (0, 60, 0), (0, 50, 60), (0, 0, 0), (152, 150, 152), (8, 76, 196), (48, 50, 236), (92, 30, 228), (136, 20, 176), (160, 20, 100), (152, 34, 32), (120, 60, 0), (84, 90, 0), (40, 114, 0), (8, 124, 0), (0, 118, 40), (0, 102, 120), (0, 0, 0), (236, 238, 236), (76, 154, 236), (120, 124, 236), (176, 98, 236), (228, 84, 236), (236, 88, 180), (236, 106, 100), (212, 136, 32), (160, 170, 0), (116, 196, 0), (76, 208, 32), (56, 204, 108), (56, 180, 204), (60, 60, 60), (236, 238, 236), (168, 204, 236), (188, 188, 236), (212, 178, 236), (236, 174, 236), (236, 174, 212), (236, 180, 176), (228, 196, 144), (204, 210, 120), (180, 222, 120), (168, 226, 144), (152, 226, 180), (160, 214, 228), (160, 162, 160)];

pub fn read_vram(addr: u16, nes: &mut Nes) -> u8 {
    let a = addr as usize;
    match addr {
        0x0000..=0x1FFF => nes.cart.chr_rom[a],
        0x2000..=0x23FF => nes.ppu.vram[a - 0x2000],
        0x2400..=0x27FF => nes.ppu.vram[a - 0x2000],
        0x2800..=0x2BFF => nes.ppu.vram[a - 0x2000],
        0x2C00..=0x2FFF => nes.ppu.vram[a - 0x2000],
        0x3000..=0x3EFF => nes.ppu.vram[a - 0x3000],
        0x3F00..=0x3F1F => nes.ppu.palette_mem[a - 0x3F00],
        0x3F20..=0x3FFF => nes.ppu.palette_mem[a - 0x3F20],
        _ => 0,
    }
}

pub fn write_vram(addr: u16, val: u8, nes: &mut Nes) {
    let a = addr as usize;
    match addr {
        0x2000..=0x23FF => nes.ppu.vram[a - 0x2000] = val,
        0x2400..=0x27FF => nes.ppu.vram[a - 0x2000] = val,
        0x2800..=0x2BFF => nes.ppu.vram[a - 0x2000] = val,
        0x2C00..=0x2FFF => nes.ppu.vram[a - 0x2000] = val,
        0x3000..=0x3EFF => nes.ppu.vram[a - 0x3000] = val,
        0x3F00..=0x3F1F => nes.ppu.palette_mem[a - 0x3F00] = val,
        0x3F20..=0x3FFF => nes.ppu.palette_mem[a - 0x3F20] = val,
        _ => (),
    }
}

pub fn step_ppu(nes: &mut Nes) {

    // Need to work through the wiki to get this implemented correctly
    // t -> v only at start of frame when set with scroll?
    // v is vram address set by PPUADDR, but also used and incremented when scrolling
    // need to get the reads and writes to ppu correct in here
    // I'm manually reading the chr_rom, vram etc. should be using read_vram

    if nes.ppu.scanline == 261 && nes.ppu.pixel == 1 {
        nes.ppu.vblank = false;
    }

    if nes.ppu.scanline == 241 && nes.ppu.pixel == 1 {
        nes.cpu.nmi_interrupt = true;
        nes.ppu.vblank = true;
    }

    if nes.ppu.pixel < 256 && nes.ppu.scanline < 240 {

        // Which nametable is selected? 
        let nametable_addr = (0x0400 * (nes.ppu.nametable_select as u32));
        let index_into_nametable = (nametable_addr + (nes.ppu.pixel / 8)) as usize;
        
        // println!("tile_select {}", index_into_nametable);
        let ten_millis = time::Duration::from_millis(10);
        thread::sleep(ten_millis);

        let index_into_pattern_table = nes.ppu.vram[index_into_nametable] as usize;
        
        let pattern_table_offset = if nes.ppu.background_tile_select {0x1000} else {0x0000};

        let lsb = (nes.cart.chr_rom[pattern_table_offset + index_into_pattern_table] >> (7 - (nes.ppu.pixel % 8))) & 0x01;
        let msb = (nes.cart.chr_rom[pattern_table_offset + index_into_pattern_table + 8] >> (7 - (nes.ppu.pixel % 8))) & 0x01;
        let palette_colour = (msb << 1) + lsb;

        let attribute_tile = nes.ppu.vram[(nametable_addr + 0x03C0 + ((nes.ppu.scanline*32)/8) + (nes.ppu.pixel / 32)) as usize];
        
        let quadrant = ((nes.ppu.scanline / 16) % 2)*2 + ((nes.ppu.pixel / 16) % 2);

        println!("quadrant {}", quadrant);


        let palette_index = (attribute_tile >> (quadrant * 2)) & 0b0000_0011;

        

        let final_colour = nes.ppu.palette_mem[(palette_index + palette_colour) as usize];

        let frame_idx = ((nes.ppu.scanline*256 + nes.ppu.pixel)*4) as usize;

        let pal_col = PALETTE[final_colour as usize];

        nes.frame[frame_idx] = pal_col.0;
        nes.frame[frame_idx+1] = pal_col.1;
        nes.frame[frame_idx+2] = pal_col.2;
        nes.frame[frame_idx+3] = 255;

    }

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