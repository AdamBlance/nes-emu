use super::channels::*;
use std::sync::mpsc::Sender;
use crate::nes::apu;

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

    // pub audio_queue: Sender<(f32, f32)>,

    // pub target_cycles_per_sample: f64,

    // pub cycles_since_last_sample: u64,
    // pub average_cycles_per_sample: f64,
    pub total_sample_count: u64,

    pub interrupt_request: bool,

}

impl Apu {
    pub fn new() -> Apu {
        Apu {
            frame_sequencer_mode_1: false,
            frame_sequencer_counter: 0,
            frame_sequencer_interrupt_inhibit: true,

            square1: Default::default(),
            square2: Default::default(),

            triangle: Default::default(),

            noise: Default::default(),

            sample: Default::default(),

            // audio_queue,

            // target_cycles_per_sample: CPU_HZ / sample_rate as f64,

            // cycles_since_last_sample: 0,
            // average_cycles_per_sample: 0.0,
            total_sample_count: 0,

            interrupt_request: false,
        }
    }
    pub fn asserting_irq(&self) -> bool {
        self.interrupt_request || self.sample.interrupt_request
        // false
    }

    pub fn get_sample(&self, stereo_pan: f32) -> (f32, f32) {
        assert!((0.0..=1.0).contains(&stereo_pan));
        let sq1_output = apu::square_channel_output(&self.square1) * 1.0;
        let sq2_output = apu::square_channel_output(&self.square2) * 1.0;
        let tri_output = apu::triangle_channel_output(&self.triangle) * 1.0;
        let noise = apu::noise_channel_output(&self.noise) * 1.0;
        let sample = apu::sample_channel_output(&self.sample) * 1.0;

        let epsilon = 0.00001;
        let pos_bias = 1.0 + stereo_pan;
        let neg_bias = 1.0 - stereo_pan;
        let pulse1_out = 95.88 / ((8128.0 / (pos_bias*sq1_output + neg_bias*sq2_output + epsilon)) + 100.0);
        let pulse2_out = 95.88 / ((8128.0 / (pos_bias*sq2_output + neg_bias*sq1_output + epsilon)) + 100.0);
        let other_out = 159.79 / ( ( 1.0 / ((tri_output/8227.0) + (noise/12241.0) + (sample/22638.0) + epsilon) ) + 100.0);

        (pulse1_out + other_out, pulse2_out + other_out)
    }
}