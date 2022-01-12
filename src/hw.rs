pub struct Nes {
    pub cpu: Cpu,
    pub wram: [u8; 2048],
    pub ppu: Ppu,
    pub cart: Cartridge,
}

pub struct Cartridge {
    pub prg_rom: Vec<u8>,
    pub chr_rom: Vec<u8>,
    pub mapper: u8,
    pub v_mirroring: bool,
}

pub struct Ppu {
    pub palette_mem: [u8; 32],
    pub oam: [u8; 256],
    pub vram: [u8; 2048],

    pub odd_frame: bool,

    pub ppu_ctrl: u8,
    pub ppu_mask: u8,
    pub ppu_status: u8,
    pub ppu_scroll: u8,
    pub ppu_addr: u8,
    pub ppu_data: u8,
    pub oam_addr: u8,
    pub oam_data: u8,
    pub oam_dma: u8,

    pub t: u16,
    pub v: u16,
    pub x: u8,
    pub w: bool,
    
    pub cycles: u64,
}

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
