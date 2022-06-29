use crate::cartridge::Cartridge;

use super::channels::*;
use super::units::*;
use std::sync::mpsc::Sender;

const CPU_HZ:   f64 = 1_789_773.0;

const STEP_1: u16 = 3728;
const STEP_2: u16 = 7456;
const STEP_3: u16 = 11185;
const STEP_4: u16 = 14914;
const STEP_5: u16 = 18640;

pub struct Apu {
    pub frame_sequencer_mode_1: bool,
    pub frame_sequencer_counter: u16,
    pub frame_sequencer_interrupt_inhibit: bool,
    pub last_frame_counter_write: u64,
    pub square1: Square,
    pub square2: Square,
    pub triangle: Triangle,
    pub noise: Noise,
    pub sample: Sample,
    pub audio_queue: Sender<(f32, f32)>,
    pub target_cycles_per_sample: f64,
    pub cycles_since_last_sample: u64,
    pub average_cycles_per_sample: f64,
    pub total_sample_count: u64,
    pub interrupt_request: bool,
}

impl Apu {
    pub fn new(audio_queue: Sender<(f32, f32)>, sample_rate: u32) -> Apu {
        Apu {
            frame_sequencer_mode_1: false,
            frame_sequencer_counter: 0,
            frame_sequencer_interrupt_inhibit: true,
            last_frame_counter_write: u64::max_value(),
            square1: Default::default(),
            square2: Default::default(),
            triangle: Default::default(),
            noise: Default::default(),
            sample: Default::default(),
            audio_queue,
            target_cycles_per_sample: CPU_HZ / sample_rate as f64,
            cycles_since_last_sample: 0,
            average_cycles_per_sample: 0.0,
            total_sample_count: 0,
            interrupt_request: false,
        }
    }

    pub fn asserting_irq(&self) -> bool {
        self.interrupt_request || self.sample.interrupt_request
    }

    pub fn step_apu(&mut self, cart: &Box<dyn Cartridge>, cpu_cycles: u64) {
        if cpu_cycles % 2 == 0 {
            self.clock_frame_sequencer();
            self.square1.clock_period_timer();
            self.square2.clock_period_timer();
            self.noise.clock_period_timer();
        }
        self.triangle.clock_period_timer();
        self.sample.clock_period_timer(cart);

        let jitter_offset = if self.last_frame_counter_write % 2 == 0 {3} else {4};
        if self.last_frame_counter_write + jitter_offset == cpu_cycles {
            self.frame_sequencer_counter = 0;
        }
    }

    fn clock_frame_sequencer(&mut self) {
    
        match self.frame_sequencer_counter {
            STEP_1 => {
                self.clock_envelope_generators_and_linear_counter();
            }
            STEP_2 => {
                self.clock_envelope_generators_and_linear_counter();
                self.clock_sweep_units_and_length_counters();
            }
            STEP_3 => {
                self.clock_envelope_generators_and_linear_counter();
            }
            STEP_4 => {
                if !self.frame_sequencer_mode_1 {
                    self.clock_envelope_generators_and_linear_counter();
                    self.clock_sweep_units_and_length_counters();
                    if !self.frame_sequencer_interrupt_inhibit {
                        self.interrupt_request = true;
                    }
                    self.frame_sequencer_counter = 0
                }
            }
            STEP_5 => {
                self.clock_envelope_generators_and_linear_counter();
                self.clock_sweep_units_and_length_counters();
                self.frame_sequencer_counter = 0;
            }
            _ => (),
        }
        self.frame_sequencer_counter += 1;
    }
    
    pub fn clock_envelope_generators_and_linear_counter(&mut self) {
        self.square1.envelope_generator.clock();
        self.square2.envelope_generator.clock();
        self.noise.envelope_generator.clock();
        self.triangle.linear_counter.clock();    
    }
    
    pub fn clock_sweep_units_and_length_counters(&mut self) {
        self.square1.sweep_unit.clock();
        self.square2.sweep_unit.clock();
        
        self.square1.length_counter.clock();
        self.square2.length_counter.clock();
        self.triangle.length_counter.clock();
        self.noise.length_counter.clock();
    }
}