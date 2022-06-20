use super::channels::*;
use std::sync::mpsc::Sender;

pub struct Apu {

    pub frame_sequencer_mode_select: bool,
    pub frame_sequencer_counter: u16,
    pub frame_sequencer_interrupt_inhibit: bool,

    pub square1: Square,
    pub square2: Square,

    pub triangle: Triangle,

    pub noise: Noise,

    pub sample: Sample,

    pub audio_queue: Sender<(f32, f32)>,

    pub cycles_since_last_sample: u64,
    pub average_cycles_per_sample: f64,
    pub total_sample_count: u64,

}

impl Apu {
    pub fn new(audio_queue: Sender<(f32, f32)>) -> Apu {
        Apu {
            frame_sequencer_mode_select: false,
            frame_sequencer_counter: 0,
            frame_sequencer_interrupt_inhibit: false,

            square1: Default::default(),
            square2: Default::default(),

            triangle: Default::default(),

            noise: Default::default(),

            sample: Default::default(),

            audio_queue,

            cycles_since_last_sample: 0,
            average_cycles_per_sample: 0.0,
            total_sample_count: 0,
        }
    }
}