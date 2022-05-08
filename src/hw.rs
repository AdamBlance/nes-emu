pub struct Nes {
    pub cpu: Cpu,
    pub wram: [u8; 2048],
    pub ppu: Ppu,
    pub cart: Cartridge,
    pub frame: Vec<u8>,
    pub skip: u64,
}

pub struct Cartridge {
    pub prg_rom: Vec<u8>,
    pub chr_rom: Vec<u8>,
    pub mapper: u8,
    pub v_mirroring: bool,
}

// feel like I need some good comments here
pub struct Ppu {
    // PPUCTRL
    pub nmi_enable: bool,
    pub master_slave: bool,
    pub tall_sprites: bool,
    pub bg_ptable_select: bool,
    pub sprite_ptable_select: bool,
    pub increment_select: bool,
    pub ntable_select: u8, 

    // PPUMASK
    pub blue_emphasis: bool,
    pub green_emphasis: bool,
    pub red_emphasis: bool,
    pub show_sprites: bool,
    pub show_bg: bool,
    pub show_leftmost_sprites: bool,
    pub show_leftmost_bg: bool,
    pub greyscale: bool,

    // PPUSTATUS
    pub in_vblank: bool,
    pub sprite_zero_hit: bool,
    pub sprite_overflow: bool,

    // OAMADDR
    pub oam_addr: u8,

    // Memories
    pub vram:        [u8; 2048],
    pub oam:         [u8; 256],
    pub palette_mem: [u8; 32],

    // v/t PPU addresses
    pub t: u16,
    pub v: u16,
    
    // Fine X
    pub x: u8,

    // Write toggle
    pub w: bool,

    // Rendering counters
    pub scanline: u32,
    pub scanline_cycle: u32,
    pub odd_frame: bool,

    // Internal latches for just-read values
    pub ntable_tmp: u8,
    pub attr_tmp: u8,
    pub ptable_lsb_tmp: u8,
    pub ptable_msb_tmp: u8,
    
    // Shift registers
    pub ptable_lsb_sr: u16,
    pub ptable_msb_sr: u16,
    pub attr_lsb_sr: u8,
    pub attr_msb_sr: u8,
    
    // 1-bit attribute latches
    pub attr_lsb_latch: bool,
    pub attr_msb_latch: bool,
    
    pub cycles: u64,
}

impl Default for Ppu {
    fn default() -> Ppu {
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

            vram: [0; 2048],
            oam: [0; 256],
            palette_mem: [0; 32],

            t: 0,
            v: 0,
            x: 0,
            w: false,

            scanline: 0,
            scanline_cycle: 0,
            odd_frame: false,

            ntable_tmp: 0,
            attr_tmp: 0,
            ptable_lsb_tmp: 0,
            ptable_msb_tmp: 0,
            
            ptable_lsb_sr: 0,
            ptable_msb_sr: 0,
            attr_lsb_sr: 0,
            attr_msb_sr: 0,
    
            attr_lsb_latch: false,
            attr_msb_latch: false,
    
            cycles: 0,
        }
    }
}

#[derive(Default)]
pub struct Cpu {
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub s: u8,
    pub p_n: bool,
    pub p_v: bool,
    pub p_d: bool,
    pub p_i: bool,
    pub p_z: bool,
    pub p_c: bool,
    pub pc: u16,
    pub nmi_interrupt: bool,
    pub nmi_internal_flag: bool,
    pub cycles: u64,
    pub instruction_count: u64,
}
