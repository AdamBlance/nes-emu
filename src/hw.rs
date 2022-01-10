pub struct Cartridge {
    pub prg_rom: Vec<u8>,
    pub chr_rom: Vec<u8>,
    pub mapper: u8,
    pub v_mirroring: bool,
}

pub struct Ppu {
    // memory mapped registers
    pub ppuctrl: u8,
    pub ppumask: u8,
    pub ppustatus: u8,
    pub oamaddr: u8,
    pub oamdata: u8,
    pub ppuscroll: u8,
    pub ppuaddr: u8,
    pub ppudata: u8,
    pub oamdma: u8,

    pub t: u16,
    pub v: u16,
    pub x: u8,
    pub w: bool,
    
    pub palette_mem: [u8; 32],
    pub oam: [u8; 256],

    pub odd_frame: bool,
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