use self::nes::Nes;
use crate::emulator::nes::cpu;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc::SyncSender;

pub mod nes;
pub mod setup;

/*
    Would be nice to create a state machine diagram to show how the program works when pausing,
    unpausing, opening the debugger, rewinding, scrubbing, stepping forward and backward through
    instructions, etc.
    Will prevent future headaches I think.
    Would also be nice to fix that off-by-one error that happens when you unpause and the game
    lurches a frame.
*/

/*

   Right how does audio sample stuff work?


*/

#[derive(Default)]
pub struct Emulator {
    // The emulator isn't gonna have a NES unless it has a game cartridge
    // The cartridge is hardwired into the address bus so that seems fair
    pub nes: Option<Nes>,
    audio_config: Option<AudioConfig>,
    stats: RunningStats,
    rewind_data: RewindData,
    nes_frame: Rc<RefCell<Vec<u8>>>,
}

struct AudioConfig {
    audio_output: AudioStream,
    volume: f64,
    stereo_pan: f32,
    apu_buffer: Vec<(f32, f32)>,
}
pub struct AudioStream {
    pub sender: SyncSender<(f32, f32)>,
    pub sample_rate: f32,
}

#[derive(Default)]
struct RunningStats {
    game_speed: f64,
    paused: bool,
    frame: u64,
    frame_progress_at_current_speed: f64,
}

#[derive(Default)]
struct RewindData {
    rewind_states: Vec<Nes>,
    rewind_state_index: f32,
}



impl Emulator {
    const NTSC_FRAMERATE: f32 = 60.0;
    const CPU_CYCLES_PER_APU_SAMPLE_RAW: u64 = 10;
    const INTERNAL_APU_SAMPLE_RATE: u32 = 2978;
    const SQUARE_WAVE_CHANNEL_STEREO_PAN: f32 = 0.0;

    /*

       Instead of faffing around with running like 40 CPU cycles and then sampling the apu,
       maybe we just have the apu shit out a new sample into a buffer every X cpu cycles, then
       come in and sample that at 44khz. That's simpler than trying to run until a sample needs
       to be taken. Hard to understand the code. Maybe we just run it until there are

       Yeah how about put a sample into a ringbuffer every time a sample finishes, then

    */

    pub fn new(audio_output: Option<AudioStream>) -> Self {
        Emulator {
            audio_config: audio_output.map(|output| AudioConfig {
                audio_output: output,
                volume: 1.0,
                stereo_pan: Self::SQUARE_WAVE_CHANNEL_STEREO_PAN,
                ..Default::default()
            }),
            stats: RunningStats {
                game_speed: 1.0,
                ..Default::default()
            },
            nes_frame: Rc::new(RefCell::new(vec![0u8; 256usize * 240 * 4])),
            ..Default::default()
        }
    }

    pub fn load_game(&mut self, rom_config: RomConfig) {
        self.nes = Some(Nes::new(cartridge, Rc::clone(&self.nes_frame)));
    }

    pub fn game_loaded(&self) -> bool {
        self.nes.is_some()
    }

    pub fn get_speed(&self) -> f64 {
        self.stats.game_speed
    }
    pub fn set_speed(&mut self, speed: f64) {
        assert!(speed > 0.0);
        self.stats.game_speed = speed;
        self.stats.frame_progress_at_current_speed = 0.0;
    }
    pub fn get_set_speed(&mut self, speed: Option<f64>) -> f64 {
        Self::get_set(Self::get_speed, Self::set_speed, speed)
    }

    // Todo: Check which of these is actually being used
    pub fn toggle_paused(&mut self) {
        if self.stats.paused {
            self.set_paused(false)
        } else {
            self.set_paused(true)
        }
    }

    pub fn get_paused(&self) -> bool {
        self.stats.paused
    }
    pub fn set_paused(&mut self, paused: bool) {
        if self.stats.paused && !paused {
            self.rewind_data
                .rewind_states
                .truncate(self.rewind_data.rewind_state_index as usize)
        }
        self.stats.paused = paused;
    }

    pub fn get_set_volume(&mut self, volume: Option<f64>) -> f64 {
        if let Some(v) = volume {
            assert!(v <= 1.0);
            self.volume = v;
        }
        self.volume
    }

    pub fn scrub_by(&mut self, n_frames: f32) {
        if self.paused && !self.rewind_states.is_empty() && n_frames != 0.0 {
            self.rewind_state_index = (self.rewind_state_index + n_frames)
                .clamp(0.0, (self.rewind_states.len() - 1) as f32);
            self.nes = Some(self.rewind_states[self.rewind_state_index.round() as usize].clone());
            self.run_to_vblank();
        }
    }

    pub fn update(&mut self, time: f64) -> bool {
        if self.nes.is_none() {
            return false;
        }

        let length_of_one_frame = (self.stats.game_speed * NTSC_FRAMERATE).recip();
        let frame_progress_when_update_called = length_of_one_frame / time;

        if frame_progress_when_update_called.ceil() as u64 > self.stats.frame {}

        /*

        How should this be working?
        So eframe will call update at the OS framerate which might be higher than 60.
        So we should get the time update is called and convert it into a frame number
        so if it starts at 0.0 and update is called 0.05 of the way through the frame,
        round up to 1 and if round(frame) > last frame (0.0) then update.

        Problem is if we change the game speed

         */

        let frame_length = (self.stats.game_speed * NTSC_FRAMERATE).recip();
        let frame_number = (time / frame_length) as u64;

        if frame_number > self.frame {
            if self.game_speed != self.target_speed {
                self.game_speed = self.target_speed;

                if let Some(stream) = &self.audio_output {
                    self.cached_cycles_per_sample =
                        Self::cpu_cycles_to_run_before_next_audio_sample_due(
                            stream.sample_rate,
                            self.game_speed as f32,
                        );
                    self.avg_sample_rate = self.cached_cycles_per_sample as f64;
                }
                let frame_length = 1.0 / (self.game_speed * DEFAULT_FRAMERATE);
                let new_frame_number = (self.time / frame_length) as u64;

                self.frame = new_frame_number;
            } else {
                self.frame = frame_number;
            }

            /*

            Right so CHR RAM and PRG RAM can be behind Rc.
            Stuff behind Rc is usually immutable (because otherwise you could potentially have multiple
            mutable references at the same time to the same part of memory).

            Turns out that you can actually mutate the data behind an Rc using make_mut.
            This works in a copy-on-write way, so if multiple Rc exist that point to the same allocation
            make_mut will clone the existing allocation then mutate it.

            So I think all we need to do is make CHR RAM and PRG RAM Rc and modify them with make_mut

            So when we clone the nes state into the rewind state list, both the rewind state and the
            current nes state should be pointing to the same RAM allocations. The moment the current
            nes modifies the ram it will be cloned and will point to a new allocation

             */

            if !self.paused {
                if !self.rewind_states.is_empty() {
                    self.rewind_state_index = (self.rewind_states.len() - 1) as f32;
                }
                let state = self.nes.as_ref().unwrap().clone();
                self.rewind_states.push(state);
                self.run_to_vblank();
            }

            true
        } else {
            false
        }
    }

    fn run_to_vblank(&mut self) {
        if let Some(nes) = self.nes.as_mut() {
            loop {
                cpu::step_cpu(nes);
                ppu::step_ppu(nes);
                ppu::step_ppu(nes);
                ppu::step_ppu(nes);
                apu::step_apu(nes);

                // Take audio sample every n CPU cycles
                if let Some(AudioConfig {
                    stereo_pan,
                    mut apu_buffer,
                    ..
                }) = self.audio_config
                    && nes.cpu.debug.cycles % Self::CPU_CYCLES_PER_APU_SAMPLE_RAW == 0
                {
                    apu_buffer.push(nes.apu.get_sample(stereo_pan))
                }

                if nes.ppu.in_vblank_final_cycles() {
                    break;
                }
            }
        }
    }

    fn resample_apu_audio_and_send_to_audio_stream(&mut self) {
        // Send the samples to the other thread when done with the frame
        if let Some(AudioConfig {
            mut apu_buffer,
            audio_stream
            ..
        }) = self.audio_config
        {
            let resampled_audio = resample(
                &apu_buffer,
                Self::INTERNAL_APU_SAMPLE_RATE,
                sample_rate as u32,
            );
            for sample in resampled_audio.iter() {
                let _ = sender.try_send(*sample);
            }
            apu_buffer.clear();
        }
    }

    pub fn update_controller(&mut self, num: u8, pressed_buttons: NesButtonState) {
        if let Some(nes) = self.nes.as_mut() {
            match num {
                1 => nes.con1.update_button_state(pressed_buttons),
                // 2 => nes.con2.update_button_state(&pressed_buttons),
                _ => panic!("Controller doesn't exist"),
            }
        }
    }

    fn do_sample(&mut self) {
        if let Some(nes) = self.nes.as_mut() {
            let new_sample = nes.apu.get_sample(self.stereo_pan);
            let new_sample_multiplied = (
                new_sample.0 * self.volume as f32,
                new_sample.1 * self.volume as f32,
            );

            let _ = self
                .audio_output
                .as_mut()
                .unwrap()
                .sender
                .try_send(new_sample_multiplied);

            let rolling_average = EXPONENTIAL_MOVING_AVG_BETA * self.avg_sample_rate
                + (1.0 - EXPONENTIAL_MOVING_AVG_BETA)
                    * (nes.cpu.cycles - self.cpu_cycle_at_last_sample) as f64;

            self.cpu_cycle_at_last_sample = nes.cpu.cycles;
            self.avg_sample_rate = rolling_average;
        }
    }

    // This is just to make integration with egui widgets easier
    pub(crate) fn get_set<T>(get: fn(&Self) -> T, set: fn(&mut Self, T), value: Option<T>) -> T {
        if let Some(v) = value {
            set(&mut Self, v)
        }
        get(&Self)
    }
}

/*

   Running the emulator only a handful of CPU cycles until we have to stop to sample the APU
   is probably not good for performance, but I'd need to benchmark to figure that out.
   It just feels like an awkward way of doing things and we have to manage a rolling average
   of the number of CPU cycles executed before polling for a sample.
   Although it's going to introduce latency (will it really?), it would be much simpler to just
   have the APU place a sample onto a buffer every X CPU cycles and then downsample that buffer
   to the sample rate of your audio device.
   Was going to use an audio processing library for this but I really don't think it'll be hard.
   Let's see.

*/

/*
   Not too bad; the ratio of one sample rate to the other tells us how much/many indices to jump
   before taking another sample, then just do lerp.
*/
pub fn resample(
    source_buffer: &[(f32, f32)],
    source_sample_rate: u32,
    output_sample_rate: u32,
) -> Vec<(f32, f32)> {
    // Estimate the sample that would follow to avoid audio clicking etc.
    let [(penultimate_l, penultimate_r), (last_l, last_r)] = source_buffer
        .last_chunk()
        .copied()
        .expect("You can't resample less than two values you twat");
    let future_sample_guess = (
        last_l + (last_l - penultimate_l),
        last_r + (last_r - penultimate_r),
    );

    let in_out_ratio = source_sample_rate as f32 / output_sample_rate as f32;
    let num_samples = (source_buffer.len() as f32 / in_out_ratio) as usize;
    let sample_indices = (0..num_samples).map(|i| i as f32 * in_out_ratio);

    sample_indices
        .map(|sample_index_as_float| {
            let (sample_l, sample_r) = source_buffer[sample_index_as_float as usize];
            let (next_sample_l, next_sample_r) = source_buffer
                .get(sample_index_as_float.ceil() as usize)
                .copied()
                .unwrap_or(future_sample_guess);
            (
                lerp(sample_l, next_sample_l, sample_index_as_float.fract()),
                lerp(sample_r, next_sample_r, sample_index_as_float.fract()),
            )
        })
        .collect()
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn upsample() {
        let source = vec![(0.0, 0.0), (250.0, 250.0), (500.0, 500.0), (750.0, 750.0)];
        let output = resample(&source, 4, 8);

        let expected_output_as_int = vec![0, 125, 250, 375, 500, 625, 750, 875];
        let output_as_int: Vec<i32> = output.iter().copied().map(|x| x.0 as i32).collect();

        assert_eq!(expected_output_as_int, output_as_int);
    }

    #[test]
    fn downsample() {
        let source = vec![
            (0.0, 0.0),
            (250.0, 250.0),
            (500.0, 500.0),
            (250.0, 250.0),
            (0.0, 0.0),
            (-250.0, -250.0),
            (-500.0, -500.0),
            (-250.0, -250.0),
        ];
        let output = resample(&source, 8, 4);

        let expected_output_as_int = vec![0, 500, 0, -500];
        let output_as_int: Vec<i32> = output.iter().copied().map(|x| x.0 as i32).collect();

        assert_eq!(expected_output_as_int, output_as_int);
    }

    #[test]
    #[should_panic]
    fn no_samples() {
        let source = vec![];
        let output = resample(&source, 8, 4);
    }

    #[test]
    fn sine_common_values() {
        let source: Vec<(f32, f32)> = (0..120)
            .map(|x| (x as f32 * 1.5f32).to_radians().sin())
            .collect();
        let output = resample(&source, 120, 9);

        let expected: Vec<f32> = (0..9)
            .map(|x| (x as f32 * 20f32).to_radians().sin())
            .collect();

        assert_eq!(output.len(), expected.len());
        for (exp, out) in expected.iter().zip(output.iter()) {
            assert!((exp - out).abs() < 0.001)
        }
    }
}

/*

   120khz
   44khz

   44/120 = 0.3666...

   1000 * 0.3666 = 367

   1000 * 0.36666

   1000 / 367 = 2.724795 index move per sample

   Just do 120/44 and skip that much index space






*/
