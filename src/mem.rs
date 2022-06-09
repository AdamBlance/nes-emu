use crate::util::*;
use crate::hw::*;
use crate::ppu;

const PPUCTRL: u16   = 0x2000;
const PPUMASK: u16   = 0x2001;
const PPUSTATUS: u16 = 0x2002;
const OAMADDR: u16   = 0x2003;
const OAMDATA: u16   = 0x2004;
const PPUSCROLL: u16 = 0x2005;
const PPUADDR: u16   = 0x2006;
const PPUDATA: u16   = 0x2007;

const PULSE_1_REG_1: u16 = 0x4000;
const PULSE_1_REG_2: u16 = 0x4001;
const PULSE_1_REG_3: u16 = 0x4002;
const PULSE_1_REG_4: u16 = 0x4003;

const PULSE_2_REG_1: u16 = 0x4004;
const PULSE_2_REG_2: u16 = 0x4005;
const PULSE_2_REG_3: u16 = 0x4006;
const PULSE_2_REG_4: u16 = 0x4007;

const TRIANGLE_REG_1: u16 = 0x4008;
const TRIANGLE_REG_2: u16 = 0x400A;
const TRIANGLE_REG_3: u16 = 0x400B;

const APU_STATUS_REG: u16 = 0x4015;

const OAMDMA: u16    = 0x4014;

const CONTROLLER_1: u16 = 0x4016;
const CONTROLLER_2: u16 = 0x4017;


const PPU_WARMUP: u64 = 29658;

fn mapper(addr: u16, nes: &mut Nes) -> u8 {
    // this is just nrom 
    let prg_size = nes.cartridge.prg_rom.len() as u16;
    nes.cartridge.prg_rom[(addr % prg_size) as usize]
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
                    nes.ppu.in_vblank = false;
                    nes.ppu.w = false;
                    status
                },
                // Reads during rendering should "expose internal OAM accesses..."
                // Apparently one games uses this
                OAMDATA => nes.ppu.oam_addr,
                PPUDATA => {
                    /*
                    Just going to be as explicit as possible here

                    When the CPU reads VRAM through PPUDATA, it pulls the value from an internal 
                    PPUDATA buffer.
                    Then, immediately after, the PPU fills this buffer with the data you were trying
                    to read. 

                    This means the data read through PPUDATA is delayed by one byte. 
                    Therefore, the buffer has to be primed the first time you read it to put a value
                    in there.

                    This behaviour changes when reading any palette data 0x3F00..=0x3FFF.
                    In this case, the data is placed directly on the CPU data bus.
                    The internal PPUDATA buffer is then filled with addr-0x1000, who knows why

                    https://archive.nes.science/nesdev-forums/f3/t18627.xhtml
                    https://www.nesdev.org/wiki/PPU_registers#The_PPUDATA_read_buffer_(post-fetch)


                    */

                    if addr < 0x3F00 {
                        let data_in_memory = ppu::read_vram(nes.ppu.v, nes);
                        let prev_data_in_buffer = nes.ppu.ppudata_buffer;
                        nes.ppu.ppudata_buffer = data_in_memory;

                        // increment v
                        let increment = if nes.ppu.increment_select == false {1} else {32};
                        nes.ppu.v = nes.ppu.v.wrapping_add(increment);

                        prev_data_in_buffer
                    } else {

                        let data_in_memory = ppu::read_vram(nes.ppu.v, nes);
                        nes.ppu.ppudata_buffer = ppu::read_vram(nes.ppu.v.wrapping_sub(0x1000), nes);

                        // increment v
                        let increment = if nes.ppu.increment_select == false {1} else {32};
                        nes.ppu.v = nes.ppu.v.wrapping_add(increment);

                        data_in_memory
                    }

                },
                
                _ => panic!("Literally impossible"),
            },

        OAMDMA => 0,

        CONTROLLER_1 => {
            let button_bit = nes.controller1.shift_register & 1;
            nes.controller1.shift_register >>= 1;
            button_bit
        }

        0x4000..=0x4017 => 0,
        0x4018..=0x401F => 0,
        0x8000..=0xFFFF => mapper(addr, nes),
        _ => 0,
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
                    nes.ppu.t &= 0b111_00_11111_11111;
                    // put nametable bits from ppuctrl into t
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
                        nes.ppu.t &= 0b111_11_11111_00000;
                        // put x scroll co-ord in fine x and coarse x (t)
                        nes.ppu.t |= val_u16 >> 3;
                        nes.ppu.x = val & 0b111;
                    } else {
                        nes.ppu.t &= 0b000_11_00000_11111;
                        // put coarse y in to t
                        nes.ppu.t |= (val_u16 & 0b11111000) << 2;
                        // put fine y at the end of t
                        nes.ppu.t |= (val_u16 & 0b00000111) << 12;
                    }
                    nes.ppu.w = !nes.ppu.w;
                },
                PPUADDR   => {
                    if nes.cpu.cycles < PPU_WARMUP {return};

                    if !nes.ppu.w {
                        nes.ppu.t &= 0b11000000_11111111;
                        // put the lower 6 bits into the upper 6 bits of t (address space is only 14bits)
                        nes.ppu.t |= (val_u16 & 0b00111111) << 8;
                        nes.ppu.t &= !(1 << 14); // clear 15th bit? idk if that does anything THIS SHIT BROKE EVERYTHING LOL
                    } else {
                        nes.ppu.t &= 0b11111111_00000000;
                        nes.ppu.t |= val_u16;
                        nes.ppu.v = nes.ppu.t;
                    }
                    nes.ppu.w = !nes.ppu.w;
                },
                PPUSTATUS => {},
                PPUDATA   => {
                    // println!("Write to vram! v is {:04X}, data is {:02X}", nes.ppu.v, val);
                    // println!("t is {:04X}, write toggle is {:?}", nes.ppu.t, nes.ppu.w);
                    // let mut input_string = String::new();
                    // std::io::stdin().read_line(&mut input_string).unwrap();
                    ppu::write_vram(nes.ppu.v, val, nes);
                    // Should really use enums or something, this is hard to read
                    let increment = if nes.ppu.increment_select == false {1} else {32};
                    nes.ppu.v = nes.ppu.v.wrapping_add(increment);
                },
                _ => panic!("Literally impossible"),
            },

        OAMDMA    => {
            let base = val_u16 << 8;
            for offset in 0x00..=0xFF {
                nes.ppu.oam[offset] = read_mem(base + offset as u16, nes);
            }
        }

        CONTROLLER_1 => {
            // If latch was high, now low, put controller state in shift register
            if nes.controller1.sr_latch_pin && (val & 1) == 0 {
                nes.controller1.shift_register = nes.controller1.button_state;
            }
            nes.controller1.sr_latch_pin = (val & 1) == 1;
        }

        PULSE_1_REG_1 => nes.apu.square1.set_reg1_from_byte(val),
        PULSE_1_REG_2 => nes.apu.square1.set_reg2_from_byte(val),
        PULSE_1_REG_3 => nes.apu.square1.set_reg3_from_byte(val),
        PULSE_1_REG_4 => nes.apu.square1.set_reg4_from_byte(val),

        PULSE_2_REG_1 => nes.apu.square2.set_reg1_from_byte(val),
        PULSE_2_REG_2 => nes.apu.square2.set_reg2_from_byte(val),
        PULSE_2_REG_3 => nes.apu.square2.set_reg3_from_byte(val),
        PULSE_2_REG_4 => nes.apu.square2.set_reg4_from_byte(val),

        TRIANGLE_REG_1 => nes.apu.triangle.set_reg1_from_byte(val),
        TRIANGLE_REG_2 => nes.apu.triangle.set_reg2_from_byte(val),
        TRIANGLE_REG_3 => nes.apu.triangle.set_reg3_from_byte(val),

        APU_STATUS_REG => {

            nes.apu.square1.enabled = (val & 0b01) > 0;
            nes.apu.square2.enabled = (val & 0b10) > 0;
            nes.apu.triangle.enabled = (val & 0b100) > 0;

            if !nes.apu.square1.enabled {nes.apu.square1.length_counter = 0;}
            if !nes.apu.square2.enabled {nes.apu.square2.length_counter = 0;}
            if !nes.apu.triangle.enabled {nes.apu.triangle.length_counter = 0;}

            // println!("Triangle enabled {}", nes.apu.triangle.enabled);

        }

        _ => (),
    };
}

fn byte_to_ppuctrl(byte: u8, nes: &mut Nes) {
    nes.ppu.nmi_enable = get_bit(byte, 7);
    nes.ppu.master_slave = get_bit(byte, 6);
    nes.ppu.tall_sprites = get_bit(byte, 5);
    nes.ppu.bg_ptable_select = get_bit(byte, 4);
    nes.ppu.sprite_ptable_select = get_bit(byte, 3);
    nes.ppu.increment_select = get_bit(byte, 2);
    nes.ppu.ntable_select = byte & 0b0000_0011;
}



fn byte_to_ppumask(byte: u8, nes: &mut Nes) {
    nes.ppu.blue_emphasis = get_bit(byte, 7);
    nes.ppu.green_emphasis = get_bit(byte, 6);
    nes.ppu.red_emphasis = get_bit(byte, 5);
    nes.ppu.show_sprites = get_bit(byte, 4);
    nes.ppu.show_bg = get_bit(byte, 3);
    nes.ppu.show_leftmost_sprites = get_bit(byte, 2);
    nes.ppu.show_leftmost_bg = get_bit(byte, 1);
    nes.ppu.greyscale = get_bit(byte, 0);
}
fn ppustatus_to_byte(nes: &Nes) -> u8 {
    (if nes.ppu.in_vblank       {0b1000_0000} else {0}) | 
    (if nes.ppu.sprite_zero_hit {0b0100_0000} else {0}) | 
    (if nes.ppu.sprite_overflow {0b0000_1000} else {0})
}
