use super::mem::read_vram;
use crate::nes::Nes;
use crate::util::*;

pub static PALETTE: [(u8, u8, u8); 64] = [
    (84, 84, 84),
    (0, 30, 116),
    (8, 16, 144),
    (48, 0, 136),
    (68, 0, 100),
    (92, 0, 48),
    (84, 4, 0),
    (60, 24, 0),
    (32, 42, 0),
    (8, 58, 0),
    (0, 64, 0),
    (0, 60, 0),
    (0, 50, 60),
    (0, 0, 0),
    (0, 0, 0),
    (0, 0, 0),
    (152, 150, 152),
    (8, 76, 196),
    (48, 50, 236),
    (92, 30, 228),
    (136, 20, 176),
    (160, 20, 100),
    (152, 34, 32),
    (120, 60, 0),
    (84, 90, 0),
    (40, 114, 0),
    (8, 124, 0),
    (0, 118, 40),
    (0, 102, 120),
    (0, 0, 0),
    (0, 0, 0),
    (0, 0, 0),
    (236, 238, 236),
    (76, 154, 236),
    (120, 124, 236),
    (176, 98, 236),
    (228, 84, 236),
    (236, 88, 180),
    (236, 106, 100),
    (212, 136, 32),
    (160, 170, 0),
    (116, 196, 0),
    (76, 208, 32),
    (56, 204, 108),
    (56, 180, 204),
    (60, 60, 60),
    (0, 0, 0),
    (0, 0, 0),
    (236, 238, 236),
    (168, 204, 236),
    (188, 188, 236),
    (212, 178, 236),
    (236, 174, 236),
    (236, 174, 212),
    (236, 180, 176),
    (228, 196, 144),
    (204, 210, 120),
    (180, 222, 120),
    (168, 226, 144),
    (152, 226, 180),
    (160, 214, 228),
    (160, 162, 160),
    (0, 0, 0),
    (0, 0, 0),
];

pub const NAMETABLE: u16 = 0b000_11_00000_00000;
pub const NAMETABLE_MSB: u16 = 0b000_10_00000_00000;
pub const NAMETABLE_LSB: u16 = 0b000_01_00000_00000;
pub const COARSE_X: u16 = 0b000_00_00000_11111;
pub const COARSE_Y: u16 = 0b000_00_11111_00000;
pub const FINE_Y: u16 = 0b111_00_00000_00000;

const NAMETABLE_READ: i32 = 1;
const ATTRIBUTE_READ: i32 = 3;
const PATTERN_LSB_READ: i32 = 5;
const PATTERN_MSB_READ: i32 = 7;

/*

    the only way that the PPU directly changes the state of the CPU is by changing the NMI line,
    just like the apu raising the irq flag.
    This whole thing could be written with methods. The advantage is that you don't have to preface
    all fields with nes.ppu. instead of self.

*/

pub fn step_ppu(nes: &mut Nes) {
    nes.cart.ppu_tick(nes.ppu.addr_bus);

    // Aliases
    let cycle = nes.ppu.scanline_cycle;
    let scanline = nes.ppu.scanline;
    let rendering_enabled = nes.ppu.show_bg || nes.ppu.show_sprites;

    // could move this down or whatever
    // the shift registers shift at the end(?) of the cycles 2..=257, 322..=337
    if (2..=257).contains(&cycle) || (322..=337).contains(&cycle) {
        nes.ppu.bg_ptable_lsb_sr <<= 1;
        nes.ppu.bg_ptable_msb_sr <<= 1;

        nes.ppu.bg_attr_lsb_sr <<= 1;
        nes.ppu.bg_attr_msb_sr <<= 1;
        nes.ppu.bg_attr_lsb_sr |= nes.ppu.bg_attr_lsb_latch as u8;
        nes.ppu.bg_attr_msb_sr |= nes.ppu.bg_attr_msb_latch as u8;
    }

    // If in visible area, draw pixel
    if (1..=256).contains(&cycle) && (0..=239).contains(&scanline) && rendering_enabled {
        // secondary OAM initialisation spans cycles 1..=64 in reality
        if cycle == 1 {
            nes.ppu.s_oam = [0xFF; 32];
        }

        // sprite evaluation spans cycles 65..=256 in reality
        if cycle == 65 {
            let sprite_height = if nes.ppu.tall_sprites { 16 } else { 8 };

            let mut n: usize = 0;
            nes.ppu.in_range_counter = 0;

            // loop until 8 in-range sprites have been found,
            // or until all sprites in oam have been checked
            while (nes.ppu.in_range_counter < 8) && (n < 256) {
                let sprite_y = nes.ppu.oam[n] as i32;
                // if in range
                if (sprite_y <= nes.ppu.scanline) && (sprite_y + sprite_height > nes.ppu.scanline) {
                    if n == 0 {
                        nes.ppu.sprite_zero_in_soam = true;
                    }
                    // copy sprite data from oam to secondary oam
                    for i in 0..4usize {
                        nes.ppu.s_oam[((nes.ppu.in_range_counter * 4) as usize) + i] =
                            nes.ppu.oam[n + i];
                    }
                    // move index of next free space in secondary oam
                    nes.ppu.in_range_counter += 1;
                }
                // move n to next sprite in oam
                n += 4;
            }
        }

        // After this point, both the background pixel and sprite pixel are calculated
        // One of these will be chosen to draw depending on certain factors

        // Get the pattern data ready for both background and sprite
        // Then depending on which is chosen, calculate the palette index

        // For the background, the index is calculated from the pattern data + attribute data
        // For the sprite, the index is calculated from the pattern data + the palette number
        // in the attribute byte of OAM

        // First, sprite pixel
        // Initialise sprite pixel to 0 (transparency)
        // If no non-transparent sprites are found, these values will not be reassigned, so the
        // background will be drawn instead

        let mut sprite_patt_lsb = false;
        let mut sprite_patt_msb = false;
        let mut sprite_palette_number = 0u16;
        let mut draw_sprite_behind = true;

        let mut sprite_number = 0xFF;

        // Loop through all sprites on the next scanline (up to 8)
        for i in 0..8 {
            if nes.ppu.sprite_x_counters[i] == 0 {
                let patt_lsb = get_bit(nes.ppu.sprite_ptable_lsb_srs[i], 7);
                let patt_msb = get_bit(nes.ppu.sprite_ptable_msb_srs[i], 7);

                // If sprite pixel is not transparent, choose this sprite to draw and break
                // from the loop.
                if patt_lsb || patt_msb {
                    let properties = nes.ppu.sprite_property_latches[i];

                    sprite_patt_lsb = patt_lsb;
                    sprite_patt_msb = patt_msb;
                    sprite_palette_number = (properties & 0b00000011) as u16;
                    draw_sprite_behind = get_bit(properties, 5);
                    sprite_number = i;

                    break;
                }
            }
        }

        // Next, determine background pixel

        // Get background pattern
        let bg_patt_lsb = get_bit(nes.ppu.bg_ptable_lsb_sr, 15 - nes.ppu.x);
        let bg_patt_msb = get_bit(nes.ppu.bg_ptable_msb_sr, 15 - nes.ppu.x);

        for i in 0..8 {
            // If a sprite's x counter has reached zero, shift pattern table registers
            if nes.ppu.sprite_x_counters[i] == 0 {
                nes.ppu.sprite_ptable_lsb_srs[i] <<= 1;
                nes.ppu.sprite_ptable_msb_srs[i] <<= 1;
            }
        }

        // Decrement all sprite x counters until they hit 0
        for i in 0..8 {
            // If an x counter hasn't reached zero yet, decrement it
            if nes.ppu.sprite_x_counters[i] > 0 {
                nes.ppu.sprite_x_counters[i] -= 1;
            }
        }

        /*
        Now, decide whether to render the bacground or sprite

        if sprite is opaque AND (sprite has fg priority OR background is zero), sprite

        if background is opaque AND (sprite has bg priority OR sprite is zero), sprite

        Otherwise, draw 0x3F00 colour
        */

        let bg_transparent = (!bg_patt_lsb && !bg_patt_msb)
            || (!nes.ppu.show_leftmost_bg && cycle <= 8)
            || !nes.ppu.show_bg;
        let sprite_transparent = (!sprite_patt_lsb && !sprite_patt_msb)
            || (!nes.ppu.show_leftmost_sprites && cycle <= 8)
            || !nes.ppu.show_sprites;

        if !bg_transparent
            && !sprite_transparent
            && cycle < 256
            && sprite_number == 0
            && nes.ppu.sprite_zero_in_latches
            && !(cycle <= 8 && (!nes.ppu.show_leftmost_bg || !nes.ppu.show_leftmost_sprites))
        {
            nes.ppu.sprite_zero_hit = true;
        }

        let palette_index = {
            // these conditions are pretty redundant, will get around to improving them
            // just want to be exhaustive for the now and get this working

            // This is probably to blame if things break
            if bg_transparent && sprite_transparent {
                0x3F00
            } else if !sprite_transparent && (bg_transparent || !draw_sprite_behind) {
                0x3F10 | (sprite_palette_number << 2)   // fuckin gross looking
                       | ((sprite_patt_msb as u16) << 1)
                       |  (sprite_patt_lsb as u16)
            } else if !bg_transparent && (sprite_transparent || draw_sprite_behind) {
                let lsb_attr = get_bit(nes.ppu.bg_attr_lsb_sr, 7 - nes.ppu.x) as u16;
                let msb_attr = get_bit(nes.ppu.bg_attr_msb_sr, 7 - nes.ppu.x) as u16;

                0x3F00
                    | (msb_attr << 3)
                    | (lsb_attr << 2)
                    | ((bg_patt_msb as u16) << 1)
                    | (bg_patt_lsb as u16)
            } else {
                panic!("Idk what would reach here?")
            }
        };

        // Which pixel in the frame are we drawing to?
        let frame_index = ((nes.ppu.scanline * 256 + nes.ppu.scanline_cycle - 1) * 4) as usize;

        // Finally, palette index will point to the colour to be drawn

        // this isn't a normal memory access I don't think
        // I think palette memory can be accessed internally without a proper memory read
        let pixel_hue_value = read_vram(palette_index, nes) & 0b0011_1111;

        let mut pixel_rgb = PALETTE[pixel_hue_value as usize];

        let r_delta = pixel_rgb.0 / 4;
        let g_delta = pixel_rgb.1 / 4;
        let b_delta = pixel_rgb.2 / 4;

        if nes.ppu.red_emphasis {
            pixel_rgb.0 = pixel_rgb.0.saturating_add(r_delta);
            pixel_rgb.1 = pixel_rgb.1.saturating_sub(g_delta);
            pixel_rgb.2 = pixel_rgb.2.saturating_sub(b_delta);
        }
        if nes.ppu.green_emphasis {
            pixel_rgb.0 = pixel_rgb.0.saturating_sub(r_delta);
            pixel_rgb.1 = pixel_rgb.1.saturating_add(g_delta);
            pixel_rgb.2 = pixel_rgb.2.saturating_sub(b_delta);
        }
        if nes.ppu.blue_emphasis {
            pixel_rgb.0 = pixel_rgb.0.saturating_sub(r_delta);
            pixel_rgb.1 = pixel_rgb.1.saturating_sub(g_delta);
            pixel_rgb.2 = pixel_rgb.2.saturating_add(b_delta);
        }

        // TODO: Note frame thing here
        // Draw the pixel!

        if let Some(frame) = nes.frame.as_ref() {
            frame.borrow_mut()[frame_index] = pixel_rgb.0; // R
            frame.borrow_mut()[frame_index + 1] = pixel_rgb.1; // G
            frame.borrow_mut()[frame_index + 2] = pixel_rgb.2; // B
            frame.borrow_mut()[frame_index + 3] = 255; // A
        }

        if cycle == 256 {
            nes.ppu.sprite_zero_in_latches = false;
        }
    }
    // At (1, 241), set PPUSTATUS in_vblank bit and raise NMI if enabled
    else if scanline == 241 && cycle == 1 {
        nes.ppu.in_vblank = true;
        if nes.ppu.nmi_enable {
            nes.ppu.nmi_line = true;
        }
    }
    // At (1, -1), clear PPUSTATUS in_vblank bit and disable NMI signal
    else if scanline == -1 && cycle == 1 {
        nes.ppu.in_vblank = false;
        nes.ppu.sprite_zero_hit = false;
        nes.ppu.nmi_line = false;
        // nes.ppu.sprite_zero_in_latches = false;
        // nes.ppu.sprite_zero_in_soam = false;
    }
    // Shift register reload happens on cycles {9, 17, 25, ... , 257} and {329, 337}
    let shift_register_reload =
        ((cycle % 8 == 1) && (9..=257).contains(&cycle)) || (cycle == 329 || cycle == 337);

    if shift_register_reload {
        // Reload pattern shift registers
        nes.ppu.bg_ptable_lsb_sr |= nes.ppu.bg_ptable_lsb_tmp as u16;
        nes.ppu.bg_ptable_msb_sr |= nes.ppu.bg_ptable_msb_tmp as u16;

        let left_num = ((nes.ppu.v & COARSE_X).wrapping_sub(1)) / 2;
        let top_num = ((nes.ppu.v & COARSE_Y) >> 5) / 2;

        let is_top = top_num % 2 == 0;
        let is_left = left_num % 2 == 0;

        let attr = nes.ppu.bg_atable_tmp;

        match (is_top, is_left) {
            (true, true) => {
                nes.ppu.bg_attr_lsb_latch = get_bit(attr, 0);
                nes.ppu.bg_attr_msb_latch = get_bit(attr, 1);
            }
            (true, false) => {
                nes.ppu.bg_attr_lsb_latch = get_bit(attr, 2);
                nes.ppu.bg_attr_msb_latch = get_bit(attr, 3);
            }
            (false, true) => {
                nes.ppu.bg_attr_lsb_latch = get_bit(attr, 4);
                nes.ppu.bg_attr_msb_latch = get_bit(attr, 5);
            }
            (false, false) => {
                nes.ppu.bg_attr_lsb_latch = get_bit(attr, 6);
                nes.ppu.bg_attr_msb_latch = get_bit(attr, 7);
            }
        }
    }

    let in_bg_fetch_cycle =
        ((1..=256).contains(&cycle) || (321..=336).contains(&cycle)) && (scanline <= 239);

    if in_bg_fetch_cycle && rendering_enabled {
        // Fetching happens in 8-cycle cycles
        match cycle % 8 {
            NAMETABLE_READ => {
                let ntable_address = 0x2000 | (nes.ppu.v & !FINE_Y);
                nes.ppu.bg_ntable_tmp = read_vram(ntable_address, nes);
            }

            ATTRIBUTE_READ => {
                let attribute_addr = 0x23C0
                    | (nes.ppu.v & NAMETABLE)
                    | ((nes.ppu.v & 0b11100_00000) >> 4)
                    | ((nes.ppu.v & 0b00000_11100) >> 2);
                nes.ppu.bg_atable_tmp = read_vram(attribute_addr, nes);
            }

            PATTERN_LSB_READ => {
                let tile_addr = ((nes.ppu.bg_ptable_select as u16) << 12)
                    | ((nes.ppu.bg_ntable_tmp as u16) << 4)
                    | ((nes.ppu.v & FINE_Y) >> 12);
                nes.ppu.bg_ptable_lsb_tmp = read_vram(tile_addr, nes);
            }

            PATTERN_MSB_READ => {
                let tile_addr = (((nes.ppu.bg_ptable_select as u16) << 12)
                    | ((nes.ppu.bg_ntable_tmp as u16) << 4)
                    | ((nes.ppu.v & FINE_Y) >> 12))
                    + 8; // equivalently | 0b1000, lower 3 are fine y
                nes.ppu.bg_ptable_msb_tmp = read_vram(tile_addr, nes);
            }

            _ => (),
        }
    }

    // sprite fetch spans 64 cycles
    // each sprite does 4 memory reads, each taking 2 cycles
    let in_sprite_fetch_cycle = (257..=320).contains(&cycle) && (scanline <= 239);

    if in_sprite_fetch_cycle && rendering_enabled {
        let current_sprite = (cycle as usize - 257) / 8;

        if nes.ppu.sprite_zero_in_soam && scanline > 0 {
            nes.ppu.sprite_zero_in_soam = false;
            nes.ppu.sprite_zero_in_latches = true;
        }

        let sprite_y = nes.ppu.s_oam[current_sprite * 4];
        let mut sprite_tile_index = nes.ppu.s_oam[current_sprite * 4 + 1];
        let sprite_properties = nes.ppu.s_oam[current_sprite * 4 + 2];
        let sprite_x = nes.ppu.s_oam[current_sprite * 4 + 3];

        // don't know when this actually happens, don't think it really matters
        nes.ppu.sprite_property_latches[current_sprite] = sprite_properties;
        nes.ppu.sprite_x_counters[current_sprite] = sprite_x;

        let ptable_select = if nes.ppu.tall_sprites {
            let ptable_bit = (sprite_tile_index & 1) == 1;
            sprite_tile_index &= 0b11111110;
            ptable_bit
        } else {
            nes.ppu.sprite_ptable_select
        };

        /*

        I think we need a match for calculating the tile addr
        there's already duplication in the match below with a bunch of conditions and shit


        */

        let vertically_flipped = (sprite_properties >> 7) == 1;

        // TODO: On the pre-render scanline, we should just reuse what was fetched on the last scanline
        let tile_y = (nes.ppu.scanline - (sprite_y as u32 as i32)).max(0);

        let offset = match (vertically_flipped, nes.ppu.tall_sprites) {
            (false, false) => tile_y,
            (false, true) => tile_y + if tile_y >= 8 { 8 } else { 0 },
            (true, false) => 7 - tile_y,
            (true, true) => {
                // if in top tile, invert, then move to bottom tile
                if tile_y < 8 {
                    (7 - tile_y) + 16
                }
                // if in bottom tile, move back to top tile, then invert
                else {
                    7 - (tile_y - 8)
                }
            }
        };

        let tile_addr = ((ptable_select as u16) << 12)
            | ((sprite_tile_index as u16) << 4).wrapping_add(offset as u16);

        let flip_horizontally = (sprite_properties & 0b01000000) > 0;

        match cycle % 8 {
            NAMETABLE_READ => { /* garbage nametable read */ }
            ATTRIBUTE_READ => { /* garbage attribute read */ }

            PATTERN_LSB_READ => {
                let mut data = read_vram(tile_addr, nes);
                if current_sprite >= (nes.ppu.in_range_counter as usize) {
                    data = 0;
                }
                if flip_horizontally {
                    data = data.reverse_bits();
                }
                nes.ppu.sprite_ptable_lsb_srs[current_sprite] = data;
            }

            PATTERN_MSB_READ => {
                let mut data = read_vram(tile_addr.wrapping_add(8), nes);
                if current_sprite >= (nes.ppu.in_range_counter as usize) {
                    data = 0;
                }
                if flip_horizontally {
                    data = data.reverse_bits();
                }
                nes.ppu.sprite_ptable_msb_srs[current_sprite] = data;
            }

            _ => (),
        }
    }

    // https://fceux.com/web/help/PPU.html
    // could tidy up these conditions later
    // Horizontal v increment happens on cycles {8, 16, 24, ... , 256} and {328, 336}
    let horizontal_v_increment = (((cycle % 8 == 0) && (8..=256).contains(&cycle))
        || (cycle == 328 || cycle == 336))
        && scanline <= 239;

    if horizontal_v_increment && rendering_enabled {
        inc_v_horizontal(nes);
    }

    if cycle == 256 && scanline <= 239 && rendering_enabled {
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
    }

    if (280..=304).contains(&cycle) && scanline == -1 && rendering_enabled {
        const VERTICAL_BITMASK: u16 = FINE_Y | NAMETABLE_MSB | COARSE_Y;

        nes.ppu.v &= !VERTICAL_BITMASK;
        nes.ppu.v |= nes.ppu.t & VERTICAL_BITMASK;
    }

    // Skip last pre-render cycle on odd frames
    if nes.ppu.odd_frame
        && nes.ppu.scanline_cycle == 339
        && nes.ppu.scanline == -1
        && rendering_enabled
    {
        nes.ppu.scanline_cycle = 0;
        nes.ppu.scanline = 0;
        nes.ppu.odd_frame = !nes.ppu.odd_frame;
    }
    // Wrap scanline cycles
    else if nes.ppu.scanline_cycle < 340 {
        nes.ppu.scanline_cycle += 1;
    } else {
        nes.ppu.scanline_cycle = 0;
        // Wrap scanline
        if nes.ppu.scanline < 260 {
            nes.ppu.scanline += 1;
        } else {
            // Pre-render scanline is -1 instead of 261 for convenience
            nes.ppu.scanline = -1;
            nes.ppu.odd_frame = !nes.ppu.odd_frame;
        }
    }
    nes.ppu.cycles += 1;
}

fn inc_v_horizontal(nes: &mut Nes) {
    if (nes.ppu.v & COARSE_X) == 31 {
        nes.ppu.v &= !COARSE_X; // sets coarse x to 0
        nes.ppu.v ^= NAMETABLE_LSB; // toggles lower nametable bit
    } else {
        nes.ppu.v += 1;
    }
}

fn inc_v_vertical(nes: &mut Nes) {
    if ((nes.ppu.v & FINE_Y) >> 12) == 7 {
        nes.ppu.v &= !FINE_Y;
        if ((nes.ppu.v & COARSE_Y) >> 5) == 29 {
            nes.ppu.v &= !COARSE_Y;
            nes.ppu.v ^= NAMETABLE_MSB;
        } else if ((nes.ppu.v & COARSE_Y) >> 5) == 31 {
            nes.ppu.v &= !COARSE_Y;
        } else {
            nes.ppu.v += 0b000_00_00001_00000;
        }
    } else {
        nes.ppu.v += 0b001_00_00000_00000;
    }
}
