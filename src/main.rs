use eframe::egui;

mod app;
pub mod can;
pub mod miu_state;

fn main() {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "miu com",
        options,
        Box::new(|cc| {
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            Box::new(app::MiuComApp::new(cc.egui_ctx.clone()))
        }),
    );
}
