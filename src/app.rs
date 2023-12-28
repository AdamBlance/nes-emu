use eframe::{CreationContext, egui};
use eframe::egui::{Color32, ColorImage, Key, TextureFilter, TextureOptions};
use gilrs::Gilrs;

use crate::emulator::Emulator;
use crate::input::{ControllerThingy, KeyMapping, new_button_state};
use crate::setup;

pub struct MyApp {
    pub emulator: Emulator,
    pub key_mapping: KeyMapping,
    pub show_cpu_debugger: bool,
    pub show_controller_config: bool,
    pub input_select_states: ControllerThingy,
    pub gilrs: Gilrs,
}

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

        Self {
            emulator: Emulator::new(screen_texture, audio_stream),
            key_mapping: KeyMapping::default(),
            show_cpu_debugger: false,
            show_controller_config: false,
            input_select_states: ControllerThingy::default(),
            gilrs: Gilrs::new().unwrap(),
        }
    }
}

impl eframe::App for MyApp {

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint();

        let (con1, con2) = ctx.input(|input| new_button_state(&input.keys_down, &self.key_mapping));
        self.emulator.update_controller(1, con1);
        self.emulator.update_controller(2, con2);

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
