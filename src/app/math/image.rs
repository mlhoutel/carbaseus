use egui::epaint::{Color32, ColorImage};
use image::imageops;

#[derive(Clone, PartialEq)]
pub enum SliceColor {
    Red,
    Green,
    Blue,
    Gray,
}

impl Default for SliceColor {
    fn default() -> Self {
        SliceColor::Blue
    }
}

#[derive(Clone, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct ImageSlice {
    pub color: SliceColor,
    /// width, height.
    pub size: [usize; 2],
    /// The pixels, row by row, from top to bottom.
    pub pixels: Vec<u8>,
}

impl ImageSlice {
    /// Create an image filled with the given color.
    pub fn new(color: SliceColor, size: [usize; 2]) -> Self {
        Self {
            color,
            size,
            pixels: vec![0; size[0] * size[1]],
        }
    }

    pub fn from_image(image: ColorImage, color: SliceColor) -> Self {
        let index = match color {
            SliceColor::Red => 0,
            SliceColor::Green => 1,
            SliceColor::Blue => 2,
            SliceColor::Gray => 0,
        };

        let pixels: Vec<u8> = image.pixels.iter().map(|p| p[index]).collect();

        Self {
            size: image.size,
            color,
            pixels,
        }
    }

    pub fn to_image(&self) -> ColorImage {
        let mut image = ColorImage::new(self.size, Color32::BLACK);

        match self.color {
            SliceColor::Red => {
                for (i, pixel) in image.pixels.iter_mut().enumerate() {
                    *pixel = Color32::from_rgb(self.pixels[i], 0, 0)
                }
            }
            SliceColor::Green => {
                for (i, pixel) in image.pixels.iter_mut().enumerate() {
                    *pixel = Color32::from_rgb(0, self.pixels[i], 0)
                }
            }
            SliceColor::Blue => {
                for (i, pixel) in image.pixels.iter_mut().enumerate() {
                    *pixel = Color32::from_rgb(0, 0, self.pixels[i])
                }
            }
            SliceColor::Gray => {
                for (i, pixel) in image.pixels.iter_mut().enumerate() {
                    *pixel = Color32::from_rgb(self.pixels[i], self.pixels[i], self.pixels[i])
                }
            }
        };

        image
    }
}

pub fn egui_to_image(image: ColorImage) -> image::RgbImage {
    let mut image_buffer = image::RgbImage::new(image.size[0] as u32, image.size[1] as u32);

    for (x, y, pixel) in image_buffer.enumerate_pixels_mut() {
        let index = (x as usize) * image.size[0] + (y as usize);
        let tmp_pixels = image.pixels[index];
        *pixel = image::Rgb([tmp_pixels.r(), tmp_pixels.g(), tmp_pixels.b()]);
    }

    image_buffer
}

pub fn image_to_egui(image: image::RgbImage) -> ColorImage {
    let mut image_buffer = ColorImage::new(
        [image.width() as usize, image.height() as usize],
        Color32::BLACK,
    );
    for (x, y, &pixel) in image.enumerate_pixels() {
        let index = (x as usize) * image_buffer.size[0] + (y as usize);
        image_buffer.pixels[index] = Color32::from_rgb(pixel[0], pixel[1], pixel[2]);
    }

    image_buffer
}

pub fn _slice_to_luma(image: ImageSlice) -> image::GrayImage {
    let mut image_buffer = image::GrayImage::new(image.size[0] as u32, image.size[1] as u32);

    for (x, y, pixel) in image_buffer.enumerate_pixels_mut() {
        let index = (x as usize) * image.size[0] + (y as usize);
        let tmp_pixels = image.pixels[index];
        *pixel = image::Luma([tmp_pixels]);
    }

    image_buffer
}

pub fn luma_to_slice(image: image::GrayImage) -> ImageSlice {
    let mut image_buffer = ImageSlice::new(
        SliceColor::Gray,
        [image.width() as usize, image.height() as usize],
    );
    for (x, y, &pixel) in image.enumerate_pixels() {
        let index = (x as usize) * image_buffer.size[0] + (y as usize);
        image_buffer.pixels[index] = pixel[0];
    }

    image_buffer
}

// Convert rgb pixel to normilized gray value
pub fn _rgb_to_normal(px: Color32) -> f32 {
    (px[0] as f32) / 255.0
}

// Convert rgb pixel to normilized gray value
pub fn _normal_to_rgb(nm: f32) -> Color32 {
    let c = (nm * 255.0).floor() as u8;
    Color32::from_rgb(c, c, c)
}

// Image to gray scale
pub fn image_to_gray(image: &ColorImage) -> ImageSlice {
    let temp_image = egui_to_image(image.clone());

    let output_image = imageops::grayscale(&temp_image);

    luma_to_slice(output_image)
}

// Image blur
pub fn image_blur(image: &ColorImage, sigma: f32) -> ColorImage {
    let temp_image = egui_to_image(image.clone());

    let output_image = imageops::blur(&temp_image, sigma);

    image_to_egui(output_image)
}

pub fn _gray_to_image(buffer: Vec<f32>, row_size: usize, col_size: usize) -> ColorImage {
    let mut image = ColorImage::new([1, 1], Color32::from_rgb(0, 0, 0));
    let cols: Vec<Color32> = buffer.into_iter().map(_normal_to_rgb).collect();
    image.size = [row_size, col_size];
    image.pixels = cols;

    image
}
// from emgui_extras 1.18.0 / images.rs
pub fn load_image_bytes(image_bytes: &[u8]) -> Option<ColorImage> {
    let result = image::load_from_memory(image_bytes).map_err(|err| err.to_string());

    match result {
        Ok(image) => {
            let size = [image.width() as _, image.height() as _];
            let image_buffer = image.to_rgba8();
            let pixels = image_buffer.as_flat_samples();

            Some(ColorImage::from_rgba_unmultiplied(size, pixels.as_slice()))
        }
        Err(_) => None,
    }
}
