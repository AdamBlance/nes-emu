#![feature(iter_advance_by)]
#![feature(map_try_insert)]
#![allow(clippy::unusual_byte_groupings)]
#![feature(try_blocks)]

mod app;
mod emulator;

use eframe::egui;
use crate::app::App;


fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([550.0, 567.0]),
        persist_window: true,
        ..Default::default()
    };
    eframe::run_native(
        "nes-emu-egui",
        options,
        Box::new(|cc| Ok(Box::new(App::new(cc)))),
    )
}
