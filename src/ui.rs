use crate::app::App;
use crate::setup;
use crate::widgets::input_select::{InputSelect, InputType};
use eframe::egui;
use eframe::egui::load::SizedTexture;
use eframe::egui::{include_image, Color32, Image, RichText, ViewportBuilder, ViewportId};

impl App {
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

                if ui.button("Controller Config").clicked() {
                    self.show_controller_config = !self.show_controller_config
                }

                ui.separator();

                ui.add_enabled_ui(self.emulator.game_loaded(), |ui| {
                    if ui.button("CPU Debugger").clicked() {
                        self.show_cpu_debugger = !self.show_cpu_debugger;
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
                    egui::DragValue::from_get_set(|val| self.emulator.get_set_speed(val))
                        .clamp_range(0.1..=2.0)
                        .speed(0.005),
                );

                ui.add_enabled_ui(self.is_paused, |ui| {
                    ui.add(egui::Slider::new(&mut self.scrubbing_rate, -3.0..=3.0));
                });

                if ui
                    .add(match self.is_paused {
                        true => egui::Button::image(Image::new(include_image!(
                            "../resources/play_light.png"
                        ))),
                        false => egui::Button::image(Image::new(include_image!(
                            "../resources/pause_light.png"
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
                    Image::new(include_image!("../resources/rewind-svgrepo-com-light.svg")),
                );
            } else if self.scrubbing_rate > 0.0 && self.is_paused {
                ui.put(
                    screen_centre_rect,
                    Image::new(include_image!(
                        "../resources/fast-forward-svgrepo-com-light.svg"
                    )),
                );
            }
        });
    }

    pub fn define_cpu_debugger(&mut self, ctx: &egui::Context) {
        ctx.show_viewport_immediate(
            ViewportId::from_hash_of("cpu_debugger"),
            ViewportBuilder::default().with_inner_size([500.0, 800.0]),
            |ctx, _class| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    let advance_button = ui.button("Step");
                    if advance_button.clicked() {
                        self.emulator.run_one_cpu_instruction();
                    }

                    ui.separator();

                    ui.add_enabled_ui(self.is_paused, |ui| {
                        let mut scroll_builder = egui::ScrollArea::vertical().auto_shrink(false);
                        if advance_button.clicked() {
                            let row = self
                                .emulator
                                .instruction_cache
                                .binary_search_by_key(
                                    &self.emulator.nes.as_ref().unwrap().cpu.pc,
                                    |x| x.opc_addr,
                                )
                                .unwrap_or_else(|i| i);
                            scroll_builder = scroll_builder.vertical_scroll_offset(
                                (10.0 + ui.spacing().item_spacing.y) * (row as f32)
                                    - (ui.available_height() / 2.5),
                            );
                        }
                        scroll_builder.show_rows(
                            ui,
                            10.0,
                            self.emulator.instruction_cache.len(),
                            |ui, row_range| {
                                for row in row_range {
                                    if let Some(nes) = self.emulator.nes.as_ref() {
                                        let debug_instr = self.emulator.instruction_cache[row];
                                        let text =
                                            RichText::new(debug_instr.debug_string()).monospace();
                                        if debug_instr.opc_addr == nes.cpu.pc {
                                            ui.label(text.color(Color32::RED));
                                        } else {
                                            ui.label(text);
                                        }
                                    }
                                }
                            },
                        )
                    });
                });

                if ctx.input(|i| i.viewport().close_requested()) {
                    self.show_cpu_debugger = false;
                }
            },
        )
    }

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
                        ui.image(include_image!("../resources/keyboard-line.svg"));
                        ui.image(include_image!("../resources/gamepad-line.svg"));
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
