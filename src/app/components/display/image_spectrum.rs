use std::collections::HashMap;

use egui::plot::{Legend, Line, Plot, Value, Values};

const COLOR_MIN: u8 = 0;
const COLOR_MAX: u8 = 255;
const SCALE: f64 = 0.02;

pub fn show(ui: &mut egui::Ui, spectrum: &[HashMap<u8, usize>]) {
    Plot::new("Image spectrum")
        .legend(Legend::default())
        .data_aspect(1.0)
        .show(ui, |plot_ui| {
            for (index, c_map) in spectrum.iter().enumerate() {
                let (color, name) = match index {
                    0 => (egui::Color32::RED, "Red spectrum"),
                    1 => (egui::Color32::GREEN, "Green spectrum"),
                    _ => (egui::Color32::BLUE, "Blue spectrum"),
                };

                let line = (COLOR_MIN..COLOR_MAX)
                    .map(|intensity| {
                        if let Some(&count) = c_map.get(&intensity) {
                            (intensity, count as f64)
                        } else {
                            (intensity, 0.0_f64)
                        }
                    })
                    // The 10 factor here is purely for a nice 1:1 aspect ratio
                    .map(|(x, f)| Value::new(x as f64, f * SCALE));

                plot_ui.line(
                    Line::new(Values::from_values_iter(line))
                        .color(color)
                        .name(name),
                );
            }
        });
}
