use eframe::egui;

fn main() -> eframe::Result {
    let args: Vec<String> = std::env::args().collect();
    let key = args[1].clone();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([50.0, 50.0]),
        ..Default::default()
    };
    eframe::run_native(
        &format!("key_hint {}", &key),
        options,
        Box::new(|cc| Ok(Box::new(KeyWindow::new(&key, cc)))),
    )
}
struct KeyWindow {
    key: String,
}
impl KeyWindow {
    fn new(key: &str, cc: &eframe::CreationContext<'_>) -> Self {
        cc.egui_ctx.set_pixels_per_point(1.5);
        Self {
            key: key.to_owned().to_uppercase(),
        }
    }
}
impl eframe::App for KeyWindow {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.centered_and_justified(|ui| {
                ui.label(egui::RichText::new(&self.key).strong());
            });
        });
    }
}
