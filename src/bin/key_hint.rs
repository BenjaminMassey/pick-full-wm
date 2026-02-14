use eframe::egui;

// TODO: sizing should be dynamic to screen size

fn main() -> eframe::Result {
    let args: Vec<String> = std::env::args().collect();
    let key = args[1].clone();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([60.0, 60.0]),
        ..Default::default()
    };
    eframe::run_native(
        &format!("key_hint {}", &key),
        options,
        Box::new(|_cc| Ok(Box::new(KeyWindow::new(&key)))),
    )
}
struct KeyWindow {
    key: String,
}
impl KeyWindow {
    fn new(key: &str) -> Self {
        Self {
            key: key.to_owned().to_uppercase(),
        }
    }
}
impl eframe::App for KeyWindow {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.centered_and_justified(|ui| {
                ui.label(egui::RichText::new(&self.key).size(24.0).strong());
            });
        });
    }
}
