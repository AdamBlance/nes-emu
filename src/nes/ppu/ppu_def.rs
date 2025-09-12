use serde::{Deserialize, Serialize};
use std::fmt;

use crate::util::get_bit;

#[derive(Clone, Serialize, Deserialize)]
pub struct Ppu {
    // PPUCTRL register
    pub nmi_enable: bool,
    pub master_slave: bool,
    pub tall_sprites: bool,
    pub bg_ptable_select: bool,
    pub sprite_ptable_select: bool,
    pub increment_select: bool,
    pub ntable_select: u8,
    // PPUMASK register
    pub blue_emphasis: bool,
    pub green_emphasis: bool,
    pub red_emphasis: bool,
    pub show_sprites: bool,
    pub show_bg: bool,
    pub show_leftmost_sprites: bool,
    pub show_leftmost_bg: bool,
    pub greyscale: bool,
    // PPUSTATUS register
    pub in_vblank: bool,
    pub sprite_zero_hit: bool,
    pub sprite_overflow: bool,
    // OAMADDR register
    pub oam_addr: u8,
    // Memories
    // TODO: Make Rc
    pub vram: Vec<u8>,
    pub oam: Vec<u8>,
    pub s_oam: [u8; 32],
    pub palette_mem: [u8; 32],
    // Rendering counters/indices/flags
    pub t: u16,
    pub v: u16,
    pub x: u8,
    pub w: bool,
    pub scanline: i32,
    pub scanline_cycle: i32,
    pub odd_frame: bool,
    // Temporary background latches
    pub bg_ntable_tmp: u8,
    pub bg_atable_tmp: u8,
    pub bg_ptable_lsb_tmp: u8,
    pub bg_ptable_msb_tmp: u8,
    // Background shift registers / latches
    pub bg_ptable_lsb_sr: u16,
    pub bg_ptable_msb_sr: u16,
    pub bg_attr_lsb_sr: u8,
    pub bg_attr_msb_sr: u8,
    pub bg_attr_lsb_latch: bool,
    pub bg_attr_msb_latch: bool,
    // Sprite shift registers / latches
    pub sprite_ptable_lsb_srs: [u8; 8],
    pub sprite_ptable_msb_srs: [u8; 8],
    pub sprite_property_latches: [u8; 8],
    pub sprite_x_counters: [u8; 8],
    // Sprite/OAM evaluation
    pub in_range_counter: u8,
    pub sprite_zero_in_soam: bool,
    pub sprite_zero_in_latches: bool,
    // Misc
    pub nmi_line: bool,
    pub ppudata_buffer: u8,
    pub cycles: u64,
    pub addr_bus: u16,

    pub dynamic_latch: u8,
    pub dynamic_latch_last_set_cycle: u64
}

impl fmt::Debug for Ppu {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        f.debug_struct("Ppu")
            .field("sprite_zero_hit", &self.sprite_zero_hit)
            .finish()
    }
}

impl Default for Ppu {
    fn default() -> Self {
        Self::new()
    }
}

impl Ppu {
    pub fn new() -> Ppu {
        Ppu {
            nmi_enable: false,
            master_slave: false,
            tall_sprites: false,
            bg_ptable_select: false,
            sprite_ptable_select: false,
            increment_select: false,
            ntable_select: 0,

            blue_emphasis: false,
            green_emphasis: false,
            red_emphasis: false,
            show_sprites: false,
            show_bg: false,
            show_leftmost_sprites: false,
            show_leftmost_bg: false,
            greyscale: false,

            in_vblank: false,
            sprite_zero_hit: false,
            sprite_overflow: false,

            oam_addr: 0,

            vram: vec![0; 2048],
            oam: vec![0; 256],
            s_oam: [0; 32],
            palette_mem: [0; 32],

            t: 0,
            v: 0,
            x: 0,
            w: false,
            scanline: 0,
            scanline_cycle: 27,
            odd_frame: false,

            bg_ntable_tmp: 0,
            bg_atable_tmp: 0,
            bg_ptable_lsb_tmp: 0,
            bg_ptable_msb_tmp: 0,

            bg_ptable_lsb_sr: 0,
            bg_ptable_msb_sr: 0,
            bg_attr_lsb_sr: 0,
            bg_attr_msb_sr: 0,
            bg_attr_lsb_latch: false,
            bg_attr_msb_latch: false,

            sprite_ptable_lsb_srs: [0; 8],
            sprite_ptable_msb_srs: [0; 8],
            sprite_property_latches: [0; 8],
            sprite_x_counters: [0; 8],

            in_range_counter: 0,
            sprite_zero_in_soam: false,
            sprite_zero_in_latches: false,

            nmi_line: false,
            ppudata_buffer: 0,
            cycles: 0,
            addr_bus: 0,

            dynamic_latch: 0,
            dynamic_latch_last_set_cycle: 0
        }
    }

    pub fn set_ppuctrl_from_byte(&mut self, byte: u8) {
        self.nmi_enable = get_bit(byte, 7);
        self.master_slave = get_bit(byte, 6);
        self.tall_sprites = get_bit(byte, 5);
        self.bg_ptable_select = get_bit(byte, 4);
        self.sprite_ptable_select = get_bit(byte, 3);
        self.increment_select = get_bit(byte, 2);
        self.ntable_select = byte & 0b0000_0011;
    }

    pub fn set_ppumask_from_byte(&mut self, byte: u8) {
        self.blue_emphasis = get_bit(byte, 7);
        self.green_emphasis = get_bit(byte, 6);
        self.red_emphasis = get_bit(byte, 5);
        self.show_sprites = get_bit(byte, 4);
        self.show_bg = get_bit(byte, 3);
        self.show_leftmost_sprites = get_bit(byte, 2);
        self.show_leftmost_bg = get_bit(byte, 1);
        self.greyscale = get_bit(byte, 0);
    }

    pub fn get_ppustatus_byte(&self) -> u8 {
        (self.in_vblank as u8) << 7
            | (self.sprite_zero_hit as u8) << 6
            | (self.sprite_overflow as u8) << 5
    }
}
