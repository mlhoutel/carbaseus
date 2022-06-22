use crate::app::components::graph::node::*;
use crate::app::state;

pub fn show(state: &mut state::AppState, ui: &mut egui::Ui, ctx: &egui::Context) {
    egui::menu::bar(ui, |ui| {
        egui::widgets::global_dark_light_mode_switch(ui);

        ui.separator();

        ui.menu_button("💻 File", |ui| {
            if ui.button("🔧 Settings").clicked() {
                state.d_settings = !state.d_settings;
            }
            if ui.button("❌ Quit").clicked() {}
        });

        if ui.button("▶ Play").clicked() {
            evaluate_graph(&mut state.graph);
        }

        ui.checkbox(&mut state.auto_compute, "Auto");

        ui.menu_button("📓 Help", |ui| {
            if ui.button("ℹ About").clicked() {
                state.d_about = !state.d_about;
            }
        });

        ui.checkbox(&mut state.d_state, "Debug");
    });

    egui::Window::new("🔧 Settings")
        .open(&mut state.d_settings)
        .vscroll(true)
        .show(ctx, |ui| {
            ctx.settings_ui(ui);
            ui.allocate_space(ui.available_size());
        });

    egui::Window::new("About")
        .open(&mut state.d_about)
        .vscroll(true)
        .show(ctx, |ui| {
            ui.label("about...");
            ui.allocate_space(ui.available_size());
        });
}
