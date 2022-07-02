use egui::epaint::ColorImage;
use egui_extras::{image::RetainedImage, Size, TableBuilder};
use std::mem::size_of;

pub fn show(ui: &mut egui::Ui, image: &ColorImage, retained: &RetainedImage) {
    let full_size = image.size[0] * image.size[1];
    let pixel_weight = 3 * size_of::<usize>();
    let full_weight = full_size * pixel_weight;
    let row_height = 14.0;

    TableBuilder::new(ui)
        .column(Size::initial(60.0).at_least(40.0))
        .column(Size::remainder().at_least(60.0))
        .body(|mut body| {
            body.row(row_height, |mut row| {
                row.col(|ui| {
                    ui.label("Name".to_string());
                });
                row.col(|ui| {
                    ui.label(retained.debug_name().to_string());
                });
            });

            body.row(row_height, |mut row| {
                row.col(|ui| {
                    ui.label("Size".to_string());
                });
                row.col(|ui| {
                    ui.label(format!(
                        "{} x {} ({} px)",
                        image.width(),
                        image.height(),
                        full_size
                    ));
                });
            });

            body.row(row_height, |mut row| {
                row.col(|ui| {
                    ui.label("Weight".to_string());
                });
                row.col(|ui| {
                    ui.label(format!(
                        "{} Ko ({} bits / px)",
                        full_weight as f32 * (1.0 / 8000.0),
                        pixel_weight,
                    ));
                });
            });
        });
}
