use eframe::egui;
use eframe::egui::{include_image, Image};
use eframe::egui::load::SizedTexture;
use crate::app::App;
use crate::emulator::{setup, Emulator};

impl App {

    fn handle_rom_load(&mut self) {
        if let Some(path) = rfd::FileDialog::new().pick_file()
            && let Ok(rom) = setup::get_rom_from_file(path.as_path()) {
            self.emulator.load_game(rom);
        }
    }


    pub fn define_main_top_panel(&mut self, ui: &mut egui::Ui) {
        egui::Panel::top("top_bar").show(ui, |ui| {
            ui.horizontal_centered(|ui| {
                if ui.button("Load ROM").clicked() { self.handle_rom_load(); }
                ui.toggle_value(&mut self.app_config.show_controller_config, "Controller Config");
            });
        });
    }

    pub fn define_main_bottom_panel(&mut self, ui: &mut egui::Ui) {

        let emulator_speed_drag_value = egui::DragValue::from_get_set(
            crate::emulator::get_set(Emulator::get_speed, Emulator::set_speed, &mut self.emulator)
        )
            .range(0.1..=2.0)
            .speed(0.005),

        egui::Panel::bottom("bottom_bar").show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label("Speed:");
                ui.add(

                );

                ui.add_enabled_ui(self.emulator.get_paused(), |ui| {
                    ui.add(egui::Slider::new(&mut self.emu_state.scrubbing_rate, -3.0..=3.0));
                });



                if ui.add(match self.emulator.get_paused() {
                        true => egui::Button::image(Image::new(include_image!(
                            "../../resources/play_light.png"
                        ))),
                        false => egui::Button::image(Image::new(include_image!(
                            "../../resources/pause_light.png"
                        ))),
                    })
                    .clicked()
                {
                    self.is_paused = !self.is_paused;
                }

                ui.add(
                    egui::Slider::from_get_set(0.0..=1.0, |val| self.emulator.get_set_volume(val))
                        .text("Volume"),
                );
            });
        });
    }

    pub fn define_main_central_panel(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let emulator_screen = ui.add(
                egui::Image::from_texture(SizedTexture::from_handle(&self.emulator.video_output))
                    .shrink_to_fit(),
            );
            let screen_centre_rect = emulator_screen.rect.expand(-200.0);
            if self.scrubbing_rate < 0.0 && self.is_paused {
                ui.put(
                    screen_centre_rect,
                    Image::new(include_image!("../../resources/rewind-svgrepo-com-light.svg")),
                );
            } else if self.scrubbing_rate > 0.0 && self.is_paused {
                ui.put(
                    screen_centre_rect,
                    Image::new(include_image!(
                        "../../resources/fast-forward-svgrepo-com-light.svg"
                    )),
                );
            }
        });
    }
}