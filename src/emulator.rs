use self::nes::Nes;
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

const EXPONENTIAL_MOVING_AVG_BETA: f64 = 0.999;

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
    avg_sample_rate: f64,
    cpu_cycle_at_last_sample: u64,
    cached_cycles_per_sample: f32,
    stereo_pan: f32,
}
pub struct AudioStream {
    pub sender: SyncSender<(f32, f32)>,
    pub sample_rate: f32,
}

struct RunningStats {
    target_speed: f64,
    game_speed: f64,
    paused: bool,
    // Should be used for counting the number of NES frames emulated
    frame: u64,
    frame_progress_at_current_speed: f64,
}

struct RewindData {
    rewind_states: Vec<Nes>,
    rewind_state_index: f32,
}

impl Emulator {
    pub fn new(audio_output: Option<AudioStream>) -> Self {
        let init_cycles_per_sample = match audio_output.as_ref() {
            Some(s) => Self::cycles_per_sample(s.sample_rate, 1.0),
            None => 0.0,
        };

        Emulator {
            nes: None,
            audio_config: audio_output.map(|output| AudioConfig {
                audio_output: output,
                volume: 1.0,
                avg_sample_rate: 1000.0,
                cpu_cycle_at_last_sample: 0,
                cached_cycles_per_sample: init_cycles_per_sample,
                stereo_pan: 0.0,
            }),
            stats: RunningStats {
                game_speed: 1.0,
                target_speed: 1.0,
                paused: false,
                frame: 0,
                frame_progress_at_current_speed: 0.0,
            },
            rewind_data: RewindData {
                rewind_state_index: 0.0,
                rewind_states: Vec::new(),
            },
            nes_frame: Rc::new(RefCell::new(vec![0u8; 256usize * 240 * 4])),
        }
    }

    pub fn load_game(&mut self, rom_config: RomConfig) {
        self.nes = Some(Nes::new(cartridge, Rc::clone(&self.nes_frame)));
    }

    pub fn game_loaded(&self) -> bool {
        self.nes.is_some()
    }

    pub fn get_set_speed(&mut self, speed: Option<f64>) -> f64 {
        Self::get_set(self.get_speed, self.set_speed)(speed)
    }
    pub fn get_speed(&self) -> f64 {
        self.stats.game_speed
    }
    pub fn set_speed(&mut self, speed: f64) {
        assert!(speed > 0.0);
        self.stats.game_speed = speed;
        self.stats.frame_progress_at_current_speed = 0.0;
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

        const NTSC_FRAMERATE: f32 = 60.0;
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
                        Self::cycles_per_sample(stream.sample_rate, self.game_speed as f32);
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

    pub fn run_one_cpu_instruction(&mut self) {
        if let Some(nes) = self.nes.as_mut() {
            loop {
                let end_of_instr = cpu::step_cpu(nes);

                ppu::step_ppu(nes);
                ppu::step_ppu(nes);
                ppu::step_ppu(nes);

                apu::step_apu(nes);

                if end_of_instr {
                    break;
                }
            }
        }
        self.update_prg_rom_debug_cache();
    }

    fn run_to_vblank(&mut self) {
        loop {
            self.try_audio_sample();
            if let Some(nes) = self.nes.as_mut() {
                cpu::step_cpu(nes);

                ppu::step_ppu(nes);
                ppu::step_ppu(nes);
                ppu::step_ppu(nes);

                apu::step_apu(nes);

                if nes.ppu.scanline == 239
                    && (nes.ppu.scanline_cycle >= 257 && nes.ppu.scanline_cycle <= 259)
                {
                    break;
                }
            }
        }
    }

    fn try_audio_sample(&mut self) {
        if !self.paused {
            if let Some(nes) = self.nes.as_mut() {
                let cycle_diff = nes.cpu.cycles - self.cpu_cycle_at_last_sample;

                if (cycle_diff == self.cached_cycles_per_sample.floor() as u64
                    && self.avg_sample_rate > self.cached_cycles_per_sample as f64)
                    || cycle_diff >= self.cached_cycles_per_sample.ceil() as u64
                {
                    // TODO: This should technically be (cached_cycles_per_sample + 1).floor() I think
                    self.do_sample();
                }
            }
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

    fn cycles_per_sample(sample_rate: f32, game_speed: f32) -> f32 {
        const CPU_CYCLES_PER_FRAME: f32 = 29780.5;
        const DEFAULT_FRAMERATE: f32 = 60.0;
        let samples_per_frame = sample_rate / (game_speed * DEFAULT_FRAMERATE);
        CPU_CYCLES_PER_FRAME / samples_per_frame
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

    pub(crate) fn get_set<T>(
        get: fn(&Self) -> T,
        set: fn(&mut Self, T),
    ) -> impl FnMut(Option<T>) -> T {
        |value| {
            if let Some(v) = value {
                set(&mut Self, v)
            }
            get(&Self)
        }
    }
}
