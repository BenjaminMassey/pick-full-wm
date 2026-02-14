use eframe::egui;

use pick_full_wm::settings;

const HEADER_SIZE: f32 = 24.0;
const STANDARD_SIZE: f32 = 16.0;
const FOOTER_SIZE: f32 = 20.0;
// TODO: dynamic scaling, probably calling some combo of xrandr scale and res

const BG_COLOR: egui::Color32 = egui::Color32::from_rgb(25, 25, 30);
const STANDARD_COLOR: egui::Color32 = egui::Color32::from_rgb(200, 200, 200);
const HEADER_COLOR: egui::Color32 = egui::Color32::from_rgb(255, 255, 255);
const SUPER_COLOR: egui::Color32 = egui::Color32::from_rgb(255, 255, 0);
const KEY_COLOR: egui::Color32 = egui::Color32::from_rgb(0, 255, 255);
const FOOTER_COLOR: egui::Color32 = egui::Color32::from_rgb(255, 255, 255);

fn main() -> eframe::Result {
    let window = HelpWindow::new();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([
            550.0,
            290.0 + (window.settings.bindings.functions.len() as f32 * STANDARD_SIZE * 1.55),
        ]),
        centered: true,
        ..Default::default()
    };
    eframe::run_native("pfwm help", options, Box::new(|_cc| Ok(Box::new(window))))
}
struct HelpWindow {
    settings: settings::Settings,
}
impl HelpWindow {
    fn new() -> Self {
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
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(BG_COLOR))
            .show(ctx, |ui| {
                ui.label(
                    egui::RichText::new("Pick Full WM - Help Menu")
                        .size(HEADER_SIZE)
                        .strong()
                        .color(HEADER_COLOR),
                );
                ui.label("");
                super_label(
                    ui,
                    &self.settings.bindings.help.to_uppercase(),
                    "open this help menu",
                );
                mix_color_label(
                    ui,
                    &vec![
                        ("[", STANDARD_COLOR),
                        ("LEFT CLICK", KEY_COLOR),
                        ("]: make side window main", STANDARD_COLOR),
                    ],
                );
                mix_color_label(
                    ui,
                    &vec![
                        ("[", STANDARD_COLOR),
                        ("RIGHT CLICK", KEY_COLOR),
                        ("]: kill side window", STANDARD_COLOR),
                    ],
                );
                super_label(
                    ui,
                    &self.settings.bindings.close_main.to_uppercase(),
                    "close main window",
                );
                super_label(
                    ui,
                    &self.settings.bindings.fullscreen.to_uppercase(),
                    "fullscreen main window",
                );
                super_label(
                    ui,
                    &self.settings.bindings.swaps.join("  ").to_uppercase(),
                    "swap with side",
                );
                super_label(
                    ui,
                    &self.settings.bindings.workspaces.join("  ").to_uppercase(),
                    "set workspace",
                );
                mix_color_label(
                    ui,
                    &vec![
                        ("[", STANDARD_COLOR),
                        ("SUPER", SUPER_COLOR),
                        ("] + [", STANDARD_COLOR),
                        ("SHIFT", SUPER_COLOR),
                        ("] + [", STANDARD_COLOR),
                        (
                            &self.settings.bindings.workspaces.join("  ").to_uppercase(),
                            KEY_COLOR,
                        ),
                        (&format!("]: move main to workspace"), STANDARD_COLOR),
                    ],
                );
                super_label(
                    ui,
                    &self.settings.bindings.monitor.to_uppercase(),
                    "cycle monitor index",
                );
                mix_color_label(
                    ui,
                    &vec![
                        ("[", STANDARD_COLOR),
                        ("SUPER", SUPER_COLOR),
                        ("] + [", STANDARD_COLOR),
                        ("SHIFT", SUPER_COLOR),
                        ("] + [", STANDARD_COLOR),
                        (&self.settings.bindings.monitor.to_uppercase(), KEY_COLOR),
                        (&format!("]: move main to next monitor"), STANDARD_COLOR),
                    ],
                );
                for (key, command) in &self.settings.bindings.functions {
                    super_label(ui, &key.to_uppercase(), &format!("\"{}\"", command));
                }
                ui.label("");
                ui.label(
                    egui::RichText::new("Click anywhere in this window (or tap any key) to close.")
                        .size(FOOTER_SIZE)
                        .strong()
                        .color(FOOTER_COLOR),
                );
            });
    }
}

fn super_label(ui: &mut egui::Ui, key: &str, action: &str) {
    mix_color_label(
        ui,
        &vec![
            ("[", STANDARD_COLOR),
            ("SUPER", SUPER_COLOR),
            ("] + [", STANDARD_COLOR),
            (key, KEY_COLOR),
            (&format!("]: {}", action), STANDARD_COLOR),
        ],
    );
}

fn mix_color_label(ui: &mut egui::Ui, items: &[(&str, egui::Color32)]) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        for item in items {
            ui.label(
                egui::RichText::new(item.0)
                    .color(item.1)
                    .size(STANDARD_SIZE),
            );
        }
    });
}
