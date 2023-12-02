#![feature(array_chunks)]

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use eframe::egui::load::SizedTexture;
use eframe::egui::{Color32, ColorImage, Event, Image, include_image, Key, TextureFilter, TextureOptions, ViewportBuilder, ViewportId};
use eframe::{egui, CreationContext, Theme};
use std::collections::HashSet;
use std::error::Error;
use std::fs;
use std::path::Path;
use std::sync::mpsc;

use nes_emu_egui::emulator;
use nes_emu_egui::emulator::{AudioStream, Emulator};
use nes_emu_egui::nes::cartridge::Mirroring;
use nes_emu_egui::nes::controller::ButtonState;
use nes_emu_egui::nes::Nes;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: ViewportBuilder::default().with_inner_size([550.0, 567.0]),
        follow_system_theme: false,
        default_theme: Theme::Dark,
        ..Default::default()
    };
    eframe::run_native(
        "nes-emu-egui",
        options,
        Box::new(|cc| Box::new(MyApp::new(cc))),
    )
}

fn get_rom_from_file(path: &Path) -> Result<emulator::RomData, Box<dyn Error>> {
    const INES_HEADER_SIZE: usize = 16;
    const KB: usize = 1024;

    // TODO: Do proper path checks

    let ines_data = fs::read(path)?;

    if ines_data.len() < INES_HEADER_SIZE || !ines_data.starts_with(b"NES\x1A") {
        return Err(format!("{} is not a vaild iNES rom file (header doesn't fit)", path.to_str().unwrap()).into());
    }

    let prg_rom_end = INES_HEADER_SIZE + 16 * KB * ines_data[4] as usize;
    let chr_rom_end = prg_rom_end + 8 * KB * ines_data[5] as usize;

    if (ines_data.len()) < chr_rom_end {
        return Err(format!("{} is not a vaild iNES rom file (file not long enough)", path.to_str().unwrap()).into());
    }

    let chr_rom_is_ram = prg_rom_end == chr_rom_end;

    Ok(emulator::RomData {
        prg_rom: ines_data[INES_HEADER_SIZE..prg_rom_end].to_owned(),
        chr_rom: match chr_rom_is_ram {
            false => ines_data[prg_rom_end..chr_rom_end].to_owned(),
            true => vec![0u8; 0x2000],
        },
        mapper_id: (ines_data[7] & 0xF0) | (ines_data[6] >> 4),
        chr_rom_is_ram,
        mirroring_config: match ines_data[6] & 0b1 {
            1 => Mirroring::Vertical,
            0 => Mirroring::Horizontal,
            _ => unreachable!(),
        },
    })
}

fn create_audio_stream() -> Result<AudioStream, Box<dyn Error>> {
    let (tx, rx) = mpsc::sync_channel::<(f32, f32)>(4096);
    let device = cpal::default_host()
        .default_output_device()
        .ok_or(cpal::BuildStreamError::DeviceNotAvailable)?;
    let config = device.default_output_config()?.config();

    let output = Ok(AudioStream {
        sender: tx,
        sample_rate: config.sample_rate.0 as f32,
    });

    std::thread::spawn(move || {
        let mut prev_sample = (0.0, 0.0);
        let output_stream = device
            .build_output_stream(
                &config,
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

fn new_button_state(
    keys_down: &HashSet<egui::Key>,
    key_mapping: &KeyMapping,
) -> (ButtonState, ButtonState) {
    let con1 = ButtonState {
        up: keys_down.contains(&key_mapping.con1_up),
        down: keys_down.contains(&key_mapping.con1_down),
        left: keys_down.contains(&key_mapping.con1_left),
        right: keys_down.contains(&key_mapping.con1_right),
        a: keys_down.contains(&key_mapping.con1_a),
        b: keys_down.contains(&key_mapping.con1_b),
        start: keys_down.contains(&key_mapping.con1_start),
        select: keys_down.contains(&key_mapping.con1_select),
    };
    let con2 = ButtonState {
        up: keys_down.contains(&key_mapping.con2_up),
        down: keys_down.contains(&key_mapping.con2_down),
        left: keys_down.contains(&key_mapping.con2_left),
        right: keys_down.contains(&key_mapping.con2_right),
        a: keys_down.contains(&key_mapping.con2_a),
        b: keys_down.contains(&key_mapping.con2_b),
        start: keys_down.contains(&key_mapping.con2_start),
        select: keys_down.contains(&key_mapping.con2_select),
    };
    (con1, con2)
}

struct KeyMapping {
    con1_up: Key,
    con1_down: Key,
    con1_left: Key,
    con1_right: Key,
    con1_a: Key,
    con1_b: Key,
    con1_start: Key,
    con1_select: Key,
    con2_up: Key,
    con2_down: Key,
    con2_left: Key,
    con2_right: Key,
    con2_a: Key,
    con2_b: Key,
    con2_start: Key,
    con2_select: Key,
}
impl Default for KeyMapping {
    fn default() -> Self {
        KeyMapping {
            con1_up: Key::W,
            con1_down: Key::R,
            con1_left: Key::A,
            con1_right: Key::S,
            con1_a: Key::E,
            con1_b: Key::N,
            con1_start: Key::J,
            con1_select: Key::G,
            con2_up: Key::Num1,
            con2_down: Key::Num2,
            con2_left: Key::Num3,
            con2_right: Key::Num4,
            con2_a: Key::Num5,
            con2_b: Key::Num6,
            con2_start: Key::Num7,
            con2_select: Key::Num8,
        }
    }
}

struct MyApp {
    emulator: Emulator,
    key_mapping: KeyMapping,
    show_cpu_debugger: bool,
    show_ppu_debugger: bool,
    show_apu_debugger: bool,
}

impl MyApp {
    fn new(eframe_creation_ctx: &CreationContext) -> Self {
        let screen_texture = eframe_creation_ctx.egui_ctx.load_texture(
            "emu",
            ColorImage::new([256, 240], Color32::BLACK),
            TextureOptions {
                magnification: TextureFilter::Nearest,
                minification: TextureFilter::Nearest,
            },
        );

        let audio_stream = match create_audio_stream() {
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
                        if let Ok(rom) = get_rom_from_file(path.as_path()) {
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
