use eframe::egui;

use pick_full_wm::settings;

// TODO: make prettier

fn main() -> eframe::Result {
    // TODO: dynamic sizing
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([550.0, 375.0]),
        centered: true,
        ..Default::default()
    };
    eframe::run_native(
        "pfwm help",
        options,
        Box::new(|cc| Ok(Box::new(HelpWindow::new(cc)))),
    )
}
struct HelpWindow {
    settings: settings::Settings,
}
impl HelpWindow {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        cc.egui_ctx.set_pixels_per_point(1.5);
        Self {
            settings: settings::get_settings(),
        }
    }
}
impl eframe::App for HelpWindow {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if ctx.input(|i| i.pointer.any_click()) {
            std::process::exit(0);
        }
        if ctx.input(|i| {
            i.events
                .iter()
                .any(|e| matches!(e, egui::Event::Key { pressed: true, .. }))
        }) {
            std::process::exit(0);
        }
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("Pick Full WM - Help Menu");
            ui.label("");
            ui.label(&format!(
                "[SUPER] + [{}]: open this help menu",
                &self.settings.bindings.help.to_uppercase()
            ));
            ui.label("[LEFT CLICK]: make side window main");
            ui.label("[RIGHT CLICK]: kill side window");
            ui.label(&format!(
                "[SUPER] + [{}]: close main window",
                &self.settings.bindings.close_main.to_uppercase()
            ));
            ui.label(&format!(
                "[SUPER] + [{}]: fullscreen main window",
                &self.settings.bindings.fullscreen.to_uppercase()
            ));
            ui.label(&format!(
                "[SUPER] + [{}]: swap with side",
                &self.settings.bindings.swaps.join(" / ").to_uppercase()
            ));
            ui.label(&format!(
                "[SUPER] + [{}]: set workspace",
                &self.settings.bindings.workspaces.join(" / ").to_uppercase()
            ));
            ui.label(&format!(
                "[SUPER] + [{}]: cycle monitor index",
                &self.settings.bindings.monitor.to_uppercase()
            ));
            ui.label(&format!(
                "[SUPER] + [SHIFT] + [{}]: move to next monitor",
                &self.settings.bindings.monitor.to_uppercase()
            ));
            for (key, command) in &self.settings.bindings.functions {
                ui.label(&format!(
                    "[SUPER] + [{}]: \"{}\"",
                    &key.to_uppercase(),
                    &command,
                ));
            }
            ui.label("");
            ui.label("Click anywhere in this window (or tap any key) to close.");
        });
    }
}
