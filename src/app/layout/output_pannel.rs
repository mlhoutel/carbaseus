use egui_extras::RetainedImage;
use std::collections::HashMap;

use crate::app::components::display;
use crate::app::components::graph::node::{self, *};
use crate::app::math::image::slice_to_image;
use crate::app::state;

pub fn show(state: &mut state::AppState, ui: &mut egui::Ui) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        if let Some(selected_id) = state.selected_node.node_id {
            let selected_node = state
                .graph
                .graph
                .nodes
                .iter()
                .find(|&(node_id, _node)| node_id == selected_id);

            // Check if the node exist in the graph
            if let Some((_, node)) = selected_node {
                let outputs = node.output_ids();

                // Try to find a node output that correspond to an image
                let find_image = outputs
                    .into_iter()
                    .find(|id| state.graph.user_state.outputs_images.contains_key(id));

                // If the image was found, we update the value
                if let Some(output_id) = find_image {
                    show_selected(state, ui, output_id);
                }
            }
        } else {
            show_not_selected(ui);
        }
    });
}

fn show_not_selected(ui: &mut egui::Ui) {
    ui.allocate_ui_with_layout(
        ui.available_size(),
        egui::Layout::centered_and_justified(egui::Direction::TopDown),
        |ui| ui.label("Select a node to preview it"),
    );
}

fn show_selected(state: &mut state::AppState, ui: &mut egui::Ui, output_id: OutputId) {
    // Extract the color image from the results
    if state.selected_node.color_image.is_none() {
        state.selected_node.color_image = match state.graph.user_state.outputs_cache.get(&output_id)
        {
            Some(node::ValueType::Image { value }) => Some(value.clone()),
            Some(node::ValueType::Slice { value }) => Some(slice_to_image(value)),
            _ => None,
        };

        // If the color image was just initialized
        if let Some(color_image) = &state.selected_node.color_image {
            // We compute and store the image spectrum
            for px in &color_image.pixels {
                for i in 0..3 {
                    let intensity = px[i];

                    if intensity == 0 {
                        continue; // dont take into account the intensity 0
                    }

                    if let Some(&count) = &mut state.selected_node.spectrum[i].get(&intensity) {
                        state.selected_node.spectrum[i].insert(intensity, count + 1);
                    } else {
                        state.selected_node.spectrum[i].insert(intensity, 1);
                    }
                }
            }
        }
    }

    // Compute a new retained image
    if state.selected_node.retained_image.is_none() {
        if let Some(color_image) = &state.selected_node.color_image {
            state.selected_node.retained_image = Some(RetainedImage::from_color_image(
                "selected node retained image",
                (*color_image).clone(),
            ))
        }
    }

    if let Some(retained_image) = &state.selected_node.retained_image {
        if let Some(_color_image) = &state.selected_node.color_image {
            ui.horizontal(|ui| {
                ui.selectable_value(
                    &mut state.o_pannel,
                    state::OutputPanel::Image,
                    "Image display",
                );
                ui.selectable_value(
                    &mut state.o_pannel,
                    state::OutputPanel::Spectrum,
                    "Spectrum graph",
                );
            });

            ui.separator();

            match state.o_pannel {
                state::OutputPanel::Image => show_image_display(ui, retained_image),
                state::OutputPanel::Spectrum => {
                    show_image_spectrum(ui, &state.selected_node.spectrum)
                }
            }
        }
    }
}

fn show_image_display(ui: &mut egui::Ui, image: &RetainedImage) {
    display::image_frame::show(ui, image);
}

fn show_image_spectrum(ui: &mut egui::Ui, spectrum: &[HashMap<u8, usize>]) {
    display::image_spectrum::show(ui, spectrum);
}
