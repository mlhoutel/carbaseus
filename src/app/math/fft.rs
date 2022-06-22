use rustfft::{num_complex::Complex, FftPlanner};

use super::image::ImageSlice;

// 2D fast fourier transform
pub fn mat_fft(i_buffer: ImageSlice) -> ImageSlice {
    let mut buffer: Vec<f32> = i_buffer
        .pixels
        .iter()
        .map(|px| (*px as f32) / 255.0)
        .collect();
    let row_size = i_buffer.size[0];
    let col_size = i_buffer.size[1];

    // 1. do 1D FFT on each row (real to complex)
    for i in 0..col_size {
        // extract row
        let mut row: Vec<f32> = vec![0.0; row_size];
        for j in 0..row_size {
            row[j] = buffer[i * col_size + j];
        }

        // compute fft
        let row_fft = row_fft(row);

        // insert row
        for j in 0..row_size {
            buffer[i * col_size + j] = row_fft[j];
        }
    }

    // 2. do 1D FFT on each column resulting from (1) (complex to complex)
    for j in 0..row_size {
        // extract col
        let mut col: Vec<f32> = vec![0.0; col_size];
        for i in 0..col_size {
            col[i] = buffer[i * col_size + j];
        }

        // compute fft
        let col_fft = row_fft(col);

        // insert row
        for i in 0..col_size {
            buffer[i * col_size + j] = col_fft[i];
        }
    }

    let image_output: ImageSlice = ImageSlice {
        color: i_buffer.color,
        size: i_buffer.size,
        pixels: buffer
            .iter()
            .map(|px| (*px * 255.0).floor() as u8)
            .collect(),
    };

    image_output
}

// 1D fast fourier transform
pub fn row_fft(buffer: Vec<f32>) -> Vec<f32> {
    let mut planner = FftPlanner::<f32>::new();
    let fft = planner.plan_fft_forward(buffer.len());

    let mut complex_buffer: Vec<Complex<f32>> = buffer
        .into_iter()
        .map(|e| Complex { re: e, im: 0.0 })
        .collect();

    fft.process(&mut complex_buffer);

    complex_buffer.into_iter().map(|e| e.re).collect()
}
