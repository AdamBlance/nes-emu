mod main_panel;

use crate::app::App;
use crate::emulator::{Emulator, get_set, setup};
use eframe::egui;
use eframe::egui::load::SizedTexture;
use eframe::egui::{Image, ViewportBuilder, ViewportId, include_image};

impl App {
    pub fn define_controller_config(&mut self, ctx: &egui::Context) {
        ctx.show_viewport_immediate(
            ViewportId::from_hash_of("controller"),
            ViewportBuilder::default().with_inner_size([500.0, 500.0]),
            |ctx, _class| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    egui::Grid::new("cool-grid").show(ui, |ui| {
                        if ctx.input(|ui| ui.focused) {
                            self.get_pressed_input(ctx);
                        }

                        let maybe_input = self.held_input.iter().next().copied();

                        ui.label("");
                        ui.image(include_image!("../../resources/keyboard-line.svg"));
                        ui.image(include_image!("../../resources/gamepad-line.svg"));
                        ui.end_row();

                        ui.label("UP:");
                        ui.add(InputSelect::new(
                            maybe_input,
                            Some(&mut self.keyboard_input_mapping.0.up),
                            "con1-up-key",
                            InputType::Keyboard,
                        ));
                        ui.add_enabled(
                            self.selected_controllers.0.is_some(),
                            InputSelect::new(
                                maybe_input,
                                self.selected_controllers.0.map(|id| {
                                    &mut self
                                        .controllers_input_mapping
                                        .get_mut(&id)
                                        .unwrap()
                                        .input_mapping
                                        .up
                                }),
                                "con1-up-gamepad",
                                InputType::Controller,
                            ),
                        );
                        ui.end_row();

                        ui.label("DOWN:");
                        ui.add(InputSelect::new(
                            maybe_input,
                            Some(&mut self.keyboard_input_mapping.0.down),
                            "con1-down-key",
                            InputType::Keyboard,
                        ));
                        ui.add_enabled(
                            self.selected_controllers.0.is_some(),
                            InputSelect::new(
                                maybe_input,
                                self.selected_controllers.0.map(|id| {
                                    &mut self
                                        .controllers_input_mapping
                                        .get_mut(&id)
                                        .unwrap()
                                        .input_mapping
                                        .down
                                }),
                                "con1-down-gamepad",
                                InputType::Controller,
                            ),
                        );
                        ui.end_row();

                        ui.label("LEFT:");
                        ui.add(InputSelect::new(
                            maybe_input,
                            Some(&mut self.keyboard_input_mapping.0.left),
                            "con1-left-key",
                            InputType::Keyboard,
                        ));
                        ui.add_enabled(
                            self.selected_controllers.0.is_some(),
                            InputSelect::new(
                                maybe_input,
                                self.selected_controllers.0.map(|id| {
                                    &mut self
                                        .controllers_input_mapping
                                        .get_mut(&id)
                                        .unwrap()
                                        .input_mapping
                                        .left
                                }),
                                "con1-left-gamepad",
                                InputType::Controller,
                            ),
                        );
                        ui.end_row();

                        ui.label("RIGHT:");
                        ui.add(InputSelect::new(
                            maybe_input,
                            Some(&mut self.keyboard_input_mapping.0.right),
                            "con1-right-key",
                            InputType::Keyboard,
                        ));
                        ui.add_enabled(
                            self.selected_controllers.0.is_some(),
                            InputSelect::new(
                                maybe_input,
                                self.selected_controllers.0.map(|id| {
                                    &mut self
                                        .controllers_input_mapping
                                        .get_mut(&id)
                                        .unwrap()
                                        .input_mapping
                                        .right
                                }),
                                "con1-right-gamepad",
                                InputType::Controller,
                            ),
                        );
                        ui.end_row();

                        ui.label("B:");
                        ui.add(InputSelect::new(
                            maybe_input,
                            Some(&mut self.keyboard_input_mapping.0.b),
                            "con1-b-key",
                            InputType::Keyboard,
                        ));
                        ui.add_enabled(
                            self.selected_controllers.0.is_some(),
                            InputSelect::new(
                                maybe_input,
                                self.selected_controllers.0.map(|id| {
                                    &mut self
                                        .controllers_input_mapping
                                        .get_mut(&id)
                                        .unwrap()
                                        .input_mapping
                                        .b
                                }),
                                "con1-b-gamepad",
                                InputType::Controller,
                            ),
                        );
                        ui.end_row();

                        ui.label("A:");
                        ui.add(InputSelect::new(
                            maybe_input,
                            Some(&mut self.keyboard_input_mapping.0.a),
                            "con1-a-key",
                            InputType::Keyboard,
                        ));
                        ui.add_enabled(
                            self.selected_controllers.0.is_some(),
                            InputSelect::new(
                                maybe_input,
                                self.selected_controllers.0.map(|id| {
                                    &mut self
                                        .controllers_input_mapping
                                        .get_mut(&id)
                                        .unwrap()
                                        .input_mapping
                                        .a
                                }),
                                "con1-a-gamepad",
                                InputType::Controller,
                            ),
                        );
                        ui.end_row();

                        ui.label("SELECT:");
                        ui.add(InputSelect::new(
                            maybe_input,
                            Some(&mut self.keyboard_input_mapping.0.select),
                            "con1-select-key",
                            InputType::Keyboard,
                        ));

                        ui.add_enabled(
                            self.selected_controllers.0.is_some(),
                            InputSelect::new(
                                maybe_input,
                                self.selected_controllers.0.map(|id| {
                                    &mut self
                                        .controllers_input_mapping
                                        .get_mut(&id)
                                        .unwrap()
                                        .input_mapping
                                        .select
                                }),
                                "con1-select-gamepad",
                                InputType::Controller,
                            ),
                        );

                        ui.end_row();

                        ui.label("START:");
                        ui.add(InputSelect::new(
                            maybe_input,
                            Some(&mut self.keyboard_input_mapping.0.start),
                            "con1-start-key",
                            InputType::Keyboard,
                        ));
                        ui.add_enabled(
                            self.selected_controllers.0.is_some(),
                            InputSelect::new(
                                maybe_input,
                                self.selected_controllers.0.map(|id| {
                                    &mut self
                                        .controllers_input_mapping
                                        .get_mut(&id)
                                        .unwrap()
                                        .input_mapping
                                        .start
                                }),
                                "con1-start-gamepad",
                                InputType::Controller,
                            ),
                        );
                        ui.end_row();

                        ui.separator();
                        ui.separator();
                        ui.separator();
                        ui.end_row();

                        ui.label("Pause:");
                        ui.add(InputSelect::new(
                            maybe_input,
                            Some(&mut self.keyboard_input_mapping.0.pause),
                            "pause-key",
                            InputType::Keyboard,
                        ));
                        ui.add_enabled(
                            self.selected_controllers.0.is_some(),
                            InputSelect::new(
                                maybe_input,
                                self.selected_controllers.0.map(|id| {
                                    &mut self
                                        .controllers_input_mapping
                                        .get_mut(&id)
                                        .unwrap()
                                        .input_mapping
                                        .pause
                                }),
                                "pause-gamepad",
                                InputType::Controller,
                            ),
                        );
                        ui.end_row();

                        ui.label("Rewind:");
                        ui.add(InputSelect::new(
                            maybe_input,
                            Some(&mut self.keyboard_input_mapping.0.rewind),
                            "rewind-key",
                            InputType::Keyboard,
                        ));
                        ui.add_enabled(
                            self.selected_controllers.0.is_some(),
                            InputSelect::new(
                                maybe_input,
                                self.selected_controllers.0.map(|id| {
                                    &mut self
                                        .controllers_input_mapping
                                        .get_mut(&id)
                                        .unwrap()
                                        .input_mapping
                                        .rewind
                                }),
                                "rewind-gamepad",
                                InputType::Controller,
                            ),
                        );
                        ui.end_row();

                        ui.label("Fast forward:");
                        ui.add(InputSelect::new(
                            maybe_input,
                            Some(&mut self.keyboard_input_mapping.0.fast_forward),
                            "fast-forward-key",
                            InputType::Keyboard,
                        ));
                        ui.add_enabled(
                            self.selected_controllers.0.is_some(),
                            InputSelect::new(
                                maybe_input,
                                self.selected_controllers.0.map(|id| {
                                    &mut self
                                        .controllers_input_mapping
                                        .get_mut(&id)
                                        .unwrap()
                                        .input_mapping
                                        .fast_forward
                                }),
                                "fast-forward-gamepad",
                                InputType::Controller,
                            ),
                        );
                        ui.end_row();

                        ui.label("");
                        ui.label("");

                        egui::ComboBox::from_id_source("controller_select")
                            .selected_text(self.selected_controllers.0.map_or("None", |con| {
                                self.controllers_input_mapping
                                    .get(&con)
                                    .unwrap()
                                    .name
                                    .as_str()
                            }))
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut self.selected_controllers.0, None, "None");
                                for (uuid, controller_config) in
                                    self.controllers_input_mapping.iter()
                                {
                                    ui.horizontal(|ui| {
                                        ui.selectable_value(
                                            &mut self.selected_controllers.0,
                                            Some(*uuid),
                                            &controller_config.name,
                                        );
                                    });
                                }
                            });

                        ui.end_row();
                    });
                });

                if ctx.input(|i| i.viewport().close_requested()) {
                    self.show_controller_config = false;
                }
            },
        )
    }
}
