use eframe::egui;
use eframe::egui::{Color32, Image, include_image, RichText, ViewportBuilder, ViewportId};
use eframe::egui::load::SizedTexture;
use crate::app::MyApp;
use crate::setup;

impl MyApp {
    pub fn define_main_top_panel(&mut self, ctx: &egui::Context) {
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
    }

    pub fn define_main_bottom_panel(&mut self, ctx: &egui::Context) {
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

                ui.add(
                    egui::Slider::from_get_set(0.0..=1.0, |val| self.emulator.get_set_volume(val) )
                        .text("Volume")
                );

            });
        });
    }

    pub fn define_main_central_panel(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add(
                egui::Image::from_texture(SizedTexture::from_handle(&self.emulator.video_output))
                    .shrink_to_fit(),
            );
        });
    }

    pub fn define_cpu_debugger(&mut self, ctx: &egui::Context) {
        ctx.show_viewport_immediate(
            ViewportId::from_hash_of("cpu_debugger"),
            ViewportBuilder::default().with_inner_size([500.0, 800.0]),
            |ctx, class| {
                egui::CentralPanel::default().show(ctx, |ui| {

                    let advance_button = ui.button("Step");
                    if advance_button.clicked() {
                        self.emulator.run_one_cpu_instruction();
                    }

                    ui.separator();

                    ui.add_enabled_ui(self.emulator.get_set_pause(None), |ui| {
                        let mut scroll_builder = egui::ScrollArea::vertical().auto_shrink(false);
                        if advance_button.clicked() {
                            let row = match self.emulator.instruction_cache.binary_search_by_key(&self.emulator.nes.as_ref().unwrap().cpu.pc, |x| x.opc_addr) {
                                Ok(i) => i,
                                Err(i) => i,
                            };
                            scroll_builder = scroll_builder.vertical_scroll_offset((10.0 + ui.spacing().item_spacing.y) * (row as f32) - (ui.available_height() / 2.5));
                        }

                        let scroll = scroll_builder.show_rows(ui, 10.0, self.emulator.instruction_cache.len(), |ui, row_range| {
                            for row in row_range {
                                if let Some(nes) = self.emulator.nes.as_ref() {
                                    let debug_instr = self.emulator.instruction_cache[row];
                                    let text = RichText::new(debug_instr.debug_string()).monospace();
                                    if debug_instr.opc_addr == nes.cpu.pc {
                                        ui.label(text.color(Color32::RED));
                                    } else {
                                        ui.label(text);
                                    }
                                }
                            }
                        });
                        scroll

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