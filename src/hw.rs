pub struct Nes {
    pub cpu: Cpu,
    pub wram: [u8; 2048],
    pub ppu: Ppu,
    pub cart: Cartridge,
    pub ppu_written_to: bool,
    pub frame: [u8; 256*240*4]
}

pub struct Cartridge {
    pub prg_rom: Vec<u8>,
    pub chr_rom: Vec<u8>,
    pub mapper: u8,
    pub v_mirroring: bool,
}


pub struct Ppu {
    pub ppu_ctrl: PpuCtrl,
    pub ppu_mask: PpuMask,
    pub ppu_status: PpuStatus,
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
}

impl Default for Ppu {
    fn default() -> Ppu {
        Ppu {
            ppu_ctrl: Default::default(),
            ppu_mask: Default::default(),
            ppu_status: Default::default(),
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
    pub cycles: u64,
}

bitflags! {
    #[derive(Default)]
    pub struct CpuP: u8 {
        const N = 0b1000_0000;
        const V = 0b0100_0000;
        const D = 0b0000_1000;
        const I = 0b0000_0100;
        const Z = 0b0000_0010;
        const C = 0b0000_0001;
    }
    #[derive(Default)]
    pub struct PpuCtrl: u8 {
        const NMI_ENABLE             = 0b1000_0000;
        const MASTER_SLAVE           = 0b0100_0000;
        const SPRITE_HEIGHT          = 0b0010_0000;
        const BACKGROUND_TILE_SELECT = 0b0001_0000;
        const SPRITE_TILE_SELECT     = 0b0000_1000;
        const INCREMENT_MODE         = 0b0000_0100;
        const NAMETABLE_SELECT       = 0b0000_0011;
    }
    #[derive(Default)]
    pub struct PpuMask: u8 {
        const BLUE_EMPHASIS      = 0b1000_0000;
        const GREEN_EMPHASIS     = 0b0100_0000;
        const RED_EMPHASIS       = 0b0010_0000;
        const SPRITE_ENABLE      = 0b0001_0000;
        const BACKGROUND_ENABLE  = 0b0000_1000;
        const SPRITE_LEFT_COLUMN_ENABLE  = 0b0000_0100;
        const BACKGROUND_LEFT_COLUMN_ENABLE  = 0b0000_0100;
        const GREYSCALE  = 0b0000_0100;
    }
    #[derive(Default)]
    pub struct PpuStatus: u8 {
        const VBLANK      = 0b1000_0000;
        const SPRITE_ZERO_HIT     = 0b0100_0000;
        const SPRITE_OVERFLOW       = 0b0010_0000;
    }
}