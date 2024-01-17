use eframe::egui::{Color32, ColorImage, Key, TextureFilter, TextureOptions};
use eframe::{egui, CreationContext, Storage};
use gilrs::{Event, EventType, Gilrs};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::{fs, iter};
use uuid::Uuid;

use crate::emulator::Emulator;
use crate::setup;
use crate::widgets::input_select::Input;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControllerConfig {
    pub name: String,
    pub input_mapping: InputMapping,
}

#[derive(Debug, Copy, Clone, Default, Serialize, Deserialize)]
pub struct InputMapping {
    pub up: Input,
    pub down: Input,
    pub left: Input,
    pub right: Input,
    pub b: Input,
    pub a: Input,
    pub start: Input,
    pub select: Input,
}

pub struct NesButtonState {
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
    pub b: bool,
    pub a: bool,
    pub start: bool,
    pub select: bool,
}

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

pub struct MyApp {
    pub emulator: Emulator,
    pub show_cpu_debugger: bool,
    pub show_controller_config: bool,
    pub controllers_input_mapping: HashMap<Uuid, ControllerConfig>,
    pub keyboard_input_mapping: (InputMapping, InputMapping),
    pub selected_controllers: (Option<Uuid>, Option<Uuid>),
    pub pressed_input: HashSet<Input>,
    pub gilrs: Gilrs,
}

const AXIS_DEADZONE: f32 = 0.1;
const CONFIG_FILE: &str = "config.cbor";

impl MyApp {
    pub fn new(eframe_creation_ctx: &CreationContext) -> Self {
        let screen_texture = eframe_creation_ctx.egui_ctx.load_texture(
            "emu",
            ColorImage::new([256, 240], Color32::BLACK),
            TextureOptions {
                magnification: TextureFilter::Nearest,
                minification: TextureFilter::Nearest,
            },
        );

        let audio_stream = match setup::create_audio_stream() {
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
            show_cpu_debugger: false,
            show_controller_config: false,
            gilrs: Gilrs::new().unwrap(),
            keyboard_input_mapping: persistent_state.keyboard_input_mapping,
            controllers_input_mapping: persistent_state.controllers_input_mapping,
            selected_controllers: persistent_state.selected_controllers,
            pressed_input: HashSet::with_capacity(32),
        }
    }

    pub fn read_from_config_file_or_default() -> PersistentData {
        match fs::read(CONFIG_FILE) {
            Ok(data) => serde_cbor::from_slice(&data).unwrap_or_else(|_| {
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
        serde_cbor::to_writer(file_handle, data).map_err(|err| "Error serializing config data")?;
        Ok(())
    }

    pub fn get_pressed_input(&mut self, ctx: &egui::Context) {
        for event in iter::from_fn(|| self.gilrs.next_event()) {
            match event {
                Event {
                    event: EventType::ButtonPressed(button, _),
                    ..
                } => {
                    self.pressed_input.insert(Input::ControllerButton(button));
                }
                Event {
                    event: EventType::ButtonReleased(button, _),
                    ..
                } => {
                    self.pressed_input.remove(&Input::ControllerButton(button));
                }
                Event {
                    event: EventType::AxisChanged(axis, position, _),
                    ..
                } if (-1.0..=-AXIS_DEADZONE).contains(&position) => {
                    self.pressed_input
                        .remove(&Input::ControllerAxis(axis, true));
                    self.pressed_input
                        .insert(Input::ControllerAxis(axis, false));
                }
                Event {
                    event: EventType::AxisChanged(axis, position, _),
                    ..
                } if (AXIS_DEADZONE..=1.0).contains(&position) => {
                    self.pressed_input
                        .remove(&Input::ControllerAxis(axis, false));
                    self.pressed_input.insert(Input::ControllerAxis(axis, true));
                }
                Event {
                    event: EventType::AxisChanged(axis, position, _),
                    ..
                } if (-AXIS_DEADZONE..=AXIS_DEADZONE).contains(&position) => {
                    self.pressed_input
                        .remove(&Input::ControllerAxis(axis, false));
                    self.pressed_input
                        .remove(&Input::ControllerAxis(axis, true));
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
                        self.pressed_input.insert(Input::Key(*key));
                    }
                    egui::Event::Key {
                        key,
                        pressed: false,
                        repeat: false,
                        ..
                    } => {
                        self.pressed_input.remove(&Input::Key(*key));
                    }
                    _ => {}
                }
            }
        });
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint();

        if ctx.input(|ui| ui.focused) {
            // TODO: Maybe clear event cache when switching focus to another viewport
            self.get_pressed_input(ctx);
        }

        ctx.input(|ui| {
            if ui.key_pressed(Key::Space) {
                // TODO: get_set isn't a great pattern, change
                let temp = !self.emulator.get_set_pause(None);
                self.emulator.get_set_pause(Some(temp));
            }
            if ui.key_down(Key::L) {
                self.emulator.scrub_by(-1.0);
            }
            if ui.key_down(Key::U) {
                self.emulator.scrub_by(1.0);
            }
        });

        let k_con1 = self.keyboard_input_mapping.0;
        let c_con1 = self
            .selected_controllers
            .0
            .map(|c| self.controllers_input_mapping.get(&c).unwrap());

        let nes_button_state = NesButtonState {
            up: k_con1.up.specified_and(|i| self.pressed_input.contains(&i))
                || c_con1.is_some_and(|c| self.pressed_input.contains(&c.input_mapping.up)),
            down: k_con1
                .down
                .specified_and(|i| self.pressed_input.contains(&i))
                || c_con1.is_some_and(|c| self.pressed_input.contains(&c.input_mapping.down)),
            left: k_con1
                .left
                .specified_and(|i| self.pressed_input.contains(&i))
                || c_con1.is_some_and(|c| self.pressed_input.contains(&c.input_mapping.left)),
            right: k_con1
                .right
                .specified_and(|i| self.pressed_input.contains(&i))
                || c_con1.is_some_and(|c| self.pressed_input.contains(&c.input_mapping.right)),
            b: k_con1.b.specified_and(|i| self.pressed_input.contains(&i))
                || c_con1.is_some_and(|c| self.pressed_input.contains(&c.input_mapping.b)),
            a: k_con1.a.specified_and(|i| self.pressed_input.contains(&i))
                || c_con1.is_some_and(|c| self.pressed_input.contains(&c.input_mapping.a)),
            start: k_con1
                .start
                .specified_and(|i| self.pressed_input.contains(&i))
                || c_con1.is_some_and(|c| self.pressed_input.contains(&c.input_mapping.start)),
            select: k_con1
                .select
                .specified_and(|i| self.pressed_input.contains(&i))
                || c_con1.is_some_and(|c| self.pressed_input.contains(&c.input_mapping.select)),
        };

        self.emulator.update_controller(1, nes_button_state);
        self.emulator.update(ctx.input(|input| input.time));

        self.define_main_top_panel(ctx);
        self.define_main_central_panel(ctx);
        self.define_main_bottom_panel(ctx);

        if self.show_cpu_debugger {
            self.define_cpu_debugger(ctx);
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
