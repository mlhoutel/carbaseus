/*
use crate::app::state;
use egui::epaint::ColorImage;

use crate::app::components::display;

pub fn show(state: &mut state::AppState, ui: &mut egui::Ui, ctx: &egui::Context) {

   // process_image(state);
    ui_image(ui, &state.output_image);
}


pub fn ui_image(ui: &mut egui::Ui, image: &Option<ColorImage>) {
    if let Some(image) = image {
        let size = image.size;
        let len = image.pixels.len();

        ui.horizontal(|ui| {
            ui.label(format!("size: {:?}", size));
            ui.label(format!("len: {:?}", len));
        });

        ui.separator();

        display::image_frame::show(ui, &image);

    } else {
        ui.with_layout(
            egui::Layout::centered_and_justified(egui::Direction::TopDown),
            |ui| ui.spinner(),
        );
    }
}
 */
