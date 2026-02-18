use eframe::egui;

// TODO: sizing should be dynamic to screen size

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([60.0, 60.0]),
        ..Default::default()
    };
    eframe::run_native(
        "pfwm close",
        options,
        Box::new(|_cc| Ok(Box::new(CloseBox {}))),
    )
}
struct CloseBox;
impl eframe::App for CloseBox {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(egui::Color32::from_rgb(140, 0, 0)))
            .show(ctx, |ui| {
                ui.style_mut().interaction.selectable_labels = false;
                ui.centered_and_justified(|ui| {
                    ui.label(
                        egui::RichText::new("X")
                            .size(24.0)
                            .color(egui::Color32::from_rgb(255, 255, 255))
                            .strong(),
                    );
                });
            });
    }
}
