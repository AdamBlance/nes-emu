use crate::hw::*;
use crate::util::*;
use std::{thread, time};

pub static PALETTE: [(u8,u8,u8); 56] = [(84, 84, 84), (0, 30, 116), (8, 16, 144), (48, 0, 136), (68, 0, 100), (92, 0, 48), (84, 4, 0), (60, 24, 0), (32, 42, 0), (8, 58, 0), (0, 64, 0), (0, 60, 0), (0, 50, 60), (0, 0, 0), (152, 150, 152), (8, 76, 196), (48, 50, 236), (92, 30, 228), (136, 20, 176), (160, 20, 100), (152, 34, 32), (120, 60, 0), (84, 90, 0), (40, 114, 0), (8, 124, 0), (0, 118, 40), (0, 102, 120), (0, 0, 0), (236, 238, 236), (76, 154, 236), (120, 124, 236), (176, 98, 236), (228, 84, 236), (236, 88, 180), (236, 106, 100), (212, 136, 32), (160, 170, 0), (116, 196, 0), (76, 208, 32), (56, 204, 108), (56, 180, 204), (60, 60, 60), (236, 238, 236), (168, 204, 236), (188, 188, 236), (212, 178, 236), (236, 174, 236), (236, 174, 212), (236, 180, 176), (228, 196, 144), (204, 210, 120), (180, 222, 120), (168, 226, 144), (152, 226, 180), (160, 214, 228), (160, 162, 160)];

// https://archive.nes.science/nesdev-forums/f3/t13966.xhtml
// https://forums.nesdev.org/viewtopic.php?t=8216
// https://www.nesdev.org/2C02%20technical%20reference.TXT

// This will need a complete overhaul, use the resources above 
// please don't get distracted on this when you've got less than a week until your first exam!

// Yeah this is a bit of a mess, try to get some good functions first, identify common functionality
// Otherwise this is just a mess of nonsense





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



// the mirroring depends on the cartridge!!!
// will need to store that as an attribute and check it

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

// Okay, so here's a reminder on how the PPU works, more complicated than CPU
// PPU is non-programmable, follows the same procedure every time with the exception of a few 
// options that change the functionality slightly

// PPU has 16kB (0x4000) of addressable memory

// Holds the following

    // Pattern tables, which contain graphics data from the cartridge

    // Nametables, which are matrices of indices into the pattern table and define which tiles should
    // make up the background

    // Attribute tables, which define what palette is used to draw each segment of the screen

    // Palettes, which define the colours used to draw the image

// PPU also has Object Attribute Memory (OAM) but this is not directly addressable and can only be 
// modified using the PPUs memory mapped registers OAMADDR OAMDATA OAMDMA


// The memory is laid out in the following way:
    // First 4kB (0x0000-0x0FFF) is pattern table 0 which is directly mapped in from CHR_ROM
    // Second 4kB (0x1000-1FFF) is pattern table 1, also mapped directly from CHR_ROM
    // These can be modified using mappers and stuff, but I'm not thinking about that yet

    // Next 2kB is the nametables, 1kB for each. This is the actual VRAM in the console
    // There are actually 4 logical nametables, starting at 0x2000,0x2400,0x2800,0x2C00
    // These are arranged in a grid, left to right, top to bottom
    // This means two logical nametables must point to the same place because there is only enough 
    // memory for two in VRAM
    // Nametables are either mirrored horizontally or vertically

    // So basically first 4kB pattern tables (chr_rom), next 4kB nametables

    // Then the next 3840B (4kB - 256B) mirrors the first 3840B of the nametables

    // The next 32B are palette ram (from 0x3F00-0x3F1F)

    // Then the next 0x3F20-0x3FFF (224B) mirror the palette tables

// I think instead of code, I need to do a flowchart or something to make sure I understand absolutely 
// everything that's going on, cause otherwise I'm just writing nonsensical if statements 

// Like, can I get away with just reading directly from the pagetables? Or do I have to simulate the 
// shift registers to get accurate simulation? Probably not, although maybe if some random reads and
// writes happen mid frame which some games do. 


pub fn step_ppu(nes: &mut Nes) {

    // Okay, when I have time and not studying, need to work through wiki and just perfectly replicate
    // what happens every cycle (within reason) 
    // A good place to start would be to see what the power up state is and match it
    // Honestly I think it would be good to rename all of the PPU registers as well to include
    // the name of the memory mapped register? Otherwise it's just going to get confusing


    // Each tile in the pattern table is 8x8 pixels with a 2-bit colour depth
    
    // Each tile is represented in the following way:

    // https://emudev.de/nes-emulator/cartridge-loading-pattern-tables-and-ppu-registers/

    // 16 bytes, two 8 byte planes one after the other
    // first 8 bytes control bit 0 of the colour
    // second 8 bytes control bit 1 of the colour

    // so to construct one horizontal line of the image, you need to grab the appropriate pattern
    // table entry, and then the one 8 bytes over

    // The resolution of the screen is 256x240, so 32x30 tiles, so 32*30=960B of nametable
    // The last 1024-960=64B make up the attribute table

    // Each byte controls the pallete of 4x4 tile area (32x32 pixels)
    // The 4x4 tile area is split into 4 2x2 tile areas (16x16 pixels each)
    // This is the size of a mario question block
    
    // The order is top left, top right, bottom left, bottom right 
    // bits 1,0 top left
    // bits 3,2 top right
    // bits 5,4 bottom left
    // bits 7,6 bottom right

    // https://www.nesdev.org/wiki/PPU_attribute_tables
    
    // 7654 3210
    // |||| ||++- Color bits 3-2 for top left quadrant of this byte
    // |||| ++--- Color bits 3-2 for top right quadrant of this byte
    // ||++------ Color bits 3-2 for bottom left quadrant of this byte
    // ++-------- Color bits 3-2 for bottom right quadrant of this byte

    // Pattern table, attribute table, nametable plane 0, nametable plane 1, fetched in that order
    // Each read takes two cycles, so 8 cycles in total, which keeps pace with the 8 pixels it has
    // to render in that time
    
    // 2 16-bit shift registers store the pattern table data
    // first one stores plane 0, second plane 1
    // They are 16-bits so they can store 2 tiles in there, (so that there's a buffer baiscally)

    // These values are stored in internal latches once read, and all blitted to the shift registers
    // at the same time every 8 cycles

    // figure out attribute shift registers next, there are 2 8-bit ones? 

    // yes, so there are two 8-bit ones
    // one is for the lower bit, one for the upper bit of the attribute for that pixel

    // these shift registers are shifted in tandem with the pattern table ones so that a pixel 
    // defined in bit 5 of the pattern table shift register corresponds to attribute in bit 5 
    // of the attribute one

    // vram accesses take two cycles because of some pin sharing stuff to minimise pins I guess
    // it's safe to just read the value from vram on the second half of the cycle I think
    // https://documentation.help/FCEUX/%7B9C73EB3E-118D-451A-AAE8-BBF99A5FDEEB%7D.htm


    // So the v register (vram address) is used to read the nametables
    // the attribute table and pattern table addresses are derived from v, just using some bitwise stuff
    // https://www.nesdev.org/wiki/PPU_scrolling#Tile_and_attribute_fetching
    



    // To get anywhere with this, need to fully understand how V works
    
    // v and t are 15 bits long


    // v is like this:

    /*

    yyy NN YYYYY XXXXX
    ||| || ||||| +++++-- coarse X scroll
    ||| || +++++-------- coarse Y scroll
    ||| ++-------------- nametable select
    +++----------------- fine Y scroll
    
    */

    // The 5 bits for X and Y can make 0-31, enough to specify any block in the 32x30 nametable

    // t is kind of like a temporary variable for setting v
    // when you write stuff to PPUCTRL or PPUSCROLL, it will go to different parts of t instead of v

    // When you write to PPUCTRL, it will copy the nametable base address into t

    /*
    t: ...GH.. ........ <- d: ......GH
       <used elsewhere> <- d: ABCDEF..
    */

    

    /*
    
    
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
    
    
    
    

    */
    

    // cycle 0 is idle!
    // right so the reads happen over 1-2, 3-4, 5-6 etc. 
    // I'm just going to do it on the last cycle of each, so 2, 4, 6 becuase it looks nicer

    // there are 4 reads, two cycles each, this covers one tile

    // nametable is 2, 10, 18, ... 242, 250
    // attribute is 4, 12, 20, ... 244, 252
    // patternL  is 6, 14, 22, ... 246, 254
    // patternH  is 8, 16, 24, ... 248, 256
    
    // at the end of cycles 8, 16, ..., v is incremented horizontally
    // at the beginning of cycles 8+1, 16+1, the latches are opened filling the upper bits of shift registers

    // at cycle 256, v is finally incremented vertically

    // at cycle 257, horizontal bits of t are copied into v, I guess if you change the scrolling
    // mid frame for split scroll





    // then, loads of sprite shit happens in the background between cycles 257-320

    // 322, 330
    // 324, 332
    // 326, 334
    // 328, 336 

    // the first two tiles are fetched and placed in the shift registers before the start of the next scanline

    // then, finally, there are two NT byte fetches spanning 337-340
    // who knows why. I'm going to just leave these for the now, apparently one mapper uses them for timing


    // this all works itself down until scanline 239, which is the last normal one
    // then at 240, there is a completely idle scanline

    // then at 241, cycle 1, PPU vblank flag in PPUSTATUS is set, and NMI occurs, i guess only if 
    // the nmi bit is set!





    // WOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOO
    // DONE !
    // other than sprite stuff of course


    // NN YYYYY XXXXX
    // 
    
    const NO_FINE_Y: u16      = 0b000_11_11111_11111;
    const ONLY_FINE_Y: u16    = 0b111_00_00000_00000;
    const ONLY_NAMETABLE: u16 = 0b000_11_00000_00000;


    // idle at scanline 0
    if nes.ppu.scanline_cycle == 0 {return};

    // if in visible area or fetching data for next scanline
    if nes.ppu.scanline < 240 && ( nes.ppu.scanline_cycle <= 256  || nes.ppu.scanline_cycle >= 322 ) {

        match nes.ppu.scanline_cycle % 8 {

            //attribute address = 0x23C0 | (v & 0x0C00) | ((v >> 4) & 0x38) | ((v >> 2) & 0x07)

            // right, there are 8 attribute "squares" running horizontally
            // therefore, the highest 3 coarse X bits should control which one gets selected horizontally

            // first attribute table starts at 0x23C0
            // then extract nametable select with mask, this literally just selects the nametable
            

            // NN----YYYXXX
            

            // nametable entry is 0-255 which 
            // pattern table has 256 tiles, so...

            // 0x0000-0x0FFF

            // right so one tile is 16 bytes, so need to skip 16 at a time
            // so lower 4 bits need to be 0000
            // then nametable number just goes above that, i think

            // then, just stick the fine y on the end! that will select the row

            2 => nes.ppu.nametable_latch = read_vram(0x2000 | (nes.ppu.v & NO_FINE_Y), nes),
            4 => {
                let attribute_addr = 0x23C0 | (nes.ppu.v & ONLY_NAMETABLE) 
                                            | ((nes.ppu.v & 0b11100_00000) >> 4)
                                            | ((nes.ppu.v & 0b00000_11100) >> 2);
                nes.ppu.attribute_latch = read_vram(attribute_addr, nes);
            }
            6 => {
                // yep, so we need to skip 16 bytes at a time 
                let tile_addr = ((nes.ppu.bg_ptable_select as u16) << 12) 
                                    | ((nes.ppu.nametable_latch as u16) << 4) 
                                    | ((nes.ppu.v & ONLY_FINE_Y) >> 12);
                nes.ppu.lsb_pattern_table_latch = read_vram(tile_addr, nes);
            }
            0 => {
                // yep, so we need to skip 16 bytes at a time 
                let tile_addr = ((nes.ppu.bg_ptable_select as u16) << 12) 
                                | ((nes.ppu.nametable_latch as u16) << 4) 
                                | ((nes.ppu.v & ONLY_FINE_Y) >> 12)
                                + 8; // equivalently | 0b1000, lower 3 are fine y 
                nes.ppu.msb_pattern_table_latch = read_vram(tile_addr, nes);
            }

            // right, so last thing to do is just put in the horizontal and vertical increments at certain cycle numbers
            // also the pre render scanline!

            // also need to let you disable rendering, still don't really know what exactly that does 

            
        }

    }




}