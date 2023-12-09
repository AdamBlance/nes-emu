use eframe::{CreationContext, egui};
use eframe::egui::{Color32, ColorImage, Image, include_image, Key, TextureFilter, TextureOptions, ViewportBuilder, ViewportId};
use eframe::egui::load::SizedTexture;
use crate::emulator::Emulator;
use crate::input::{KeyMapping, new_button_state};
use crate::setup;

pub struct MyApp {
    emulator: Emulator,
    key_mapping: KeyMapping,
    show_cpu_debugger: bool,
    show_ppu_debugger: bool,
    show_apu_debugger: bool,
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
            show_ppu_debugger: false,
            show_apu_debugger: false,
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

        egui::TopBottomPanel::top("my_panel").show(ctx, |ui| {
            ui.horizontal_centered(|ui| {
                if ui.button("Load ROM").clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_file() {
                        if let Ok(rom) = setup::get_rom_from_file(path.as_path()) {
                            self.emulator.load_game(rom);
                        }
                    }
                }
                ui.separator();

                ui.button("Save");
                ui.button("Load");
                ui.button("Load...");
                ui.separator();

                ui.add_enabled_ui(self.emulator.game_loaded(), |ui| {
                    if ui.button("CPU Debugger").clicked() {
                        self.show_cpu_debugger = !self.show_cpu_debugger;
                    }
                    if ui.button("PPU Debugger").clicked() {
                        self.show_ppu_debugger = !self.show_ppu_debugger;
                    }
                    if ui.button("CPU Debugger").clicked() {
                        self.show_apu_debugger = !self.show_apu_debugger;
                    }
                });

            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add(
                egui::Image::from_texture(SizedTexture::from_handle(&self.emulator.video_output))
                    .shrink_to_fit(),
            );

        });

        egui::TopBottomPanel::bottom("bottom?").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Speed:");
                ui.add(
                    egui::DragValue::from_get_set(|val| self.emulator.get_set_speed(val) )
                        .clamp_range(0.1..=2.0)
                        .speed(0.005)

                );

                ui.add_enabled_ui(self.emulator.get_set_pause(None), |ui| {
                    let mut rewind_speed_offset = 0.0;
                    ui.add(egui::Slider::new(&mut rewind_speed_offset, -3.0..=3.0));
                    self.emulator.scrub_by(rewind_speed_offset);
                });


                if ui.add(
                    match self.emulator.get_set_pause(None) {
                        true => egui::Button::image(Image::new(include_image!("../resources/play_light.png"))),
                        false => egui::Button::image(Image::new(include_image!("../resources/pause_light.png"))),
                    }
                ).clicked() {
                    // TODO: get_set isn't a great pattern, change
                    let temp = !self.emulator.get_set_pause(None);
                    self.emulator.get_set_pause(Some(temp));
                }

            });
        });

        if self.show_cpu_debugger {
            ctx.show_viewport_immediate(
                ViewportId::from_hash_of("cpu_debugger"),
                ViewportBuilder::default().with_inner_size([300.0, 300.0]),
                |ctx, class| {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        ui.label("Hello from immediate viewport");

                        egui::ScrollArea::vertical().show_rows(ui, 10.0, 100, |ui, row_range| {
                            for row in row_range {
                                let text = format!("Row {}/{}", row + 1, 100);
                                ui.label(text);
                            }
                        });

                    });

                    if ctx.input(|i| i.viewport().close_requested()) {
                        // Tell parent viewport that we should not show next frame:
                        self.show_cpu_debugger = false;
                    }
                },
            )
        }
    }
}
