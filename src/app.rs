mod input;
pub mod ui;
mod widgets;

use crate::app::widgets::Input;
use crate::emulator::{AudioStream, Emulator};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use eframe::egui::{Color32, ColorImage, TextureFilter, TextureHandle, TextureOptions, Ui};
use eframe::{CreationContext, Frame, Storage, egui};
use gilrs::{Event, EventType, Gilrs};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fs::File;
use std::sync::mpsc;
use std::{fs, iter};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct PersistentData {
    pub volume: f64,
    pub keyboard_input_mapping: (InputMapping, InputMapping),
    pub controllers_input_mapping: HashMap<Uuid, ControllerConfig>,
    pub selected_controllers: (Option<Uuid>, Option<Uuid>),
}

impl Default for PersistentData {
    fn default() -> Self {
        PersistentData {
            volume: 1.0,
            keyboard_input_mapping: (InputMapping::default(), InputMapping::default()),
            controllers_input_mapping: HashMap::new(),
            selected_controllers: (None, None),
        }
    }
}

struct InputConfig {
    // Which controllers
    controller_input_mapping: HashMap<Uuid, ControllerConfig>,
}

struct AppConfig {
    pub show_controller_config: bool,
    pub show_cpu_debugger: bool,
    pub controllers_input_mapping: HashMap<Uuid, ControllerConfig>,
    pub keyboard_input_mapping: (InputMapping, InputMapping),
    pub selected_controllers: (Option<Uuid>, Option<Uuid>),
}

struct EmuState {
    pub held_input: HashSet<Input>,
    pub pressed_input: HashSet<Input>,
    pub scrubbing_rate: f32,
}

pub struct App {
    pub emulator: Emulator,
    pub screen_texture: TextureHandle,
    pub app_config: AppConfig,
    pub emu_state: EmuState,
    pub gilrs: Gilrs,
}

const AXIS_DEADZONE: f32 = 0.1;
const CONFIG_FILE: &str = "config.json";

impl App {
    pub fn new(eframe_creation_ctx: &CreationContext) -> Self {
        let screen_texture = eframe_creation_ctx.egui_ctx.load_texture(
            "emu",
            ColorImage::filled([256, 240], Color32::BLACK),
            TextureOptions {
                magnification: TextureFilter::Nearest,
                minification: TextureFilter::Nearest,
                wrap_mode: Default::default(),
                mipmap_mode: None,
            },
        );

        let audio_stream = match create_audio_stream() {
            Ok(stream) => Some(stream),
            Err(e) => {
                eprintln!("Failed to create stream, emulator will have no audio output: {e}");
                None
            }
        };

        egui_extras::install_image_loaders(&eframe_creation_ctx.egui_ctx);

        let persistent_state = Self::read_from_config_file_or_default();

        let mut emulator = Emulator::new(screen_texture, audio_stream);
        emulator.get_set_volume(Some(persistent_state.volume));

        Self {
            emulator,
            app_config: AppConfig {
                show_cpu_debugger: false,
                show_controller_config: false,
                keyboard_input_mapping: persistent_state.keyboard_input_mapping,
                controllers_input_mapping: persistent_state.controllers_input_mapping,
                selected_controllers: persistent_state.selected_controllers,
            },
            emu_state: EmuState {
                held_input: HashSet::with_capacity(32),
                pressed_input: HashSet::with_capacity(32),
                scrubbing_rate: 0.0,
            },
            gilrs: Gilrs::new().unwrap(),
        }
    }

    pub fn read_from_config_file_or_default() -> PersistentData {
        match fs::read(CONFIG_FILE) {
            Ok(data) => serde_json::from_slice(&data).unwrap_or_else(|_| {
                eprintln!("Failed to decode config file");
                PersistentData::default()
            }),
            Err(_) => {
                eprintln!("Config file does not exist");
                PersistentData::default()
            }
        }
    }

    pub fn write_to_config_file(data: &PersistentData) -> Result<(), &'static str> {
        let file_handle = File::create(CONFIG_FILE).map_err(|err| "Error opening config file")?;
        serde_json::to_writer(file_handle, data).map_err(|err| "Error serializing config data")?;
        Ok(())
    }

    pub fn get_pressed_input(&mut self, ctx: &egui::Context) {
        // TODO: Only process selected controller with UUID
        self.pressed_input.clear();
        for event in iter::from_fn(|| self.gilrs.next_event()) {
            match event {
                Event {
                    event: EventType::ButtonPressed(button, _),
                    ..
                } => {
                    self.held_input.insert(Input::ControllerButton(button));
                    self.pressed_input.insert(Input::ControllerButton(button));
                }
                Event {
                    event: EventType::ButtonReleased(button, _),
                    ..
                } => {
                    self.held_input.remove(&Input::ControllerButton(button));
                }
                Event {
                    event: EventType::AxisChanged(axis, position, _),
                    ..
                } => {
                    if (-1.0..=-AXIS_DEADZONE).contains(&position) {
                        self.held_input.remove(&Input::ControllerAxis(axis, true));
                        self.held_input.insert(Input::ControllerAxis(axis, false));
                    } else if (AXIS_DEADZONE..=1.0).contains(&position) {
                        self.held_input.remove(&Input::ControllerAxis(axis, false));
                        self.held_input.insert(Input::ControllerAxis(axis, true));
                    } else if (-AXIS_DEADZONE..=AXIS_DEADZONE).contains(&position) {
                        self.held_input.remove(&Input::ControllerAxis(axis, false));
                        self.held_input.remove(&Input::ControllerAxis(axis, true));
                    }
                }
                _ => {}
            }
        }

        for (_id, gamepad) in self.gilrs.gamepads() {
            let _ = self.controllers_input_mapping.try_insert(
                Uuid::from_slice(&gamepad.uuid()).unwrap(),
                ControllerConfig {
                    name: gamepad.name().to_owned(),
                    input_mapping: InputMapping::default(),
                },
            );
        }

        ctx.input(|i| {
            for event in i.events.iter() {
                match event {
                    egui::Event::Key {
                        key,
                        pressed: true,
                        repeat: false,
                        ..
                    } => {
                        self.held_input.insert(Input::Key(*key));
                    }
                    egui::Event::Key {
                        key,
                        pressed: false,
                        repeat: false,
                        ..
                    } => {
                        self.held_input.remove(&Input::Key(*key));
                    }
                    _ => {}
                }
            }
        });
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut Ui, frame: &mut Frame) {
        self.define_main_top_panel(ui);
        self.define_main_bottom_panel(ctx);
        self.define_main_central_panel(ctx);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint();

        if ctx.input(|ui| ui.focused) {
            // TODO: Maybe clear event cache when switching focus to another viewport
            self.get_pressed_input(ctx);
        }

        let k_con1 = self.keyboard_input_mapping.0;
        let c_con1 = self
            .selected_controllers
            .0
            .map(|c| self.controllers_input_mapping.get(&c).unwrap());

        // This input stuff DESPERATELY needs a refactor
        // Need to decide on a proper strategy instead of this spaghetti
        // Should have an event list, pressed, held, all accessible

        self.scrubbing_rate = 0.0;

        if self
            .pressed_input
            .contains(&self.keyboard_input_mapping.0.pause)
            || c_con1.is_some_and(|c| self.pressed_input.contains(&c.input_mapping.pause))
        {
            self.is_paused = !self.is_paused;
        }
        if self
            .held_input
            .contains(&self.keyboard_input_mapping.0.rewind)
            || c_con1.is_some_and(|c| self.held_input.contains(&c.input_mapping.rewind))
        {
            self.scrubbing_rate = -1.0;
        } else if self
            .held_input
            .contains(&self.keyboard_input_mapping.0.fast_forward)
            || c_con1.is_some_and(|c| self.held_input.contains(&c.input_mapping.fast_forward))
        {
            self.scrubbing_rate = 1.0;
        }

        let nes_button_state = NesButtonState {
            up: k_con1.up.specified_and(|i| self.held_input.contains(&i))
                || c_con1.is_some_and(|c| self.held_input.contains(&c.input_mapping.up)),
            down: k_con1.down.specified_and(|i| self.held_input.contains(&i))
                || c_con1.is_some_and(|c| self.held_input.contains(&c.input_mapping.down)),
            left: k_con1.left.specified_and(|i| self.held_input.contains(&i))
                || c_con1.is_some_and(|c| self.held_input.contains(&c.input_mapping.left)),
            right: k_con1.right.specified_and(|i| self.held_input.contains(&i))
                || c_con1.is_some_and(|c| self.held_input.contains(&c.input_mapping.right)),
            b: k_con1.b.specified_and(|i| self.held_input.contains(&i))
                || c_con1.is_some_and(|c| self.held_input.contains(&c.input_mapping.b)),
            a: k_con1.a.specified_and(|i| self.held_input.contains(&i))
                || c_con1.is_some_and(|c| self.held_input.contains(&c.input_mapping.a)),
            start: k_con1.start.specified_and(|i| self.held_input.contains(&i))
                || c_con1.is_some_and(|c| self.held_input.contains(&c.input_mapping.start)),
            select: k_con1
                .select
                .specified_and(|i| self.held_input.contains(&i))
                || c_con1.is_some_and(|c| self.held_input.contains(&c.input_mapping.select)),
        };

        self.define_main_top_panel(ctx);
        self.define_main_bottom_panel(ctx);
        self.define_main_central_panel(ctx);

        self.emulator.get_set_pause(Some(self.is_paused));
        self.emulator.scrub_by(self.scrubbing_rate);

        self.emulator.update_controller(1, nes_button_state);
        self.emulator.update(ctx.input(|input| input.time));

        if self.show_cpu_debugger {
            // self.define_cpu_debugger(ctx);
        }
        if self.show_controller_config {
            self.define_controller_config(ctx);
        }
    }

    fn save(&mut self, _storage: &mut dyn Storage) {
        let new_config = PersistentData {
            controllers_input_mapping: self.controllers_input_mapping.clone(),
            keyboard_input_mapping: self.keyboard_input_mapping,
            volume: self.emulator.get_set_volume(None),
            selected_controllers: self.selected_controllers,
        };
        Self::write_to_config_file(&new_config)
            .unwrap_or_else(|err| eprintln!("Couldn't save config state"));
    }
}

pub fn create_audio_stream() -> Result<AudioStream, Box<dyn Error>> {
    let (tx, rx) = mpsc::sync_channel::<(f32, f32)>(4096);
    let device = cpal::default_host()
        .default_output_device()
        .ok_or("No valid audio device")?;
    let config = device.default_output_config()?.config();

    let output = Ok(AudioStream {
        sender: tx,
        sample_rate: config.sample_rate.0 as f32,
    });

    std::thread::spawn(move || {
        let mut prev_sample = (0.0, 0.0);
        let output_stream = device
            .build_output_stream(
                config,
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    // Uses const generics to magically infer that we want &[f32; 2], wow!
                    for [l_channel, r_channel] in data.array_chunks_mut() {
                        (*l_channel, *r_channel) = match rx.recv() {
                            Ok(sample) => {
                                prev_sample = sample;
                                sample
                            }
                            Err(_) => prev_sample,
                        };
                    }
                },
                |_err| panic!("Audio stream encountered an error: {_err}"),
                None,
            )
            .unwrap();
        output_stream.play().unwrap();
        std::thread::park();
    });
    output
}
