use crate::app::components::graph::node::*;
use crate::app::state;

pub fn show(state: &mut state::AppState, ui: &mut egui::Ui) {
    ui.label("➕ Add a node");

    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.with_layout(egui::Layout::top_down_justified(egui::Align::LEFT), |ui| {
            full_collapsing("🖼 Input", ui, |ui| {
                ui.with_layout(egui::Layout::top_down_justified(egui::Align::LEFT), |ui| {
                    if ui.button("🔃 Fetcher").clicked() {
                        create_node(state, NodeTemplate::ImageFetcher, egui::pos2(0.0, 0.0));
                    }
                });
            });

            full_collapsing("↔ Conversion", ui, |ui| {
                if ui.button("▓ Gray scales").clicked() {
                    create_node(state, NodeTemplate::GrayScales, egui::pos2(0.0, 0.0));
                }
                if ui.button("✖ Image to RGB Slice").clicked() {
                    create_node(state, NodeTemplate::ImageToSlice, egui::pos2(0.0, 0.0));
                }
                if ui.button("➗ RGB Slice to Image").clicked() {
                    create_node(state, NodeTemplate::SliceToImage, egui::pos2(0.0, 0.0));
                }
            });

            full_collapsing("＃ Processing", ui, |ui| {
                if ui.button("👓 Gaussian blur").clicked() {
                    create_node(state, NodeTemplate::GaussianBlur, egui::pos2(0.0, 0.0));
                }
                if ui.button("〰 Fourier space").clicked() {
                    create_node(state, NodeTemplate::FourierSpace, egui::pos2(0.0, 0.0));
                }
            });
        });
    });
}

fn full_collapsing(label: &str, ui: &mut egui::Ui, add_content: impl FnOnce(&mut egui::Ui)) {
    ui.collapsing(label, |ui| {
        ui.with_layout(egui::Layout::top_down_justified(egui::Align::LEFT), |ui| {
            add_content(ui);
        });
    });
}
