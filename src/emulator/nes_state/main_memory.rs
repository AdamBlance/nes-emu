use crate::emulator::nes::cartridge::Mirroring;
use crate::emulator::nes::mem_consts::*;
use crate::emulator::nes::ppu::{
    PpuAddress, PpuCtrl, PpuMask, get_dynamic_latch, increment_v_after_ppudata_access, read_vram,
    set_dynamic_latch, write_vram,
};
use crate::emulator::nes_state::NesState;

impl NesState {
    pub fn read_mem(&mut self, addr: u16) -> u8 {
        let val = match addr {
            ..=WRAM_END_1FFF => self.wram[self.wram_mirroring(addr)],
            PPU_REG_START_2000..=PPU_REG_END_3FFF => self.memory_mapped_register_read(addr),
            OPEN_BUS_4000..=OPEN_BUS_4014 => self.cpu.ireg.open_bus,
            APU_STATUS_4015 => apu_status_read(nes),
            CON_1_4016 => self.con1.shift_out_button_state() | (self.cpu.ireg.open_bus & 0xE0),
            CON_2_4017 => self.con2.shift_out_button_state() | (self.cpu.ireg.open_bus & 0xE0),
            OPEN_BUS_4018..=OPEN_BUS_5FFF => self.cpu.ireg.open_bus,
            PRG_RAM_START_6000..=PRG_RAM_END_7FFF => self.read_prg_ram(addr),
            PRG_ROM_START_8000.. => self.cart.read_prg_rom(addr),
        };
        // The data bus isn't used when reading 0x4015
        if addr != APU_STATUS_4015 {
            self.cpu.ireg.open_bus = val;
        }
        val
    }

    fn read_prg_ram(&mut self, addr: u16) -> u8 {
        // If the cartridge doesn't have any PRG RAM whatever's on the open bus is returned
        self.cart
            .read_prg_ram(addr)
            .unwrap_or(self.cpu.ireg.open_bus)
    }

    pub fn write_mem(&mut self, addr: u16, val: u8) {
        self.cpu.ireg.open_bus = val;
        match addr {
            ..=WRAM_END_1FFF => self.wram[self.wram_mirroring(addr)] = val,
            PPU_REG_START_2000..=PPU_REG_END_3FFF => self.memory_mapped_register_write(addr, val),
            APU_REG_START_4000..=APU_REG_END_4013 => apu_channels_write(addr, val, nes),
            OAMDMA_4014 => self.write_oam_data(val),
            APU_STATUS_4015 => apu_status_write(val, nes),
            CON_1_4016 => self.con1.write_to_data_latch(val),
            CON_2_AND_APU_FRAME_COUNTER_4017 => self.write_to_con2_shared_4017(val),
            PRG_RAM_START_6000..=PRG_RAM_END_7FFF => self.cart.write_prg_ram(addr, val),
            PRG_ROM_START_8000.. => self.cart.write_prg_rom(addr, val),
        };
    }

    fn write_oam_data(&mut self, val: u8) {
        let base = (val as u16) << 8;
        // TODO: OAM DMA doesn't stall CPU - implement this properly
        for offset in 0x00..=0xFF {
            self.ppu.memory.oam[offset] = read_mem(&mut self, base + offset as u16, nes);
        }
    }

    fn write_to_con2_shared_4017(&mut self, val: u8) {
        self.con2.write_to_data_latch(val);
        self.apu.frame_sequencer_mode_1 = (val & 0x80) > 0;
        self.apu.frame_sequencer_interrupt_inhibit = (val & 0b0100_0000) > 0;
    }

    fn wram_mirroring(addr: u16) -> usize {
        addr as usize % 0x800
    }
}
// Would be fun to try and make this generic
// TODO: Get this outta here
#[derive(Clone)]
pub struct U3(u8);
impl U3 {
    pub(crate) fn new(x: u8) -> Self {
        Self(0b111 & x)
    }
}
