use eframe::egui;

// TODO: sizing should be dynamic to screen size

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([60.0, 60.0]),
        ..Default::default()
    };
    eframe::run_native("pfwm pop", options, Box::new(|_cc| Ok(Box::new(PopBox {}))))
}
struct PopBox;
impl eframe::App for PopBox {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(egui::Color32::from_rgb(220, 190, 0)))
            .show(ui, |ui| {
                ui.style_mut().interaction.selectable_labels = false;
                ui.centered_and_justified(|ui| {
                    ui.label(
                        egui::RichText::new("|>")
                            .size(24.0)
                            .color(egui::Color32::from_rgb(0, 0, 0))
                            .strong(),
                    );
                });
            });
    }
}
