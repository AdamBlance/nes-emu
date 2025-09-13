use crate::nes::apu::{apu_channels_write, apu_status_read, apu_status_write};
use crate::nes::Nes;
use crate::nes::ppu::{memory_mapped_register_read, memory_mapped_register_write};
use crate::nes::mem_consts::*;

pub fn read_mem(addr: u16, nes: &mut Nes) -> u8 {
    let value_read = match addr {
        ..=WRAM_END_1FFF =>
            nes.wram[(addr % 0x800) as usize],
        PPU_REG_START_2000..=PPU_REG_END_3FFF =>
            memory_mapped_register_read(addr, nes),
        OPEN_BUS_4000..=OPEN_BUS_4014 =>
            nes.cpu.open_bus,
        APU_STATUS_4015 =>
            apu_status_read(nes),
        CON_1_4016 =>
            nes.con1.shift_out_button_state() | (nes.cpu.open_bus & 0b1110_0000),
        CON_2_4017 =>
            nes.con2.shift_out_button_state() | (nes.cpu.open_bus & 0b1110_0000),
        OPEN_BUS_4018..=OPEN_BUS_5FFF =>
            nes.cpu.open_bus,
        PRG_RAM_START_6000..=PRG_RAM_END_7FFF =>
            nes.cart.read_prg_ram(addr).unwrap_or(nes.cpu.open_bus),
        PRG_ROM_START_8000.. =>
            nes.cart.read_prg_rom(addr),
    };
    // The data bus isn't used when reading 0x4015
    if addr != APU_STATUS_4015 {
        nes.cpu.open_bus = value_read;
    }
    value_read
}

pub fn write_mem(addr: u16, val: u8, nes: &mut Nes) {
    nes.cpu.open_bus = val;
    match addr {
        ..=WRAM_END_1FFF =>
            nes.wram[(addr % 0x800) as usize] = val,
        PPU_REG_START_2000..=PPU_REG_END_3FFF =>
            memory_mapped_register_write(addr, val, nes),
        APU_REG_START_4000..=APU_REG_END_4013 =>
            apu_channels_write(addr, val, nes),
        OAMDMA_4014 => {
            let base = (val as u16) << 8;
            // TODO: OAM DMA doesn't stall CPU
            for offset in 0x00..=0xFF {
                nes.ppu.oam[offset] = read_mem(base + offset as u16, nes);
            }
        }
        APU_STATUS_4015 =>
            apu_status_write(val, nes),
        CON_1_4016 =>
            nes.con1.write_to_data_latch(val),
        CON_2_AND_APU_FRAME_COUNTER_4017 => {
            nes.con2.write_to_data_latch(val);
            nes.apu.frame_sequencer_mode_1 = (val & 0b1000_0000) > 0;
            nes.apu.frame_sequencer_interrupt_inhibit = (val & 0b0100_0000) > 0;
        }
        PRG_RAM_START_6000..=PRG_RAM_END_7FFF =>
            nes.cart.write_prg_ram(addr, val),

        PRG_ROM_START_8000.. => nes.cart.write_prg_rom(addr, val),
        _ => (),
    };
}
