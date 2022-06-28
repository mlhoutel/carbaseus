use rustfft::{num_complex::Complex, FftPlanner};

use super::image::ImageSlice;

// 2D fast fourier transform
pub fn mat_fft(i_buffer: ImageSlice) -> ImageSlice {
    // 1. Convert real to complex.
    let mut buffer: Vec<Complex<f32>> = i_buffer
        .pixels
        .iter()
        .map(|px| Complex::from((*px as f32) / 255.0))
        .collect();

    let row_size = i_buffer.size[0];
    let col_size = i_buffer.size[1];

    // 2. do 1D FFT on each row
    for i in 0..col_size {
        let mut row: Vec<Complex<f32>> = vec![Complex::default(); row_size];

        // extract row
        for j in 0..row_size {
            row[j] = buffer[i * row_size + j];
        }

        // compute fft
        row_fft(&mut row);

        // insert row
        for j in 0..row_size {
            buffer[i * row_size + j] = row[j];
        }
    }

    // 3. do 1D FFT on each column
    for j in 0..row_size {
        // extract col
        let mut col: Vec<Complex<f32>> = vec![Complex::default(); col_size];
        for i in 0..col_size {
            col[i] = buffer[i * row_size + j];
        }

        // compute fft
        row_fft(&mut col);

        // insert row
        for i in 0..col_size {
            buffer[i * row_size + j] = col[i];
        }
    }

    // 4. Normalize result
    let factor = (buffer.len() as f32).sqrt();
    for i in 0..col_size {
        for j in 0..row_size {
            buffer[i * row_size + j] = Complex {
                re: buffer[i * row_size + j].re / factor,
                im: buffer[i * row_size + j].im / factor,
            };
        }
    }

    // 5. Shift the zero-frequency component to the center
    shift_fft(&mut buffer, col_size, row_size);

    // 6. Convert complex to real
    let image_output: ImageSlice = ImageSlice {
        color: i_buffer.color,
        size: i_buffer.size,
        pixels: buffer
            .iter()
            .map(|px| (px.re * 255.0).floor() as u8)
            .collect(),
    };

    image_output
}

// 1D fast fourier transform
pub fn row_fft(buffer: &mut Vec<Complex<f32>>) {
    let mut planner = FftPlanner::<f32>::new();

    let fft = planner.plan_fft_forward(buffer.len());

    fft.process(buffer);
}
// Swap opposite diagonal quadrants
pub fn shift_fft(buffer: &mut [Complex<f32>], row_size: usize, col_size: usize) {
    let h_half = (row_size as f32) / 2.0; // horizontal half
    let v_half = (col_size as f32) / 2.0; // vertical half

    let row_msize = h_half.floor() as usize; // center i - half
    let col_msize = v_half.floor() as usize; // center j - half

    let ii = h_half.ceil() as usize; // center i + half
    let jj = v_half.ceil() as usize; // center j + half

    let center = ii * row_size + jj;

    // Swap top left with bottom right
    for i in 0..row_msize {
        for j in 0..col_msize {
            let tl_idx = i * row_size + j;
            let br_idx = center + tl_idx;

            buffer.swap(tl_idx, br_idx);
        }
    }

    // Swap top right with bottom left
    for i in 0..row_msize {
        for j in 0..col_msize {
            let tl_idx = i * row_size + j + jj;
            let br_idx = (i + ii) * row_size + j;

            buffer.swap(tl_idx, br_idx);
        }
    }
}
