struct Cartridge {
    prg_rom: Vec<u8>,
    chr_rom: Vec<u8>,
    mapper: u8,
}

struct Cpu {
    a: u8,
    x: u8,
    y: u8,
    s: u8,
    p_n: bool,
    p_v: bool,
    p_b: bool,
    p_i: bool,
    p_z: bool,
    p_c: bool,
    pc: u16,
    cycles: u64,
}