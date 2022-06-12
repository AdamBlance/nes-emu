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

const NOISE_REG_1: u16 = 0x400C;
const NOISE_REG_2: u16 = 0x400E;
const NOISE_REG_3: u16 = 0x400F;

const APU_STATUS_REG: u16 = 0x4015;

const OAMDMA: u16    = 0x4014;

const CONTROLLER_1: u16 = 0x4016;
const CONTROLLER_2: u16 = 0x4017;


const PPU_WARMUP: u64 = 29658;


pub fn read_mem(addr: u16, nes: &mut Nes) -> u8 {
    match addr {

        // Main memory, mirrored 4 times
        0x0000..=0x1FFF => nes.wram[(addr % 0x800) as usize],

        // PPU memory mapped registers are mirrored through this range
        0x2000..=0x3FFF => match 0x2000 + (addr % 8) {
            // Reading PPUSTATUS clears flags
            PPUSTATUS => {
                let status = nes.ppu.get_ppustatus_byte();
                nes.ppu.in_vblank = false;
                nes.ppu.w = false;
                status
            }
            OAMDATA => nes.ppu.oam_addr,
            PPUDATA => {
                // PPUDATA latch behaves differently when reading palette data
                if addr < 0x3F00 {
                    // Read what was already in the buffer
                    let prev_data_in_buffer = nes.ppu.ppudata_buffer;
                    // Fill the buffer with the value read from VRAM
                    nes.ppu.ppudata_buffer = ppu::read_vram(nes.ppu.v, nes);;
                    ppu::increment_v_after_ppudata_access(nes);
                    // Return what was in the buffer before memory read
                    prev_data_in_buffer
                } else {
                    // Get palette data from memory
                    let data_in_memory = ppu::read_vram(nes.ppu.v, nes);
                    // Fill the buffer with the data at v - 0x1000
                    nes.ppu.ppudata_buffer = ppu::read_vram(nes.ppu.v.wrapping_sub(0x1000), nes);
                    ppu::increment_v_after_ppudata_access(nes);
                    // Return palette data from memory
                    data_in_memory
                }
            }
            _ => 0,
        }

        // Should read back data about the length counters
        APU_STATUS_REG => 0,

        CONTROLLER_1 => nes.controller1.shift_out_button_state(),
        CONTROLLER_2 => nes.controller2.shift_out_button_state(),

        // Cartridge space
        0x4020..=0xFFFF => {
            let prg_rom_addr = nes.cartridge.mapper.get_raw_prg_address(addr);
            nes.cartridge.prg_rom[prg_rom_addr]
        }

        _ => 0,
    }
}



pub fn write_mem(addr: u16, val: u8, nes: &mut Nes) {
    let val_u16 = val as u16;
    match addr {

        // Main memory, mirrored 4 times
        0x0000..=0x1FFF => nes.wram[(addr % 0x800) as usize] = val,

        // PPU memory mapped registers are mirrored through this range
        0x2000..=0x3FFF => match 0x2000 + (addr % 8) {
            PPUCTRL => {
                if nes.cpu.cycles < PPU_WARMUP {return};
                nes.ppu.set_ppuctrl_from_byte(val);
                nes.ppu.t &= !ppu::NAMETABLE;
                nes.ppu.t |= (val_u16 & 0b11) << 10;
            }
            PPUMASK => {
                if nes.cpu.cycles < PPU_WARMUP {return};
                nes.ppu.set_ppumask_from_byte(val);
            }
            OAMADDR => nes.ppu.oam_addr = val,
            OAMDATA => {
                nes.ppu.oam[nes.ppu.oam_addr as usize] = val;
                nes.ppu.oam_addr = nes.ppu.oam_addr.wrapping_add(1);
            }
            PPUSCROLL => {
                if nes.cpu.cycles < PPU_WARMUP {return};
                if !nes.ppu.w {
                    // Put x-scroll into t, x after first write
                    nes.ppu.t &= !ppu::COARSE_X;
                    nes.ppu.t |= val_u16 >> 3;
                    nes.ppu.x = val & 0b111;
                } else {
                    // Put y-scroll into, t after second write
                    nes.ppu.t &= !(ppu::COARSE_Y | ppu::FINE_Y);
                    nes.ppu.t |= (val_u16 & 0b11111_000) << 2;
                    nes.ppu.t |= (val_u16 & 0b00000_111) << 12;
                }
                nes.ppu.w = !nes.ppu.w;
            }
            PPUADDR => {
                if nes.cpu.cycles < PPU_WARMUP {return};
                if !nes.ppu.w {
                    // Write into upper 6 bits of t
                    nes.ppu.t &= 0b000000_11111111;
                    nes.ppu.t |= (val_u16 & 0b111111) << 8;
                } else {
                    // Write into lower 8 bits of t
                    nes.ppu.t &= 0b111111_00000000;
                    nes.ppu.t |= val_u16;
                    // Copy t into v
                    nes.ppu.v = nes.ppu.t;
                }
                nes.ppu.w = !nes.ppu.w;
            }
            PPUDATA => {
                ppu::write_vram(nes.ppu.v, val, nes);
                ppu::increment_v_after_ppudata_access(nes);
            }
            _ => (),
        },

        OAMDMA => {
            let base = val_u16 << 8;
            for offset in 0x00..=0xFF {
                nes.ppu.oam[offset] = read_mem(base + offset as u16, nes);
            }
        }

        CONTROLLER_1 => nes.controller1.write_to_data_latch(val),
        CONTROLLER_2 => nes.controller2.write_to_data_latch(val),

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

        NOISE_REG_1 => nes.apu.noise.set_reg1_from_byte(val),
        NOISE_REG_2 => nes.apu.noise.set_reg2_from_byte(val),
        NOISE_REG_3 => nes.apu.noise.set_reg3_from_byte(val),
        
        APU_STATUS_REG => {
            nes.apu.square1.enabled = (val & 0b01) > 0;
            nes.apu.square2.enabled = (val & 0b10) > 0;
            nes.apu.triangle.enabled = (val & 0b100) > 0;
            nes.apu.noise.enabled = (val & 0b1000) > 0;

            if !nes.apu.square1.enabled {nes.apu.square1.length_counter = 0;}
            if !nes.apu.square2.enabled {nes.apu.square2.length_counter = 0;}
            if !nes.apu.triangle.enabled {nes.apu.triangle.length_counter = 0;}
            if !nes.apu.noise.enabled {nes.apu.noise.length_counter = 0;}
        }
        0x4020..=0xFFFF => {
            nes.cartridge.mapper.prg_write(addr, val, nes.cpu.cycles);
        }
        _ => (),
    };
}
