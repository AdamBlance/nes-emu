use crate::emulator::nes::cartridge::Mirroring;
use crate::emulator::nes::mem_consts::{
    PALETTE_RAM_END_3FFF, PALETTE_RAM_START_3F00, PATTERN_TABLE_END_1FFF, VRAM_END_3EFF,
    VRAM_START_2000,
};
use crate::emulator::nes_state::NesState;

impl NesState {
    pub fn read_vram(&mut self, addr: u16) -> u8 {
        match addr {
            0x0000..=PATTERN_TABLE_END_1FFF => self.cart.read_chr(addr),
            VRAM_START_2000..=VRAM_END_3EFF => self.ppu.memory.vram[self.mirroring_mapping(addr)],
            PALETTE_RAM_START_3F00..=PALETTE_RAM_END_3FFF => {
                let colour = self.ppu.memory.palette_mem[self.map_vram_addr_to_palette_addr(addr)];
                if self.ppu.ppu_mask.greyscale() {
                    colour & 0x30
                } else {
                    colour
                }
            }
            x => panic!("Invalid PPU address {x:016b}"),
        }
    }

    pub fn write_vram(&mut self, addr: u16, val: u8) {
        match addr {
            ..=PATTERN_TABLE_END_1FFF => self.cart.write_chr(addr, val),
            VRAM_START_2000..=VRAM_END_3EFF => {
                self.ppu.memory.vram[self.cart.mirroring_mapping()] = val
            }
            PALETTE_RAM_START_3F00..=PALETTE_RAM_END_3FFF => {
                self.ppu.memory.palette_mem[self.map_vram_addr_to_palette_addr(addr)] = val
            }
            x => panic!("Invalid PPU address {x:016b}"),
        }
    }

    fn map_vram_addr_to_palette_addr(addr: u16) -> usize {
        let offset = ((addr - 0x3F00) % 0x20) as usize;
        // Shared first entry between sprites and background
        if offset > 0xF && offset % 4 == 0 {
            offset - 0x10
        } else {
            offset
        }
    }

    pub fn mirroring_mapping(&self, addr: u16) -> usize {
        // The physical nametables sit at 0x2000..=0x23FF and 0x2400..=0x27FF
        let truncated = addr & 0b0000_1111_1111_1111;
        match self.cart.mirroring() {
            Mirroring::Vertical => truncated % 0x800,
            Mirroring::Horizontal => (truncated / 0x800) * 0x400 + (truncated % 0x400),
            Mirroring::SingleScreenLower => truncated % 0x400,
            Mirroring::SingleScreenUpper => 0x400 + (truncated % 0x400),
        }
    }
}
