use super::channels::*;
use crate::nes::apu;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct Apu {
    pub frame_sequencer_mode_1: bool,
    pub frame_sequencer_counter: u16,
    pub frame_sequencer_interrupt_inhibit: bool,

    pub square1: Square,
    pub square2: Square,

    pub triangle: Triangle,

    pub noise: Noise,

    pub sample: Sample,

    pub total_sample_count: u64,

    pub interrupt_request: bool,
}

impl Default for Apu {
    fn default() -> Self {
        Self::new()
    }
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

            total_sample_count: 0,

            interrupt_request: false,
        }
    }
    pub fn asserting_irq(&self) -> bool {
        self.interrupt_request || self.sample.interrupt_request
    }

    pub fn get_sample(&self, stereo_pan: f32) -> (f32, f32) {
        assert!((0.0..=1.0).contains(&stereo_pan));
        let sq1_output = apu::square_channel_output(&self.square1) * 1.0;
        let sq2_output = apu::square_channel_output(&self.square2) * 1.0;
        let tri_output = apu::triangle_channel_output(&self.triangle) * 1.0;
        let noise = apu::noise_channel_output(&self.noise) * 0.5; // Noise is way too loud!
        let sample = apu::sample_channel_output(&self.sample) * 1.0;

        let epsilon = 0.00001;
        let pos_bias = 1.0 + stereo_pan;
        let neg_bias = 1.0 - stereo_pan;
        let pulse1_out =
            95.88 / ((8128.0 / (pos_bias * sq1_output + neg_bias * sq2_output + epsilon)) + 100.0);
        let pulse2_out =
            95.88 / ((8128.0 / (pos_bias * sq2_output + neg_bias * sq1_output + epsilon)) + 100.0);
        let other_out = 159.79
            / ((1.0 / ((tri_output / 8227.0) + (noise / 12241.0) + (sample / 22638.0) + epsilon))
                + 100.0);

        (pulse1_out + other_out, pulse2_out + other_out)
    }
}
