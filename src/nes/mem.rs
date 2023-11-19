
use crate::nes::Nes;
use crate::nes::ppu;

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

const SAMPLE_REG_1: u16 = 0x4010;
const SAMPLE_REG_2: u16 = 0x4011;
const SAMPLE_REG_3: u16 = 0x4012;
const SAMPLE_REG_4: u16 = 0x4013;

const OAMDMA: u16    = 0x4014;

const APU_STATUS_REG: u16 = 0x4015;

const CONTROLLER_1: u16 = 0x4016;
const CONTROLLER_2_AND_FRAME_COUNTER_REG: u16 = 0x4017;


const PPU_WARMUP: u64 = 29658;

pub fn read_mem(addr: u16, nes: &mut Nes) -> u8 {
    read_mem_with_safety(nes, addr, false)
}

pub fn read_mem_safe(addr: u16, nes: &mut Nes) -> u8 {
    read_mem_with_safety(nes, addr, true)
}

pub fn read_mem_with_safety(nes: &mut Nes, addr: u16, safe_read: bool) -> u8 {
    match addr {

        // Main memory, mirrored 4 times
        0x0000..=0x1FFF => nes.wram[(addr % 0x800) as usize],

        // PPU memory mapped registers are mirrored through this range
        0x2000..=0x3FFF => match 0x2000 + (addr % 8) {
            PPUCTRL | PPUMASK | OAMADDR | PPUSCROLL | PPUADDR => nes.ppu.dynamic_latch,
            
            // Reading PPUSTATUS clears flags
            PPUSTATUS => {
                /*
                
                    Notes, okay, so,

                    One CPU cycle lasts for 3 PPU cycles. 

                    If CPU reads at scanline dot 0, the CPU reads 0, which makes sense right?
                    That's what it would do anyway since it hasn't been set yet.
                    The only difference is that NMI doesn't fire on the next PPU cycle, that could
                    be a flag in the PPU or the CPU that we check before asserting NMI.

                    If the CPU reads at scanline dot 1, it reads back a 1, setting the value back
                    to 0. Although the NMI line briefly goes low, it is brought back up again
                    immediately, so the CPU doesn't have a chance to process the falling edge. 
                    We don't have to worry about this, and can simply not pull the NMI line low. 

                    Since this emulator runs the CPU clock, then 3 PPU clocks sequentially, 
                    the PPUSTATUS flag won't actually be set when we go to read it. In this case,
                    we have to use a little hack. That's fine though, this emulator will still be 
                    cycle perfect, lol. 

                    If the CPU reads at scanline dot 2, the NMI line will have been asserted for 
                    one PPU cycle. When a CPU cycle starts, it will check for rising edges, and will
                    detect the low NMI line. Since this is just the edge detector and not any sort
                    of pending NMI state, we can just reset the state of the edge detector. 


                */



                let status = nes.ppu.get_ppustatus_byte() | (nes.ppu.dynamic_latch & 0b0001_1111);

                if !safe_read {
                    nes.ppu.in_vblank = false;
                    nes.ppu.w = false;
                    nes.ppu.dynamic_latch = status
                }
                status
            }
            OAMDATA => {
                let addr = nes.ppu.oam_addr;
                if !safe_read {
                    nes.ppu.dynamic_latch = addr;
                }
                addr
            }
            PPUDATA => {
                // PPUDATA latch behaves differently when reading palette data
                if addr < 0x3F00 {
                    // Read what was already in the buffer
                    let prev_data_in_buffer = nes.ppu.ppudata_buffer;
                    if !safe_read {
                        // Fill the buffer with the value read from VRAM
                        nes.ppu.ppudata_buffer = ppu::read_vram(nes.ppu.v, nes);
                        ppu::increment_v_after_ppudata_access(nes);
                        nes.ppu.dynamic_latch = prev_data_in_buffer;
                    }
                    // Return what was in the buffer before memory read
                    prev_data_in_buffer
                } else {
                    // Get palette data from memory
                    let data_in_memory = ppu::read_vram(nes.ppu.v, nes);
                    if !safe_read {
                        // Fill the buffer with the data at v - 0x1000
                        nes.ppu.ppudata_buffer = ppu::read_vram(nes.ppu.v.wrapping_sub(0x1000), nes);
                        ppu::increment_v_after_ppudata_access(nes);
                        nes.ppu.dynamic_latch = data_in_memory;
                    }
                    // Return palette data from memory
                    data_in_memory
                }
            }
            _ => 0,
        }

        APU_STATUS_REG => {
            let result = nes.apu.square1.length_counter.min(1)
                       | (nes.apu.square2.length_counter.min(1) << 1)
                       | (nes.apu.triangle.length_counter.min(1) << 2)
                       | (nes.apu.noise.length_counter.min(1) << 3)
                       | ((nes.apu.sample.remaining_sample_bytes.min(1) as u8) << 4)
                       | ((nes.apu.interrupt_request as u8) << 6)
                       | ((nes.apu.sample.interrupt_request as u8) << 7);
            if !safe_read {
                nes.apu.interrupt_request = false;
            }
            result
        },

        CONTROLLER_1 => nes.con1.shift_out_button_state(),
        CONTROLLER_2_AND_FRAME_COUNTER_REG => nes.con2.shift_out_button_state(),

        // Cartridge space

        0x6000..=0x7FFF => nes.cart.read_prg_ram(addr),

        0x8000..=0xFFFF => nes.cart.read_prg_rom(addr),

        _ => 0,
    }
}



pub fn write_mem(addr: u16, val: u8, nes: &mut Nes) {
    let val_u16 = val as u16;
    match addr {

        // Main memory, mirrored 4 times
        0x0000..=0x1FFF => nes.wram[(addr % 0x800) as usize] = val,

        // Open bus behaviour here is incorrect, dynamic latch and bus are the same? 

        // PPU memory mapped registers are mirrored through this range
        0x2000..=0x3FFF => {
            nes.ppu.dynamic_latch = val;
            match 0x2000 + (addr % 8) {
                PPUCTRL => {
                    if nes.cpu.cycles < PPU_WARMUP {return};
                    nes.ppu.set_ppuctrl_from_byte(val);
                    nes.ppu.t &= !ppu::NAMETABLE;
                    nes.ppu.t |= (val_u16 & 0b11) << 10;
                    // println!("nmi {}", nes.ppu.nmi_enable);
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
                        nes.ppu.addr_bus = nes.ppu.v;
                        // println!("new ppu addr {:013b} {:04X}", nes.ppu.v, nes.ppu.v);
                    }
                    nes.ppu.w = !nes.ppu.w;
                }
                PPUDATA => {
                    ppu::write_vram(nes.ppu.v, val, nes);
                    ppu::increment_v_after_ppudata_access(nes);
                }
            _ => (),
            }
        },

        OAMDMA => {
            let base = val_u16 << 8;
            for offset in 0x00..=0xFF {
                nes.ppu.oam[offset] = read_mem(base + offset as u16, nes);
            }
        }

        CONTROLLER_1 => nes.con1.write_to_data_latch(val),

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
        
        SAMPLE_REG_1 => nes.apu.sample.set_reg1_from_byte(val),
        SAMPLE_REG_2 => nes.apu.sample.set_reg2_from_byte(val),
        SAMPLE_REG_3 => nes.apu.sample.set_reg3_from_byte(val),
        SAMPLE_REG_4 => nes.apu.sample.set_reg4_from_byte(val),

        APU_STATUS_REG => {

            nes.apu.sample.interrupt_request = false;

            nes.apu.square1.enabled = (val & 0b01) > 0;
            nes.apu.square2.enabled = (val & 0b10) > 0;
            nes.apu.triangle.enabled = (val & 0b100) > 0;
            nes.apu.noise.enabled = (val & 0b1000) > 0;
            nes.apu.sample.enabled = (val & 0b10000) > 0;

            if !nes.apu.square1.enabled {nes.apu.square1.length_counter = 0;}
            if !nes.apu.square2.enabled {nes.apu.square2.length_counter = 0;}
            if !nes.apu.triangle.enabled {nes.apu.triangle.length_counter = 0;}
            if !nes.apu.noise.enabled {nes.apu.noise.length_counter = 0;}
            if !nes.apu.sample.enabled {nes.apu.sample.remaining_sample_bytes = 0;
            } else {
                nes.apu.sample.remaining_sample_bytes = nes.apu.sample.sample_length;
                nes.apu.sample.curr_sample_addr = nes.apu.sample.init_sample_addr;
            } // fix this, add silence flag like other channels

        }

        CONTROLLER_2_AND_FRAME_COUNTER_REG => {

            nes.con2.write_to_data_latch(val);
            nes.apu.frame_sequencer_mode_1 = (val & 0b1000_0000) > 0;
            nes.apu.frame_sequencer_interrupt_inhibit = (val & 0b0100_0000) > 0;
        }

        // Cartridge space

        0x6000..=0x7FFF => nes.cart.write_prg_ram(addr, val),

        0x8000..=0xFFFF => nes.cart.write_prg_rom(addr, val),

        _ => (),
    };
}
