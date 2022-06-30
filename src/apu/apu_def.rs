use super::channels::*;
use std::sync::mpsc::Sender;

const CPU_HZ:   f64 = 1_789_773.0;

pub struct Apu {

    pub frame_sequencer_mode_1: bool,
    pub frame_sequencer_counter: u16,
    pub frame_sequencer_interrupt_inhibit: bool,

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
        // false
    }
}