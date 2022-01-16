use crate::hw::*;
use crate::mem::*;
use crate::cpu;
use crate::ppu;

// PPU shouldn't be touched until 29658 CPU cycles have passed.
// Some registers will work immediately on start up, like PPUSTATUS
// Constantly checking PPUSTATUS bit-7 VBLANK flag with BIT instruction in a loop
// will let 27,000 odd cycles pass
// Doing it again will let 50,000 odd cycles pass
// By this point, everything should work

// PPU needs to report how many cycles are left until it will raise VBLANK NMI
// CPU should run until it either affects the PPU, has run up to VBLANK as reported by PPU
// On NTSC, 1 CPU tick to 3 PPU ticks

// 341 cycles per scanline
// 262 scanlines


pub fn run_to_vblank(nes: &mut Nes) {

    if nes.cpu.cycles == 0 {
        nes.cpu.pc = read_mem_u16(0xFFFC, nes);
        nes.cpu.p_i = true;
        nes.cpu.s = 0xFD;
    }

    let mut last_nmi_state = true;

    while !((last_nmi_state == false) && (nes.cpu.nmi_interrupt == true)) {
        last_nmi_state = nes.cpu.nmi_interrupt;
        cpu::step_cpu(nes);

        ppu::step_ppu(nes);
        ppu::step_ppu(nes);
        ppu::step_ppu(nes);
    }

}




