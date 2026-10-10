use crate::emulator::nes::ppu::{
    PpuAddress, PpuCtrl, PpuMask, increment_v_after_ppudata_access, read_vram,
};
use crate::emulator::nes_state::NesState;
use crate::emulator::nes_state::consts::*;
use crate::emulator::nes_state::main_memory::U3;

impl NesState {
    pub fn memory_mapped_register_read(&mut self, addr: u16) -> u8 {
        match 0x2000 + (addr % 8) {
            PPUCTRL_2000 | PPUMASK_2001 | OAMADDR_2003 | PPUSCROLL_2005 | PPUADDR_2006 => {
                self.get_dynamic_latch()
            }
            addr => {
                let val = match addr {
                    PPUSTATUS_2002 => self.read_ppu_status(),
                    OAMDATA_2004 => self.ppu.memory.oam_addr,
                    PPUDATA_2007 => self.read_ppu_data(),
                    _ => unreachable!(),
                };
                self.set_dynamic_latch(val);
                val
            }
        }
    }

    fn read_ppu_status(&mut self) -> u8 {
        let [status] = self.ppu.ppu_status.into_bytes();
        // Lower 5 bits of PPUSTATUS are open bus
        let status_with_dl = status | (self.get_dynamic_latch() & 0x1F);
        self.ppu.ppu_status.set_in_vblank(false);
        self.ppu.scroll.w = false;
        status_with_dl
    }

    fn read_ppu_data(&mut self) -> u8 {
        let val = match self.ppu.scroll.v.into() {
            v @ 0x0000..=VRAM_END_3EFF => {
                let val_in_buffer = self.ppu.ppudata_buffer;
                self.ppu.ppudata_buffer = read_vram(v);
                val_in_buffer
            }
            v @ PALETTE_RAM_START_3F00..=PALETTE_RAM_END_3FFF => {
                let val_in_memory = read_vram(v);
                // Some weird behaviour when the PPU reads from palette memory
                self.ppu.ppudata_buffer = read_vram(v.wrapping_sub(0x1000));
                val_in_memory
            }
        };
        self.ppu.scroll.v += self.ppu.ppu_ctrl.increment() as u8;
        val
    }

    pub fn memory_mapped_register_write(&mut self, addr: u16, val: u8) {
        if self.cpu.debug.cycles < PPU_WARMUP
            && matches!(
                addr,
                PPUCTRL_2000 | PPUMASK_2001 | PPUSCROLL_2005 | PPUADDR_2006
            )
        {
            return;
        }

        self.set_dynamic_latch(val);

        match 0x2000 + (addr % 8) {
            PPUCTRL_2000 => {
                self.ppu.ppu_ctrl = PpuCtrl::from_bytes([val]);
                self.ppu.scroll.t.set_nametable(val);
            }
            PPUMASK_2001 => self.ppu.ppu_mask = PpuMask::from_bytes([val]),
            OAMADDR_2003 => self.ppu.memory.oam_addr = val,
            OAMDATA_2004 => {
                self.ppu.memory.oam[self.ppu.memory.oam_addr as usize] = val;
                self.ppu.memory.oam_addr = self.ppu.memory.oam_addr.wrapping_add(1);
            }
            PPUSCROLL_2005 if !self.ppu.scroll.w => {
                // Put x-scroll into t and x after first write
                self.ppu.scroll.t.set_coarse_x_scroll(val >> 3);
                self.ppu.scroll.x = U3::new(val & 0b111);
                self.ppu.scroll.w = true;
            }
            PPUSCROLL_2005 if self.ppu.scroll.w => {
                // Put y-scroll into t after second write
                self.ppu.scroll.t.set_coarse_y_scroll(val >> 3);
                self.ppu.scroll.t.set_fine_y_scroll(val & 0b111);
                self.ppu.scroll.w = false;
            }
            PPUADDR_2006 if !self.ppu.scroll.w => {
                // Write into upper 6 bits of t
                let [_, lower] = self.ppu.scroll.t.into_bytes();
                self.ppu.scroll.t = PpuAddress::from_bytes([val & 0x3F, lower]);
                self.ppu.scroll.w = true;
            }
            PPUADDR_2006 if self.ppu.scroll.w => {
                // Write into lower 8 bits of t
                let [upper, _] = self.ppu.scroll.t.into_bytes();
                self.ppu.scroll.t = PpuAddress::from_bytes([upper, val]);
                self.ppu.scroll.v = self.ppu.scroll.t;
                self.ppu.scroll.w = false;
            }
            PPUDATA_2007 => {
                self.write_vram(nes.ppu.v, val, nes);
                increment_v_after_ppudata_access(nes);
            }
            _ => (),
        }
    }

    fn write_ppu_scroll(&mut self, val: u8) {
        if self.ppu.scroll.w == false {
            // Put x-scroll into t and x after first write
            self.ppu.scroll.t.set_coarse_x_scroll(val >> 3);
            self.ppu.scroll.x = U3::new(val);
        } else {
            // Put y-scroll into t after second write
            self.ppu.scroll.t.set_coarse_y_scroll(val >> 3);
            self.ppu.scroll.t.set_fine_y_scroll(val & 0b111);
        }
        self.ppu.scroll.w = !self.ppu.scroll.w;
    }

    fn write_ppu_addr(&mut self, val: u8) {
        if self.ppu.scroll.w == false {
            // Write into upper 6 bits of t
            let [_, lower] = self.ppu.scroll.t.into_bytes();
            self.ppu.scroll.t = PpuAddress::from_bytes([val & 0x3F, lower]);
        } else {
            // Write into lower 8 bits of t
            let [upper, _] = self.ppu.scroll.t.into_bytes();
            self.ppu.scroll.t = PpuAddress::from_bytes([upper, val]);
            self.ppu.scroll.v = self.ppu.scroll.t;
        }
        self.ppu.scroll.w = !self.ppu.scroll.w;
    }

    // This latch is just a consequence of the capacitance of some really long traces inside the PPU

    pub fn get_dynamic_latch(&mut self) -> u8 {
        if self.ppu.cycles - self.ppu.dynamic_latch_last_set_cycle > PPU_DYNAMIC_LATCH_DECAY_TIME {
            self.ppu.dynamic_latch = 0;
        }
        self.ppu.dynamic_latch
    }

    pub fn set_dynamic_latch(&mut self, val: u8) {
        self.ppu.dynamic_latch = val;
        self.ppu.dynamic_latch_last_set_cycle = self.ppu.cycles;
    }
}
