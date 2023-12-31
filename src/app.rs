use eframe::egui::{Color32, ColorImage, Key, TextureFilter, TextureOptions};
use eframe::{egui, CreationContext};
use gilrs::{Event, EventType, Gilrs};
use std::collections::{HashMap, HashSet};
use std::iter;

use crate::emulator::Emulator;
use crate::nes::controller::NesButton;
use crate::setup;
use crate::widgets::input_select::Input;

pub struct MyApp {
    pub emulator: Emulator,
    pub show_cpu_debugger: bool,
    pub show_controller_config: bool,
    pub input_mapping: HashMap<NesButton, Option<Input>>,
    pub input: HashSet<Input>,
    pub gilrs: Gilrs,
}

const AXIS_DEADZONE: f32 = 0.1;

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
                println!("Failed to create stream, emulator will have no audio output: {e}");
                None
            }
        };

        egui_extras::install_image_loaders(&eframe_creation_ctx.egui_ctx);

        let input_mapping = HashMap::from([
            (NesButton::Up, None),
            (NesButton::Down, None),
            (NesButton::Left, None),
            (NesButton::Right, None),
            (NesButton::B, None),
            (NesButton::A, None),
            (NesButton::Start, None),
            (NesButton::Select, None),
        ]);

        Self {
            emulator: Emulator::new(screen_texture, audio_stream),
            show_cpu_debugger: false,
            show_controller_config: false,
            input_mapping,
            gilrs: Gilrs::new().unwrap(),
            input: HashSet::with_capacity(16),
        }
    }

    pub fn get_pressed_input(&mut self, ctx: &egui::Context) {
        for event in iter::from_fn(|| self.gilrs.next_event()) {
            match event {
                Event {
                    event: EventType::ButtonPressed(button, _),
                    ..
                } => {
                    self.input.insert(Input::ControllerButton(button));
                }
                Event {
                    event: EventType::ButtonReleased(button, _),
                    ..
                } => {
                    self.input.remove(&Input::ControllerButton(button));
                }
                Event {
                    event: EventType::AxisChanged(axis, position, _),
                    ..
                } if (-1.0..=-AXIS_DEADZONE).contains(&position) => {
                    self.input.remove(&Input::ControllerAxis(axis, true));
                    self.input.insert(Input::ControllerAxis(axis, false));
                }
                Event {
                    event: EventType::AxisChanged(axis, position, _),
                    ..
                } if (AXIS_DEADZONE..=1.0).contains(&position) => {
                    self.input.remove(&Input::ControllerAxis(axis, false));
                    self.input.insert(Input::ControllerAxis(axis, true));
                }
                Event {
                    event: EventType::AxisChanged(axis, position, _),
                    ..
                } if (-AXIS_DEADZONE..=AXIS_DEADZONE).contains(&position) => {
                    self.input.remove(&Input::ControllerAxis(axis, false));
                    self.input.remove(&Input::ControllerAxis(axis, true));
                }
                _ => {}
            }
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
                        self.input.insert(Input::Key(*key));
                    }
                    egui::Event::Key {
                        key,
                        pressed: false,
                        repeat: false,
                        ..
                    } => {
                        self.input.remove(&Input::Key(*key));
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

        // Should only be 8 items, will be 16 when controller 2 is implemented
        let pressed_button_set: HashSet<NesButton> = self
            .input_mapping
            .iter()
            .filter_map(|(nes_button, mapped_input)| {
                if let Some(m) = mapped_input {
                    if self.input.contains(m) {
                        Some(*nes_button)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();

        self.emulator.update_controller(1, pressed_button_set);

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
}
