use crate::{hw::*, util::{get_bit, get_bit_u16}};

pub static PALETTE: [(u8,u8,u8); 64] = [
    ( 84,  84,  84), (  0,  30, 116), (  8,  16, 144), ( 48,   0, 136), ( 68,   0, 100), ( 92,   0,  48), ( 84,   4,   0), ( 60,  24,   0), ( 32,  42,   0), (  8,  58,   0), (  0,  64,   0), (  0,  60,   0), (  0,  50,  60), (  0,   0,   0), (  0,   0,   0), (  0,   0,   0), 
    (152, 150, 152), (  8,  76, 196), ( 48,  50, 236), ( 92,  30, 228), (136,  20, 176), (160,  20, 100), (152,  34,  32), (120,  60,   0), ( 84,  90,   0), ( 40, 114,   0), (  8, 124,   0), (  0, 118,  40), (  0, 102, 120), (  0,   0,   0), (  0,   0,   0), (  0,   0,   0), 
    (236, 238, 236), ( 76, 154, 236), (120, 124, 236), (176,  98, 236), (228,  84, 236), (236,  88, 180), (236, 106, 100), (212, 136,  32), (160, 170,   0), (116, 196,   0), ( 76, 208,  32), ( 56, 204, 108), ( 56, 180, 204), ( 60,  60,  60), (  0,   0,   0), (  0,   0,   0), 
    (236, 238, 236), (168, 204, 236), (188, 188, 236), (212, 178, 236), (236, 174, 236), (236, 174, 212), (236, 180, 176), (228, 196, 144), (204, 210, 120), (180, 222, 120), (168, 226, 144), (152, 226, 180), (160, 214, 228), (160, 162, 160), (  0,   0,   0), (  0,   0,   0)
];

const NAMETABLE: u16    = 0b000_11_00000_00000;
const NAMETABLE_MSB: u16    = 0b000_10_00000_00000;
const NAMETABLE_LSB: u16    = 0b000_01_00000_00000;
const COARSE_X: u16      = 0b000_00_00000_11111;
const COARSE_Y: u16      = 0b000_00_11111_00000;
const FINE_Y: u16        = 0b111_00_00000_00000;


fn inc_v_horizontal(nes: &mut Nes) {
    if (nes.ppu.v & COARSE_X) == 31 {
        nes.ppu.v &= !COARSE_X;      // sets coarse x to 0
        nes.ppu.v ^= NAMETABLE_LSB;  // toggles lower nametable bit
    } else {
        nes.ppu.v += 1;
    }
}

fn inc_v_vertical(nes: &mut Nes) {
    if ((nes.ppu.v & FINE_Y) >> 12) == 7 {
        nes.ppu.v &= !FINE_Y;
        if ((nes.ppu.v & COARSE_Y) >> 5) == 31 {
            nes.ppu.v &= !COARSE_Y;
            nes.ppu.v ^= NAMETABLE_MSB;  // this was bugged previously
        } else {
            nes.ppu.v += 0b000_00_00001_00000;
        }
    } else {
        nes.ppu.v += 0b001_00_00000_00000;
    }
}

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




fn draw_pixel(nes: &mut Nes) {

    // Render pixel!
    let lsb_attr = get_bit(nes.ppu.attr_lsb_sr, 7 - nes.ppu.x) as u16;
    let msb_attr = get_bit(nes.ppu.attr_msb_sr, 7 - nes.ppu.x) as u16;
    let lsb_ptable = get_bit_u16(nes.ppu.ptable_lsb_sr, 15 - nes.ppu.x) as u16;
    let msb_ptable = get_bit_u16(nes.ppu.ptable_msb_sr, 15 - nes.ppu.x) as u16;

    // if lsb_ptable == 0 && msb_ptable == 0 {return;}

    let palette_index = if lsb_ptable == 0 && msb_ptable == 0 {0x3F00} else {
        // need something to specify whether a sprite or background pixel is being drawn
        0x3F00 | (msb_attr << 3) | (lsb_attr << 2) | (msb_ptable << 1) | lsb_ptable
    };

    let frame_index = ((nes.ppu.scanline * 256 + nes.ppu.scanline_cycle - 1) * 4) as usize;
    
    let pixel_hue_value = read_vram(palette_index as u16, nes);
    let pixel_rgb = PALETTE[pixel_hue_value as usize];
    
    nes.frame[frame_index    ] = pixel_rgb.0;  // R
    nes.frame[frame_index + 1] = pixel_rgb.1;  // G
    nes.frame[frame_index + 2] = pixel_rgb.2;  // B
    nes.frame[frame_index + 3] =           255;  // A

    // if (frame_index / 4) % 8 == 0 {
    //     nes.frame[frame_index] = nes.frame[frame_index].wrapping_add(150);
    // }
    // if ((frame_index / 4) / 256) % 8 == 0 {
    //     nes.frame[frame_index+1] = nes.frame[frame_index+1].wrapping_add(150);
    // }

    if nes.ppu_log_toggle {
        println!("\nPixel drawn!");
        println!(
            "lsb attr = {}, msb attr = {}, lsb ptable = {}, msb ptable = {}",
            lsb_attr, 
            msb_attr,
            lsb_ptable,
            msb_ptable,
        );
        println!("palette index = {:016b} ({:04X})", palette_index, palette_index);
        println!("frame index = {}", frame_index);
        println!("raw colour byte from palette = {:02X}", pixel_hue_value);
        println!("as tuple {:?}", pixel_rgb);
        println!();
    }
    

}

const NAMETABLE_READ:   i32 = 1;
const ATTRIBUTE_READ:   i32 = 3;
const PATTERN_LSB_READ: i32 = 5;
const PATTERN_MSB_READ: i32 = 7;

fn print_ppu_log(nes: &Nes) {
    println!("Values at beginning of PPU step:");

    println!("rendering: {}", nes.ppu.show_bg || nes.ppu.show_sprites);
    println!("scanline: {}, cycle: {}", nes.ppu.scanline, nes.ppu.scanline_cycle);
    println!("v: {:016b} ({:04X})", nes.ppu.v, nes.ppu.v);
    println!("t: {:016b} ({:04X}), x: {:08b}", nes.ppu.t, nes.ppu.t, nes.ppu.x);

    println!("pt_lsb_sr: {:016b}", nes.ppu.ptable_lsb_sr);
    println!("pt_msb_sr: {:016b}", nes.ppu.ptable_msb_sr);

    println!("at_lsb_sr: {:08b}", nes.ppu.attr_lsb_sr);
    println!("at_msb_sr: {:08b}", nes.ppu.attr_msb_sr);

    println!("ntable_tmp: {:08b} ({:02X})", nes.ppu.ntable_tmp, nes.ppu.ntable_tmp);
    println!("attr_tmp: {:08b} ({:02X})", nes.ppu.attr_tmp, nes.ppu.attr_tmp);
    println!("ptable_lsb_tmp: {:08b} ({:02X})", nes.ppu.ptable_lsb_tmp, nes.ppu.ptable_lsb_tmp);
    println!("ptable_msb_tmp: {:08b} ({:02X})", nes.ppu.ptable_msb_tmp, nes.ppu.ptable_msb_tmp);

    println!("attr_lsb_latch: {:?}, attr_msb_latch: {:?}", nes.ppu.attr_lsb_latch, nes.ppu.attr_msb_latch);
    println!();
}

pub fn step_ppu(nes: &mut Nes) {
    


    if nes.ppu_log_toggle {print_ppu_log(nes);}

    // aliases
    let cycle = nes.ppu.scanline_cycle;
    let scanline = nes.ppu.scanline;
    let rendering_enabled = nes.ppu.show_bg || nes.ppu.show_sprites;
    
    // The following three conditions are mutually exclusive

    // If in visible area, draw pixel
    if (cycle >= 1 && cycle <= 256) && (scanline >= 0 && scanline <= 239) && rendering_enabled {
        draw_pixel(nes);
    }
    // At (1, 241), set PPUSTATUS in_vblank bit and raise NMI if enabled
    else if scanline == 241 && cycle == 1 {
        nes.ppu.in_vblank = true;
        if nes.ppu.nmi_enable {
            nes.cpu.nmi_interrupt = true;
        }
    } 
    // At (1, -1), clear PPUSTATUS in_vblank bit and disable NMI signal
    else if scanline == -1 && cycle == 1 {
        nes.ppu.in_vblank = false;
        nes.cpu.nmi_interrupt = false;
    }




    // Shift register reload happens on cycles {9, 17, 25, ... , 257} and {329, 337}
    let shift_register_reload = ((cycle % 8 == 1) && (cycle >= 9 && cycle <= 257)) 
                              || (cycle == 329 || cycle == 337);

    if shift_register_reload {
        // Reload pattern shift registers
        nes.ppu.ptable_lsb_sr |= nes.ppu.ptable_lsb_tmp as u16;
        nes.ppu.ptable_msb_sr |= nes.ppu.ptable_msb_tmp as u16;

        // scanline goes from 1 to 256 inclusive


        // taking 1 off just makes the tile numbering go from 1-32 to 0-31
        // it is reading ahead after all 

        // no idea how this works during prefetch, seems to though
        // I guess the wrapping doesn't matter since it's getting modded

        // also it's only using the coarse x bits, so it doesn't matter that the rest of v 
        // gets corrupted with subtraction when v = 0

        let left_num = ((nes.ppu.v & COARSE_X).wrapping_sub(1)) / 2; 
        let top_num = ((nes.ppu.v & COARSE_Y) >> 5) / 2;

        let is_top  = top_num % 2 == 0;
        let is_left = left_num % 2 == 0;


        // !!!!!!!!!
        // Coarse X and Y is not the area we are curerntly rendering in!
        // It's a little bit ahead, because v is used to read ahead


        // reads two tiles ahead? so at cycle 0, v is ...10
        // at cycle 9, it's ...11
        

        let attr = nes.ppu.attr_tmp;

        match (is_top, is_left) {
            (true, true) => {
                nes.ppu.attr_lsb_latch = get_bit(attr, 0);
                nes.ppu.attr_msb_latch = get_bit(attr, 1);
            }
            (true, false) => {
                nes.ppu.attr_lsb_latch = get_bit(attr, 2);
                nes.ppu.attr_msb_latch = get_bit(attr, 3);
            }
            (false, true) => {
                nes.ppu.attr_lsb_latch = get_bit(attr, 4);
                nes.ppu.attr_msb_latch = get_bit(attr, 5);
            }
            (false, false) => {
                nes.ppu.attr_lsb_latch = get_bit(attr, 6);
                nes.ppu.attr_msb_latch = get_bit(attr, 7);
            }
        }

        

        if nes.ppu_log_toggle {    
            println!("Temp values copied to shift register\n");
            println!("So, attribute byte is {:08b} ({:02X})", attr, attr);
            println!("left_num {} {:08b}", left_num, left_num);
            println!("top_num {} {:08b}", top_num, top_num);
            println!("is_top {}, is_left {}", is_top, is_left);
            println!("selecting {}{}", nes.ppu.attr_msb_latch as u8, nes.ppu.attr_lsb_latch as u8);
        
        }
    }





    let in_fetch_cycle = ((cycle >= 1 && cycle <= 256) || (cycle >= 321 && cycle <= 336))
                      && (scanline <= 239);  

    if in_fetch_cycle && rendering_enabled {
        // Fetching happens in 8-cycle cycles
        match cycle % 8 {
            
            NAMETABLE_READ => {
                let ntable_address = 0x2000 | (nes.ppu.v & !FINE_Y);
                nes.ppu.ntable_tmp = read_vram(ntable_address, nes);

                if nes.ppu_log_toggle {println!("read nametable byte {:02X} from {:016b} ({:04X})\n", nes.ppu.ntable_tmp, ntable_address, ntable_address);}
            }

            ATTRIBUTE_READ => {
                let attribute_addr = 0x23C0 | (nes.ppu.v & NAMETABLE) 
                                            | ((nes.ppu.v & 0b11100_00000) >> 4)
                                            | ((nes.ppu.v & 0b00000_11100) >> 2);
                nes.ppu.attr_tmp = read_vram(attribute_addr, nes);

                if nes.ppu_log_toggle {println!("read attribute byte {:02X} from {:016b} ({:04X}\n)", nes.ppu.attr_tmp, attribute_addr, attribute_addr);}
            }

            PATTERN_LSB_READ => {
                let tile_addr = ((nes.ppu.bg_ptable_select as u16) << 12) 
                              | ((nes.ppu.ntable_tmp as u16) << 4) 
                              | ((nes.ppu.v & FINE_Y) >> 12);
                nes.ppu.ptable_lsb_tmp = read_vram(tile_addr, nes);

                if nes.ppu_log_toggle {println!("read pattern lsb {:08b} from {:016b} ({:04X})\n", nes.ppu.ptable_lsb_tmp, tile_addr, tile_addr);}
            }

            PATTERN_MSB_READ => {
                let tile_addr = ((nes.ppu.bg_ptable_select as u16) << 12) 
                                | ((nes.ppu.ntable_tmp as u16) << 4) 
                                | ((nes.ppu.v & FINE_Y) >> 12)
                                + 8; // equivalently | 0b1000, lower 3 are fine y 
                nes.ppu.ptable_msb_tmp = read_vram(tile_addr, nes);

                if nes.ppu_log_toggle {println!("read pattern msb {:08b} from {:016b} ({:04X})\n", nes.ppu.ptable_msb_tmp, tile_addr, tile_addr);}
            }

            _ => ()
        }
    }

    // could tidy up these conditions later

    // Horizontal v increment happens on cycles {8, 16, 24, ... , 256} and {328, 336}
    let horizontal_v_increment = (((cycle % 8 == 0) && (cycle >= 8 && cycle <= 256)) || (cycle == 328 || cycle == 336)) && scanline <= 239;

    if horizontal_v_increment && rendering_enabled {
        if nes.ppu_log_toggle {println!("v incremented horizontally\n");}
        inc_v_horizontal(nes);
    }

    if cycle == 256 && scanline <= 239 && rendering_enabled {
        if nes.ppu_log_toggle {println!("v incremented vertically\n");}
        inc_v_vertical(nes);
    }


        /*
    
        from ppu timing graph:

        tile fetches at 321-336 fill the shift registers with stuff to render immediately on the next
        scanline 

        shift registers are loaded with stuff at ticks 9, 17, 25, ... , 257 and also ticks 329, 337

        the background shift registers do not always shift! This makes sense as all the stuff loaded
        in on the last few cycles of the previous scanline would have mostly passed through the 
        shift register by the time they were read

        the shift registers shift at the end of the cycles 2..=257, 322..=337

    
    */

    











    
    // Copy horizontal bits from t to v
    if cycle == 257 && scanline <= 239 && rendering_enabled {
        const HORIZONTAL_BITMASK: u16 = NAMETABLE_LSB | COARSE_X;

        nes.ppu.v &= !HORIZONTAL_BITMASK; // clear horizontal bits
        nes.ppu.v |= nes.ppu.t & HORIZONTAL_BITMASK;

        if nes.ppu_log_toggle {println!("copied horizontal bits from t to v\n");}
    }


    if (cycle >= 280 && cycle <= 304) && scanline == -1 && rendering_enabled {
        const VERTICAL_BITMASK: u16 = FINE_Y | NAMETABLE_MSB | COARSE_Y;

        nes.ppu.v &= !VERTICAL_BITMASK;
        nes.ppu.v |= nes.ppu.t & VERTICAL_BITMASK;

        if nes.ppu_log_toggle {println!("copied vertical bits from t to v\n");}
    }



    // the shift registers shift at the end of the cycles 2..=257, 322..=337
    
    if (cycle >= 2 && cycle <= 257) || (cycle >= 322 && cycle <= 337) {
        nes.ppu.ptable_lsb_sr <<= 1;
        nes.ppu.ptable_msb_sr <<= 1;
    
        nes.ppu.attr_lsb_sr <<= 1;
        nes.ppu.attr_msb_sr <<= 1;
        nes.ppu.attr_lsb_sr |= nes.ppu.attr_lsb_latch as u8;
        nes.ppu.attr_msb_sr |= nes.ppu.attr_msb_latch as u8;
    }



    // Wrap scanline cycles
    if nes.ppu.scanline_cycle < 340 {
        nes.ppu.scanline_cycle += 1;
    } else {
        nes.ppu.scanline_cycle = 0;
        // Wrap scanline
        if nes.ppu.scanline < 260 {
            nes.ppu.scanline += 1;
        } else {
            // Pre-render scanline is -1 instead of 261 for convenience
            nes.ppu.scanline = -1;
        }
    }

}



/*
 https://archive.nes.science/nesdev-forums/f3/t13966.xhtml
 https://forums.nesdev.org/viewtopic.php?t=8216
 https://www.nesdev.org/2C02%20technical%20reference.TXT

 Okay, so here's a reminder on how the PPU works, more complicated than CPU
 PPU is non-programmable, follows the same procedure every time with the exception of a few 
 options that change the functionality slightly

 PPU has 16kB (0x4000) of addressable memory

 Holds the following:
     - Pattern tables, which contain graphics data from the cartridge
     - Nametables, which are matrices of indices into the pattern table and define which tiles should
       make up the background
     - Attribute tables, which define what palette is used to draw each segment of the screen
     - Palettes, which define the colours used to draw the image

 PPU also has Object Attribute Memory (OAM) but this is not directly addressable and can only be 
 modified using the PPUs memory mapped registers OAMADDR OAMDATA OAMDMA

 The memory is laid out in the following way:
     First 4kB (0x0000-0x0FFF) is pattern table 0 which is directly mapped in from CHR_ROM
     Second 4kB (0x1000-1FFF) is pattern table 1, also mapped directly from CHR_ROM
     These can be modified using mappers and stuff, but I'm not thinking about that yet

     Next 2kB is the nametables, 1kB for each. This is the actual VRAM in the console
     There are actually 4 logical nametables, starting at 0x2000,0x2400,0x2800,0x2C00
     These are arranged in a grid, left to right, top to bottom
     This means two logical nametables must point to the same place because there is only enough 
     memory for two in VRAM
     Nametables are either mirrored horizontally or vertically

     So basically first 4kB pattern tables (chr_rom), next 4kB nametables

     Then the next 3840B (4kB - 256B) mirrors the first 3840B of the nametables

     The next 32B are palette ram (from 0x3F00-0x3F1F)

     Then the next 0x3F20-0x3FFF (224B) mirror the palette tables

    Each tile in the pattern table is 8x8 pixels with a 2-bit colour depth
    
     Each tile is represented in the following way:

     https://emudev.de/nes-emulator/cartridge-loading-pattern-tables-and-ppu-registers/

     16 bytes, two 8 byte planes one after the other
     first 8 bytes control bit 0 of the colour
     second 8 bytes control bit 1 of the colour

     so to construct one horizontal line of the image, you need to grab the appropriate pattern
     table entry, and then the one 8 bytes over

     The resolution of the screen is 256x240, so 32x30 tiles, so 32*30=960B of nametable
     The last 1024-960=64B make up the attribute table

     Each byte controls the pallete of 4x4 tile area (32x32 pixels)
     The 4x4 tile area is split into 4 2x2 tile areas (16x16 pixels each)
     This is the size of a mario question block
    
     The order is top left, top right, bottom left, bottom right 
     bits 1,0 top left
     bits 3,2 top right
     bits 5,4 bottom left
     bits 7,6 bottom right

     https://www.nesdev.org/wiki/PPU_attribute_tables
    
     7654 3210
     |||| ||++- Color bits 3-2 for top left quadrant of this byte
     |||| ++--- Color bits 3-2 for top right quadrant of this byte
     ||++------ Color bits 3-2 for bottom left quadrant of this byte
     ++-------- Color bits 3-2 for bottom right quadrant of this byte

     Pattern table, attribute table, nametable plane 0, nametable plane 1, fetched in that order
     Each read takes two cycles, so 8 cycles in total, which keeps pace with the 8 pixels it has
     to render in that time
    
     2 16-bit shift registers store the pattern table data
     first one stores plane 0, second plane 1
     They are 16-bits so they can store 2 tiles in there, (so that there's a buffer baiscally)

     These values are stored in internal latches once read, and all blitted to the shift registers
     at the same time every 8 cycles

     figure out attribute shift registers next, there are 2 8-bit ones? 

     yes, so there are two 8-bit ones
     one is for the lower bit, one for the upper bit of the attribute for that pixel

     these shift registers are shifted in tandem with the pattern table ones so that a pixel 
     defined in bit 5 of the pattern table shift register corresponds to attribute in bit 5 
     of the attribute one

     vram accesses take two cycles because of some pin sharing stuff to minimise pins I guess
     it's safe to just read the value from vram on the second half of the cycle I think
     https://documentation.help/FCEUX/%7B9C73EB3E-118D-451A-AAE8-BBF99A5FDEEB%7D.htm

     So the v register (vram address) is used to read the nametables
     the attribute table and pattern table addresses are derived from v, just using some bitwise stuff
     https://www.nesdev.org/wiki/PPU_scrolling#Tile_and_attribute_fetching
    
     To get anywhere with this, need to fully understand how V works
    
     v and t are 15 bits long

     v is like this:

    yyy NN YYYYY XXXXX
    ||| || ||||| +++++-- coarse X scroll
    ||| || +++++-------- coarse Y scroll
    ||| ++-------------- nametable select
    +++----------------- fine Y scroll

     The 5 bits for X and Y can make 0-31, enough to specify any block in the 32x30 nametable

     t is kind of like a temporary variable for setting v
     when you write stuff to PPUCTRL or PPUSCROLL, it will go to different parts of t instead of v

     When you write to PPUCTRL, it will copy the nametable base address into t

    t: ...GH.. ........ <- d: ......GH
       <used elsewhere> <- d: ABCDEF..
    
    Alright, so the scroll coordinates of the screen is set with PPUSCROLL

    This is done by writing twice to PPUSCROLL, first with X, then with Y

    8 bits can represent 256 values, perfect for speficying a pixel on the screen

    the 8 bits are split up into the upper 5 and lower 3 bits

    the lower bits are called the "fine" x and y, because they specify a pixel inside of one 8x8 tile

    the upper 5 are used to specify a tile to select (coarse x and y)

    when writing X to PPUSCROLL, the upper 5 go to the bottom of t, the lower 3 get put in a separate
    register; x, fine x scroll
    it also toggles the write bit thing

    when writing Y, the upper 5 bits go into t, the 3 lower go to the upper bits of t

    although the fine y is stored in the MSBs of t, I don't think they're included when addressing vram
    yep, it's not, instead 0x2000 is added to the NN YYYYY XXXXX thing I think

    writes to PPUADDR will set v directly

    the first write will take the 6 lsbs and put them in the top half of t
    it will also clear the highest bit (15th, not 16th) for some reason? 

    the second write will take all bits and put them in the bottom 8 bits of t
    t is then copied entirely into v

    t is not copied entirely when there are two writes to scroll though 

    t is copied into v right at the start of the frame automatically
    
    right, now need to cover how the address changes as the image is rendered

    when we're scanning the image and need to get the next tile, coarse X is increased, which makes sense
    this overflows into bit 10 of v (the lower nametable bit), which will cross to the next nametable

    This makes sense for scrolling horizontally right

    if it didn't do this, it would just overflow into Y which wouldn't make any sense if the screen 
    is between two nametables, like in mario

    incrementing the Y works differently

    at the end of every scanline, fine y is incremented which makes sense
    not entirely sure how this actually works though, since fine y isn't part of the address? 

    fine y overflows into coarse y, since every 8 tile rows you'll need to grab a lower tile

    coarse y then overflows into the 11th bit, vertical nametable, makes sense for vertical scrolling

    tile address      = 0x2000 | (v & 0x0FFF)           0xFFF is just 12 bits, NNYYYYYXXXXX
    attribute address = 0x23C0 | (v & 0x0C00) | ((v >> 4) & 0x38) | ((v >> 2) & 0x07)
    
    so, fine x isn't used in the calculation either, actually just selects a bit from the shift registers

    fine y is then only used once you've used the nametable value as an index into the pattern table

    so like, you get the index, get the starting address of a tile, then just add fine y to move vertically
    through the bytes that make up the tile! tada!
    
    now the only thing to do is figure out the timings? I guess? 

    I know some scanlines are longer than others etc, and what happens when rendering is disabled? 

    cycle 0 is idle!
    right so the reads happen over 1-2, 3-4, 5-6 etc. 
    I'm just going to do it on the last cycle of each, so 2, 4, 6 becuase it looks nicer

    there are 4 reads, two cycles each, this covers one tile

    nametable is 2, 10, 18, ... 242, 250
    attribute is 4, 12, 20, ... 244, 252
    patternL  is 6, 14, 22, ... 246, 254
    patternH  is 8, 16, 24, ... 248, 256

    at the end of cycles 8, 16, ..., v is incremented horizontally
    at the beginning of cycles 8+1, 16+1, the latches are opened filling the upper bits of shift registers

    at cycle 256, v is finally incremented vertically

    at cycle 257, horizontal bits of t are copied into v, I guess if you change the scrolling
    mid frame for split scroll

    then, loads of sprite shit happens in the background between cycles 257-320

    322, 330
    324, 332
    326, 334
    328, 336 

    the first two tiles are fetched and placed in the shift registers before the start of the next scanline

    then, finally, there are two NT byte fetches spanning 337-340
    who knows why. I'm going to just leave these for the now, apparently one mapper uses them for timing


    this all works itself down until scanline 239, which is the last normal one
    then at 240, there is a completely idle scanline

    then at 241, cycle 1, PPU vblank flag in PPUSTATUS is set, and NMI occurs, i guess only if 
    the nmi bit is set!

    attribute address = 0x23C0 | (v & 0x0C00) | ((v >> 4) & 0x38) | ((v >> 2) & 0x07)

     right, there are 8 attribute "squares" running horizontally
     therefore, the highest 3 coarse X bits should control which one gets selected horizontally

     first attribute table starts at 0x23C0
     then extract nametable select with mask, this literally just selects the nametable
    
     NN----YYYXXX
    
     nametable entry is 0-255 which 
     pattern table has 256 tiles, so...

     0x0000-0x0FFF

     right so one tile is 16 bytes, so need to skip 16 at a time
     so lower 4 bits need to be 0000
     then nametable number just goes above that, i think

     then, just stick the fine y on the end! that will select the row

     right, so last thing to do is just put in the horizontal and vertical increments at certain cycle numbers
     also the pre render scanline!

     also need to let you disable rendering, still don't really know what exactly that does 

     listen, just come back later today after tea and can get this working
     it's still a fair bit off working, like several hours of work because I need to check
     nmi, the rendering enabled disabled, v and h increments in the cycle, the last cycle
     updating the shift registers, the code that determines the pixel value from 
     the attribute, fine x, two pattern table lsb and msbs, etc. 

     still maybe 3/4 hours of work, so don't get distracted when you only have a couple 
     days until exam! You can do it!

*/
    /*
    
    https://www.nesdev.org/wiki/PPU_attribute_tables
    
     7654 3210
     |||| ||++- Color bits 3-2 for top left quadrant of this byte
     |||| ++--- Color bits 3-2 for top right quadrant of this byte
     ||++------ Color bits 3-2 for bottom left quadrant of this byte
     ++-------- Color bits 3-2 for bottom right quadrant of this byte

    

     Right, so, bits of coarse X and Y will need to be used here

     one attribute byte assigns a colour to 4 16x16 areas

     so, second byte of coarse X will flip every 2 tiles (16 pixels)
     so if it's 0, we're on the left side of the attribute area
     if it's a 1, we're on the right side of the attribute area

     second byte of coarse Y will also flip every 2 tiles, 
     if it's 0, we're on top side of attribute area
     if it's 1, we're at bottom
    
    ~y, ~x = top left
    ~y,  x = top right
     y, ~x = bottom left
     y,  x = bottom right

    so, if neither bits are on, get bits 1, 0
    if x is on, get bits 3, 2
    if y is on, get bits 5, 4
    if both, bits 7, 6

    so x contributes 2 bits distance
    y contributes 4 bits distance
    
    cycle 1 through 256 are rendered

    alright, final steps for getting things working in some way.
    how exactly will the cycles update? 

    43210
    |||||
    |||++- Pixel value from tile data
    |++--- Palette number from attribute table or OAM
    +----- Background/Sprite select
    */