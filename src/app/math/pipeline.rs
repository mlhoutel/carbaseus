use math::*;

pub fn process_image(state: &mut state::AppState) {
    // Request received but output image not processed yet
    if state.output_image.is_none() {
        if let Some(image) = &state.input_image {
            let copy_image = (*image).clone();

            let size = copy_image.size;

            let gray_image = image::image_to_gray(&copy_image);

            state.output_image = Some(gray_image.clone());

            let gray_buffer = gray_image
                .pixels
                .into_iter()
                .map(image::rgb_to_normal)
                .collect();

            let computed = fft::mat_fft(gray_buffer, size[0], size[1]);

            state.output_image = Some(image::gray_to_image(computed, size[0], size[1]));
        }
    }
}

pub fn compute_gray(state: &mut state::AppState) {
    if let Some(image) = &state.input_image {
        let copy_image = (*image).clone();

        let size = copy_image.size;

        let gray_image = image::image_to_gray(&copy_image);
    }
}
