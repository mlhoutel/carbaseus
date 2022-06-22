use egui::epaint::ColorImage;
use egui_extras::image::RetainedImage;

pub fn _from_image(ui: &mut egui::Ui, image: &ColorImage) {
    let ret_image = RetainedImage::from_color_image("fetched image", (*image).clone());
    show(ui, &ret_image);
}

pub fn show(ui: &mut egui::Ui, image: &RetainedImage) {
    let mut size = image.size_vec2();
    size *= (ui.available_width() / size.x).min(1.0);
    image.show_size(ui, size);
}
