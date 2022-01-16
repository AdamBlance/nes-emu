pub struct Nes {
    pub cpu: Cpu,
    pub wram: [u8; 2048],
    pub ppu: Ppu,
    pub cart: Cartridge,
    pub ppu_written_to: bool,
    pub frame: Vec<u8>
}

pub struct Cartridge {
    pub prg_rom: Vec<u8>,
    pub chr_rom: Vec<u8>,
    pub mapper: u8,
    pub v_mirroring: bool,
}

pub struct Ppu {
    pub nmi_enable: bool,
    pub master_slave: bool,
    pub sprite_height: bool,
    pub background_tile_select: bool,
    pub sprite_tile_select: bool,
    pub increment_mode: bool,
    pub nametable_select: u8,

    pub blue_emphasis: bool,
    pub green_emphasis: bool,
    pub red_emphasis: bool,
    pub sprite_enable: bool,
    pub background_enable: bool,
    pub sprite_left_column_enable: bool,
    pub background_left_column_enable: bool,
    pub greyscale: bool,

    pub vblank: bool,
    pub sprite_zero_hit: bool,
    pub sprite_overflow: bool,

    pub oam_addr: u8,
    pub ppu_scroll: u8,
    pub ppu_addr: u8,
    pub ppu_data: u8,
    pub oam_data: u8,
    pub oam_dma: u8,
    pub palette_mem: [u8; 32],
    pub oam: [u8; 256],
    pub vram: [u8; 2048],
    pub scanline: u16,
    pub pixel: u16,
    pub t: u16,
    pub v: u16,
    pub x: u8,
    pub w: bool,
    pub odd_frame: bool,
    pub cycles: u64,

    pub internal_latch: u8,
    pub nmi: bool,
}

impl Default for Ppu {
    fn default() -> Ppu {
        Ppu {
            nmi_enable: false,
            master_slave: false,
            sprite_height: false,
            background_tile_select: false,
            sprite_tile_select: false,
            increment_mode: false,
            nametable_select: 0,
        
            blue_emphasis: false,
            green_emphasis: false,
            red_emphasis: false,
            sprite_enable: false,
            background_enable: false,
            sprite_left_column_enable: false,
            background_left_column_enable: false,
            greyscale: false,
        
            vblank: false,
            sprite_zero_hit: false,
            sprite_overflow: false,
            oam_addr: 0,
            ppu_scroll: 0,
            ppu_addr: 0,
            ppu_data: 0,
            oam_data: 0,
            oam_dma: 0,
            palette_mem: [0; 32],
            oam: [0; 256],
            vram: [0; 2048],
            scanline: 0,
            pixel: 0,
            t: 0,
            v: 0,
            x: 0,
            w: false,
            odd_frame: false,
            cycles: 0,

            internal_latch: 0,
            nmi: false,
            
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
    pub cycles: u64,
    pub counter: u64,
}
