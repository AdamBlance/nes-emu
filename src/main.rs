use eframe::egui;
use nes_emu_egui::app::App;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([550.0, 567.0]),
        follow_system_theme: false,
        default_theme: eframe::Theme::Dark,
        persist_window: true,
        ..Default::default()
    };
    eframe::run_native(
        "nes-emu-egui",
        options,
        Box::new(|cc| Box::new(App::new(cc))),
    )
}
