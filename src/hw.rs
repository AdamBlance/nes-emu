use crate::instr_defs::Instruction;
use crate::util::*;
use crate::apu;

use std::sync::mpsc::Sender;




pub struct Nes {
    // Hardware
    pub cpu:         Cpu,
    pub ppu:         Ppu,
    pub apu:         Apu,
    pub wram:        [u8; 2048],
    pub cartridge:   Cartridge,
    pub controller1: Controller,
    pub controller2: Controller,
    // External
    pub frame:        Vec<u8>,
    // Debugging
    pub ppu_log_toggle: bool,
    pub old_cpu_state:  Cpu,
    pub old_ppu_state:  Ppu,
}

impl Nes {
    pub fn new(cartridge: Cartridge, audio_queue: Sender<f32>) -> Nes {
        Nes { 
            cpu: Cpu::new(),
            ppu: Ppu::new(),
            apu: Apu::new(audio_queue),
            wram: [0; 2048],
            cartridge,
            controller1: Default::default(),
            controller2: Default::default(),

            // RGBA image (4 channels)
            frame: vec![0u8; 256usize * 240 * 4], 

            ppu_log_toggle: false,
            old_cpu_state: Cpu::new(),
            old_ppu_state: Ppu::new(),
        }
    }
}






#[derive(Copy, Clone, Default)]
pub struct Cpu {
    // Registers
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
    pub pc:  u16,
    // Interrupts
    pub nmi_interrupt:     bool,
    pub nmi_internal_flag: bool,
    pub interrupt_request: bool,
    pub irq_internal_flag: bool,
    // Internal
    pub instruction:        Instruction,
    pub instruction_cycle:  i8,
    pub data:               u8,
    pub lower_address:      u8,
    pub upper_address:      u8,
    pub lower_pointer:      u8,
    pub upper_pointer:      u8,
    pub branch_offset:      u8,
    pub branching:          bool,
    pub internal_carry_out: bool,
    pub cycles:             u64,
    // Debugging
    pub instruction_count: u64,
    pub trace_opcode: u8,
    pub trace_byte2: u8,
    pub trace_byte3: u8,
    pub trace_imm: u8,
    pub trace_stored_val: u8,
}

impl Cpu {
    pub fn new() -> Cpu {Default::default()}

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
        (self.p_d as u8) << 3 |
        (self.p_i as u8) << 2 |
        (self.p_z as u8) << 1 |
        (self.p_c as u8)
    }
    pub fn set_p(&mut self, byte: u8) {
        self.p_n = get_bit(byte, 7);
        self.p_v = get_bit(byte, 6);
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






#[derive(Copy, Clone)]
pub struct Ppu {
    // PPUCTRL register
    pub nmi_enable:           bool,
    pub master_slave:         bool,
    pub tall_sprites:         bool,
    pub bg_ptable_select:     bool,
    pub sprite_ptable_select: bool,
    pub increment_select:     bool,
    pub ntable_select:        u8, 
    // PPUMASK register
    pub blue_emphasis:         bool,
    pub green_emphasis:        bool,
    pub red_emphasis:          bool,
    pub show_sprites:          bool,
    pub show_bg:               bool,
    pub show_leftmost_sprites: bool,
    pub show_leftmost_bg:      bool,
    pub greyscale:             bool,
    // PPUSTATUS register
    pub in_vblank:       bool,
    pub sprite_zero_hit: bool,
    pub sprite_overflow: bool,
    // OAMADDR register
    pub oam_addr: u8,
    // Memories
    pub vram:        [u8; 2048],
    pub oam:         [u8; 256],
    pub s_oam:       [u8; 32],
    pub palette_mem: [u8; 32],
    // Rendering counters/indices/flags
    pub t:              u16,
    pub v:              u16,
    pub x:              u8,
    pub w:              bool,
    pub scanline:       i32,
    pub scanline_cycle: i32,
    pub odd_frame:      bool,
    // Temporary background latches
    pub bg_ntable_tmp:     u8,
    pub bg_atable_tmp:     u8,
    pub bg_ptable_lsb_tmp: u8,
    pub bg_ptable_msb_tmp: u8,
    // Background shift registers / latches
    pub bg_ptable_lsb_sr:  u16,
    pub bg_ptable_msb_sr:  u16,
    pub bg_attr_lsb_sr:    u8,
    pub bg_attr_msb_sr:    u8,
    pub bg_attr_lsb_latch: bool,
    pub bg_attr_msb_latch: bool,
    // Sprite shift registers / latches
    pub sprite_ptable_lsb_srs:   [u8; 8],
    pub sprite_ptable_msb_srs:   [u8; 8],
    pub sprite_property_latches: [u8; 8],
    pub sprite_x_counters:       [u8; 8],
    // Sprite/OAM evaluation
    pub in_range_counter:       u8,
    pub sprite_zero_in_soam:    bool,
    pub sprite_zero_in_latches: bool,
    // Misc
    pub ppudata_buffer: u8,
    pub cycles:         u64,
}

impl Ppu {
    fn new() -> Ppu {
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

            vram: [0; 2048],
            oam: [0; 256],
            s_oam: [0; 32],
            palette_mem: [0; 32],

            t: 0,
            v: 0,
            x: 0,
            w: false,
            scanline: 0,
            scanline_cycle: 0,
            odd_frame: false,

            bg_ntable_tmp: 0,
            bg_atable_tmp: 0,
            bg_ptable_lsb_tmp: 0,
            bg_ptable_msb_tmp: 0,

            bg_ptable_lsb_sr: 0,
            bg_ptable_msb_sr: 0,
            bg_attr_lsb_sr: 0,
            bg_attr_msb_sr: 0,
            bg_attr_lsb_latch: false,
            bg_attr_msb_latch: false,

            sprite_ptable_lsb_srs: [0; 8],
            sprite_ptable_msb_srs: [0; 8],
            sprite_property_latches: [0; 8],
            sprite_x_counters: [0; 8],

            in_range_counter: 0,
            sprite_zero_in_soam: false,
            sprite_zero_in_latches: false,

            ppudata_buffer: 0,
            cycles: 0,
        }
    }
}






pub struct Apu {

    pub frame_sequencer_mode_select: bool,
    pub frame_sequencer_counter: u16,
    pub frame_sequencer_interrupt_inhibit: bool,

    pub square1: SquareWave,
    pub square2: SquareWave,

    pub audio_queue: Sender<f32>,

    pub cycles_since_last_sample: u64,
    pub average_cycles_per_sample: f64,
    pub total_sample_count: u64,

}

impl Apu {
    fn new(audio_queue: Sender<f32>) -> Apu {
        Apu {
            frame_sequencer_mode_select: false,
            frame_sequencer_counter: 0,
            frame_sequencer_interrupt_inhibit: false,

            square1: Default::default(),
            square2: Default::default(),

            audio_queue,

            cycles_since_last_sample: 0,
            average_cycles_per_sample: 0.0,
            total_sample_count: 0,
        }
    }
}


#[derive(Copy, Clone, Default)]
pub struct SquareWave {
    pub mute: bool,
    pub sequencer_stage: u8,
    pub timer_init_value: u16,
    pub timer_curr_value: u16,
    pub duty_cycle: u8,
    pub length_counter: u8,
    pub constant_volume: bool,
    pub envelope_loop_and_length_counter_halt: bool,
    pub volume_and_envelope_period: u8,
    pub sweep_enabled: bool,
    pub sweep_counter_init_value: u8,
    pub sweep_counter_curr_value: u8,
    pub sweep_mute_signal: bool,
    pub sweep_negate: bool,
    pub sweep_shift_amount: u8,
    pub sweep_reload_flag: bool,
    pub output: bool,
}

impl SquareWave {
    pub fn set_reg1_from_byte(&mut self, byte: u8) {
        self.duty_cycle = byte >> 6;
        self.envelope_loop_and_length_counter_halt = (byte & 0b0010_0000) > 0;
        self.constant_volume = (byte & 0b0001_0000) > 0;
        self.volume_and_envelope_period = byte & 0b0000_1111;
    }
    pub fn set_reg2_from_byte(&mut self, byte: u8) {
        self.sweep_enabled = (byte & 0b1000_0000) > 0;
        self.sweep_counter_init_value = (byte & 0b0111_0000) >> 4;
        self.sweep_negate = (byte & 0b0000_1000) > 0;
        self.sweep_shift_amount = byte & 0b0000_0111;
    }
    pub fn set_reg3_from_byte(&mut self, byte: u8) {
        self.timer_init_value &= 0b111_0000_0000;
        self.timer_init_value |= byte as u16;
    }
    pub fn set_reg4_from_byte(&mut self, byte: u8) {
        self.timer_init_value &= 0b000_1111_1111;
        self.timer_init_value |= ((byte as u16) & 0b111) << 8;
        self.length_counter = apu::LENGTH_TABLE[((byte & 0b11111_000) >> 3) as usize];
    }
}


pub struct Cartridge {
    pub prg_rom: Vec<u8>,
    pub chr_rom: Vec<u8>,
    pub mapper: u8,
    pub v_mirroring: bool,
}

impl Cartridge {
    pub fn new(ines_data: Vec<u8>) -> Cartridge {
        // Program ROM begins immediately after header
        // Fifth header byte defines size of program ROM in 16kB chunks
        let prg_start: usize = 16;
        let prg_end = prg_start + (ines_data[4] as usize) * 0x4000;

        // Character ROM (sprites, graphics) begins immediately after program ROM
        // Sixth header byte defines size of character ROM in 8kB chunks
        let chr_end = prg_end + (ines_data[5] as usize) * 0x2000;

        Cartridge {
            prg_rom: ines_data[prg_start..prg_end].to_vec(),
            chr_rom: ines_data[prg_end..chr_end].to_vec(),
            mapper: (ines_data[7] & 0xF0) | (ines_data[6] >> 4),
            v_mirroring: (ines_data[6] & 1) > 0,
        }
    }
}






#[derive(Copy, Clone, Default)]
pub struct Controller {
    pub button_state: u8,
    pub shift_register: u8,    
    pub sr_latch_pin: bool,
}
