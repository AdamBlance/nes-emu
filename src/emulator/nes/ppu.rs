mod consts;
mod control;
mod mem;
mod step;

pub use self::mem::{
    get_dynamic_latch, increment_v_after_ppudata_access, memory_mapped_register_read,
    memory_mapped_register_write, read_vram, set_dynamic_latch, write_vram,
};
pub use self::step::step_ppu;
use crate::emulator::nes::util::U3;
use crate::util::get_bit;
use modular_bitfield::prelude::*;
use modular_bitfield::{Specifier, bitfield};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Specifier)]
enum VIncrement {
    Inc1,
    Inc32,
}

#[bitfield]
#[derive(Clone)]
struct PpuCtrl {
    nmi_enable: bool,
    master_slave: bool,
    tall_sprites: bool,
    bg_ptable: B1,
    sprite_ptable: B1,
    #[bits = 2]
    increment: VIncrement,
    ntable: B2,
}

#[bitfield]
#[derive(Clone)]
struct PpuMask {
    pub blue_emphasis: bool,
    pub green_emphasis: bool,
    pub red_emphasis: bool,
    pub show_sprites: bool,
    pub show_bg: bool,
    pub show_leftmost_sprites: bool,
    pub show_leftmost_bg: bool,
    pub greyscale: bool,
}

#[bitfield]
struct PpuAddress {
    _padding: B1,
    fine_y_scroll: B3,
    nametable: B2,
    coarse_y_scroll: B5,
    coarse_x_scroll: B5,
}

#[bitfield]
#[derive(Clone)]
struct PpuStatus {
    in_vblank: bool,
    sprite_zero_hit: bool,
    sprite_overflow: bool,
    _padding: B5,
}

#[derive(Clone)]
struct Memory {
    pub vram: [u8; 2048],
    pub palette_mem: [u8; 32],
    pub oam: [u8; 256],
    pub secondary_oam: [u8; 32],
    pub oam_addr: u8,
}

#[derive(Clone)]
struct Scroll {
    pub t: PpuAddress,
    pub v: PpuAddress,
    pub x: U3,
    pub w: bool,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Ppu {
    ppu_ctrl: PpuCtrl,
    ppu_mask: PpuMask,
    ppu_status: PpuStatus,
    memory: Memory,
    scroll: Scroll,
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

    pub dynamic_latch: u8,
    pub dynamic_latch_last_set_cycle: u64,
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

            dynamic_latch: 0,
            dynamic_latch_last_set_cycle: 0,
        }
    }

    pub fn in_vblank_final_cycles(&self) -> bool {
        self.scanline == 239 && (257..=259).contains(&self.scanline_cycle)
    }
}
