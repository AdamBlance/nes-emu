use crate::util::*;
use crate::hw::*;
use crate::ppu;

pub fn read_mem_u16(addr: u16, nes: &mut Nes) -> u16 {
    let lsb = read_mem(addr, nes);
    let msb = read_mem(addr.wrapping_add(1), nes);
    concat_u8(msb, lsb)
}

pub fn read_mem_u16_zp(addr: u16, nes: &mut Nes) -> u16 {
    let lsb = read_mem(addr, nes);
    let msb = read_mem((addr.wrapping_add(1) % 256), nes);
    concat_u8(msb, lsb)
}

const PPUCTRL: u16   = 0x2000;
const PPUMASK: u16   = 0x2001;
const PPUSTATUS: u16 = 0x2002;
const OAMADDR: u16   = 0x2003;
const OAMDATA: u16   = 0x2004;
const PPUSCROLL: u16 = 0x2005;
const PPUADDR: u16   = 0x2006;
const PPUDATA: u16   = 0x2007;
const OAMDMA: u16    = 0x4014;

const PPU_WARMUP: u64 = 29658;

fn mapper(addr: u16, nes: &mut Nes) -> u8 {
    match addr {
        0xC000..=0xFFFF => nes.cart.prg_rom[(addr - 0xC000) as usize],
        _ => 0,
    }
}

pub fn read_mem(addr: u16, nes: &mut Nes) -> u8 {
    match addr {
        // Main memory, mirrored 4 times
        0x0000..=0x1FFF => nes.wram[(addr % 0x800) as usize],

        // PPU memory mapped registers are mirrored through this range
        // Could use guards here, might perform worse but would be more readable? 
        0x2000..=0x3FFF => 
            match 0x2000 + (addr % 8) {
                // These registers are write only
                // Should technically return the value in the weird capacitive latch 
                PPUCTRL   => 0,
                PPUMASK   => 0,
                OAMADDR   => 0,
                PPUSCROLL => 0,
                PPUADDR   => 0,


                PPUSTATUS => {
                    let status = ppustatus_to_byte(nes);
                    // Write toggle used by PPUADDR and PPUSCROLL gets reset when PPUSTATUS is read
                    nes.ppu.w = false;
                    status
                },
                // Reads during rendering should "expose internal OAM accesses..."
                // Apparently one games uses this
                OAMDATA => nes.ppu.oam_addr,
                PPUDATA => {
                    let val = ppu::read_vram(nes.ppu.v, nes);
                    // Should really use enums or something, this is hard to read
                    let increment = if nes.ppu.ppuctrl_vram_address_increment_select == false {1} else {32};
                    nes.ppu.v = nes.ppu.v.wrapping_add(increment);
                    val
                },
                
                _ => panic!("Literally impossible"),
            },

        OAMDMA => 0,

        0x4000..=0x4017 => 0,
        0x4018..=0x401F => 0,
        0x4020..=0xFFFF => mapper(addr, nes),
    }
}

// Needs to set ppu_written_to when affecting registers
pub fn write_mem(addr: u16, val: u8, nes: &mut Nes) {
    let val_u16 = val as u16;
    match addr {
        // Write to normal RAM
        0x0000..=0x1FFF => nes.wram[(addr % 0x800) as usize] = val,

        // PPU memory mapped registers
        0x2000..=0x3FFF =>
            match 0x2000 + (addr % 8) {
                PPUCTRL   => {
                    if nes.cpu.cycles < PPU_WARMUP {return};
                    byte_to_ppuctrl(val, nes);
                    nes.ppu.t &= 0b1_111001111111111;
                    nes.ppu.t |= (val_u16 & 0b11) << 10;
                },
                PPUMASK   => {
                    if nes.cpu.cycles < PPU_WARMUP {return};
                    byte_to_ppumask(val, nes);
                }
                OAMADDR   => nes.ppu.oam_addr = val,
                OAMDATA   => {
                    nes.ppu.oam[nes.ppu.oam_addr as usize] = val;
                    nes.ppu.oam_addr = nes.ppu.oam_addr.wrapping_add(1);
                },
                PPUSCROLL => {
                    if nes.cpu.cycles < PPU_WARMUP {return};

                    if !nes.ppu.w {
                        nes.ppu.t &= 0b1_111_11_11111_00000;
                        nes.ppu.t |= (val as u16) >> 3;
                        nes.ppu.x = val & 0b0000_0111;
                    } else {
                        nes.ppu.t &= 0b1_000_11_00000_11111;
                        nes.ppu.t |= (val_u16 & 0b11111_000) << 2;
                        nes.ppu.t |= (val_u16 & 0b00000_111) << 12;
                    }
                    nes.ppu.w = !nes.ppu.w;
                },
                PPUADDR   => {
                    if nes.cpu.cycles < PPU_WARMUP {return};

                    if !nes.ppu.w {
                        nes.ppu.t &= 0b1_0_00000_11111111;
                        nes.ppu.t |= (val_u16 & 0b00011111) << 8;
                    } else {
                        nes.ppu.t &= 0b1_1_11111_00000000;
                        nes.ppu.t |= val_u16;
                        nes.ppu.v = nes.ppu.t;
                    }
                    nes.ppu.w = !nes.ppu.w;
                },
                PPUSTATUS => {},
                PPUDATA   => {
                    ppu::write_vram(nes.ppu.v, val, nes);
                    // Should really use enums or something, this is hard to read
                    let increment = if nes.ppu.ppuctrl_vram_address_increment_select == false {1} else {32};
                    nes.ppu.v = nes.ppu.v.wrapping_add(increment);
                },
                _ => panic!("Literally impossible"),
            },

        OAMDMA    => {
            let base = (val_u16 << 8);
            for offset in 0x00..0xFF {
                nes.ppu.oam[offset] = read_mem(base + offset as u16, nes);
            }
        },
        _ => (),
    };
}

fn byte_to_ppuctrl(byte: u8, nes: &mut Nes) {
    nes.ppu.ppuctrl_nmi_enable = get_bit(byte, 7);
    nes.ppu.ppuctrl_master_slave = get_bit(byte, 6);
    nes.ppu.ppuctrl_tall_sprites = get_bit(byte, 5);
    nes.ppu.ppuctrl_background_pattern_table_select = get_bit(byte, 4);
    nes.ppu.ppuctrl_sprite_pattern_table_select = get_bit(byte, 3);
    nes.ppu.ppuctrl_vram_address_increment_select = get_bit(byte, 2);
    nes.ppu.ppuctrl_nametable_select = byte & 0b0000_0011;
}



fn byte_to_ppumask(byte: u8, nes: &mut Nes) {
    nes.ppu.blue_emphasis = get_bit(byte, 7);
    nes.ppu.green_emphasis = get_bit(byte, 6);
    nes.ppu.red_emphasis = get_bit(byte, 5);
    nes.ppu.sprite_enable = get_bit(byte, 4);
    nes.ppu.background_enable = get_bit(byte, 3);
    nes.ppu.sprite_left_column_enable = get_bit(byte, 2);
    nes.ppu.background_left_column_enable = get_bit(byte, 1);
    nes.ppu.greyscale = get_bit(byte, 0);
}
fn ppustatus_to_byte(nes: &Nes) -> u8 {
    (if nes.ppu.vblank          {0b1000_0000} else {0}) | 
    (if nes.ppu.sprite_zero_hit {0b0100_0000} else {0}) | 
    (if nes.ppu.sprite_overflow {0b0000_1000} else {0})
}
fn byte_to_ppustatus(byte: u8, nes: &mut Nes) {
    nes.ppu.vblank = get_bit(byte, 7);
    nes.ppu.sprite_zero_hit = get_bit(byte, 6);
    nes.ppu.sprite_overflow = get_bit(byte, 5);
}
