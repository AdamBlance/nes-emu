use crate::instr_defs::Instruction;
use crate::util::*;

pub struct Nes {
    pub cpu: Cpu,
    pub wram: [u8; 2048],
    pub ppu: Ppu,
    pub cart: Cartridge,
    pub frame: Vec<u8>,
    pub skip: u64,
    pub old_cpu_state: Cpu,
    pub old_ppu_state: Ppu,
    pub jammed: bool,
    pub ppu_log_toggle: bool,
    pub controller_1: Controller,
    pub controller_2: Controller,
}

#[derive(Copy, Clone, Default)]
pub struct Controller {
    pub button_state: u8,
    pub shift_register: u8,    
    pub sr_latch_pin: bool,
}

pub struct Cartridge {
    pub prg_rom: Vec<u8>,
    pub chr_rom: Vec<u8>,
    pub mapper: u8,
    pub v_mirroring: bool,
}




/*

sound is too hard, going to improve the cpu and ppu for now? 
yeah, will try to get mario running first
so sprite 0 hit flag and one byte delay when reading ppudata need to be implemented

APU NOTES

Two square wave channels
One triangle wave channel
One noise channel
One sample channel


a divider outputs a clock every n input clocks, where n is the dividers period
internally, this is just a counter that gets decremented by the input clock until it hits 0
then the value n is reloaded




frame conuter

so there's this thing called the frame counter (nothing to do with video frames) that has a divider
it divides the master clock to produce a 240Hz clock (that I think can be used by channels)

in the frame counter, there is also a sequencer, which I think is like a selector in 
little big planet? 

the sequencer is not controlled by the 240Hz clock! the sequencer is clocked every second CPU cycle

there is some counter in the sequencer that counts how many apu cycles have passed (1 apu = 2 cpu)

once the count has reached some value, the sequencer advances and the count resets

the sequencer will "clock" envelopes, sweep units, length counters and all that stuff
once that step in the sequencer is reached, those things happen


    f = set interrupt flag
    l = clock length counters and sweep units
    e = clock envelopes and triangle's linear counter

mode 0: 4-step  effective rate (approx)
---------------------------------------
    - - - f      60 Hz
    - l - l     120 Hz
    e e e e     240 Hz

mode 1: 5-step  effective rate (approx)
---------------------------------------
    - - - - -   (interrupt flag never set)
    l - l - -    96 Hz
    e e e e -   192 Hz

right so the 240Hz clock does clock the sequencer!
I guess? I suppose the exact reason doesn't matter



most channels have a "length counter", which will store a number and count down
when the counter reaches zero, the channel will be muted

all channels have a bit that says whether they will stop when the counter reaches zero


Square wave channels

square waves have a duty cycle that can be modified
two bits in the i/o register can change this

there is a constant volume bit, which determines whether the volume is constant, or controlled
by an envelope

an envelope is like, a changing value? So like EEeeeeeuuuuoooooommmmmm ↓

if volume is constant, the volume is stored in 4 bits

there is a bit to enable and disable sweep




right so there's envelope which is volume
there's sweep which is pitch (period)
duty cycle, which changes timbre

there are also length counters, which is clocked by the frame thing
these stop the sound when the counter is 0, setting a length for the note basically





*/

// #[derive(Copy, Clone)]
// pub struct Apu {

//     pub duty_cycle: u8,
//     pub volume: u8,
//     pub period

// }




#[derive(Copy, Clone)]
pub struct Ppu {
    // PPUCTRL
    pub nmi_enable: bool,
    pub master_slave: bool,
    pub tall_sprites: bool,
    pub bg_ptable_select: bool,
    pub sprite_ptable_select: bool,
    pub increment_select: bool,
    pub ntable_select: u8, 

    // PPUMASK
    pub blue_emphasis: bool,
    pub green_emphasis: bool,
    pub red_emphasis: bool,
    pub show_sprites: bool,
    pub show_bg: bool,
    pub show_leftmost_sprites: bool,
    pub show_leftmost_bg: bool,
    pub greyscale: bool,

    // PPUSTATUS
    pub in_vblank: bool,
    pub sprite_zero_hit: bool,
    pub sprite_overflow: bool,

    // OAMADDR
    pub oam_addr: u8,

    // Memories
    pub vram:        [u8; 2048],
    pub oam:         [u8; 256],
    pub s_oam:       [u8; 32],
    pub palette_mem: [u8; 32],

    pub in_range_counter: u8,

    pub ppudata_buffer: u8,

    // sprite stuff
    pub sprite_lsb_srs: [u8; 8],
    pub sprite_msb_srs: [u8; 8],

    pub sprite_property_latches: [u8; 8],
    pub sprite_x_counters: [u8; 8],

    pub sprite_zero_in_soam: bool,
    pub sprite_zero_in_latches: bool,

    // v/t PPU addresses
    pub t: u16,
    pub v: u16,
    
    // Fine X
    pub x: u8,

    // Write toggle
    pub w: bool,

    // Rendering counters
    pub scanline: i32,
    pub scanline_cycle: i32,
    pub odd_frame: bool,

    // Internal latches for just-read values
    pub ntable_tmp: u8,
    pub attr_tmp: u8,
    pub ptable_lsb_tmp: u8,
    pub ptable_msb_tmp: u8,
    
    // Shift registers
    pub ptable_lsb_sr: u16,
    pub ptable_msb_sr: u16,
    pub attr_lsb_sr: u8,
    pub attr_msb_sr: u8,
    
    // 1-bit attribute latches
    pub attr_lsb_latch: bool,
    pub attr_msb_latch: bool,
    
    pub cycles: u64,
}

impl Default for Ppu {
    fn default() -> Ppu {
        Ppu {
            nmi_enable: false,
            master_slave: false,
            tall_sprites: false,
            bg_ptable_select: false,
            sprite_ptable_select: false,
            increment_select: false,
            ntable_select: 0,
        
            blue_emphasis: false,
            green_emphasis: false,
            red_emphasis: false,
            show_sprites: false,
            show_bg: false,
            show_leftmost_sprites: false,
            show_leftmost_bg: false,
            greyscale: false,
        
            in_vblank: false,
            sprite_zero_hit: false,
            sprite_overflow: false,

            oam_addr: 0,

            sprite_zero_in_soam: false,
            sprite_zero_in_latches: false,

            vram: [0; 2048],
            oam: [0; 256],
            s_oam: [0; 32],
            palette_mem: [0; 32],

            in_range_counter: 0,

            ppudata_buffer: 0,

            sprite_lsb_srs: [0; 8],
            sprite_msb_srs: [0; 8],

            sprite_property_latches: [0; 8],
            sprite_x_counters: [0; 8],

            t: 0,
            v: 0,
            x: 0,
            w: false,

            scanline: 0,
            scanline_cycle: 0,
            odd_frame: false,

            ntable_tmp: 0,
            attr_tmp: 0,
            ptable_lsb_tmp: 0,
            ptable_msb_tmp: 0,
            
            ptable_lsb_sr: 0,
            ptable_msb_sr: 0,
            attr_lsb_sr: 0,
            attr_msb_sr: 0,
    
            attr_lsb_latch: false,
            attr_msb_latch: false,
    
            cycles: 0,
        }
    }
}

#[derive(Copy, Clone, Default)]
pub struct Cpu {
    // Common registers
    pub a:   u8,
    pub x:   u8,
    pub y:   u8,
    pub s:   u8,
    pub p_n: bool,
    pub p_v: bool,
    pub p_d: bool,
    pub p_i: bool,
    pub p_z: bool,
    pub p_c: bool,

    // don't actually exist, just trying to match nestest log
    pub p_b5: bool,
    pub p_b4: bool,

    // Internal
    pub pc:             u16,
    pub instruction:    Instruction,
    pub data:           u8,
    pub lower_address:       u8,
    pub upper_address:      u8,
    pub internal_carry_out: bool,
    pub lower_pointer:   u8,
    pub upper_pointer:   u8,
    pub branch_offset: u8,
    pub branching: bool,
    pub instruction_cycle: i8,
    // Interrupts
    pub nmi_interrupt:     bool,
    pub nmi_internal_flag: bool,
    // Helpful things
    pub cycles:            u64,
    pub instruction_count: u64,

    // nestest
    pub trace_opcode: u8,
    pub trace_byte2: u8,
    pub trace_byte3: u8,
    pub trace_imm: u8,
    pub trace_stored_val: u8,

}

// Was trying to avoid methods? This is so convenient though...
impl Cpu {

    pub fn set_upper_pc(&mut self, byte: u8) {
        self.pc &= 0b00000000_11111111;
        self.pc |= (byte as u16) << 8;
    }
    pub fn set_lower_pc(&mut self, byte: u8) {
        self.pc &= 0b11111111_00000000;
        self.pc |= byte as u16;
    }

    pub fn get_p(&self) -> u8 {
        (self.p_n as u8) << 7 | 
        (self.p_v as u8) << 6 | 
        1 << 5 |
        // (self.p_b4 as u8) << 4 |
        (self.p_d as u8) << 3 |
        (self.p_i as u8) << 2 |
        (self.p_z as u8) << 1 |
        (self.p_c as u8)
    }
    pub fn set_p(&mut self, byte: u8) {
        self.p_n = get_bit(byte, 7);
        self.p_v = get_bit(byte, 6);
        // self.p_b5 = get_bit(byte, 5);
        // self.p_b4 = get_bit(byte, 4);
        self.p_d = get_bit(byte, 3);
        self.p_i = get_bit(byte, 2);
        self.p_z = get_bit(byte, 1);
        self.p_c = get_bit(byte, 0);
    }

    pub fn get_address(&self) -> u16 {
        concat_u8(self.upper_address, self.lower_address)
    }
    pub fn get_pointer(&self) -> u16 {
        concat_u8(self.upper_pointer, self.lower_pointer)
    }
}