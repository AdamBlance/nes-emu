pub fn get_bit<T: Into<u32>>(integer: T, bit_index: u8) -> bool {
    let mask = 1 << bit_index;
    let masked = integer.into() & mask;
    masked > 0
}

pub fn concat_u8(msb: u8, lsb: u8) -> u16 {
    ((msb as u16) << 8) + (lsb as u16)
}

pub fn to_mask(input: bool) -> u8 {
    !(input as u8).wrapping_sub(1)
}

/*

   Running the emulator only a handful of CPU cycles until we have to stop to sample the APU
   is probably not good for performance, but I'd need to benchmark to figure that out.
   It just feels like an awkward way of doing things and we have to manage a rolling average
   of the number of CPU cycles executed before polling for a sample.
   Although it's going to introduce latency (will it really?), it would be much simpler to just
   have the APU place a sample onto a buffer every X CPU cycles and then downsample that buffer
   to the sample rate of your audio device.
   Was going to use an audio processing library for this but I really don't think it'll be hard.
   Let's see.

*/

/*
   Not too bad; the ratio of one sample rate to the other tells us how much/many indices to jump
   before taking another sample, then just do lerp.
*/
pub fn resample(
    source_buffer: &[f64],
    source_sample_rate: f64,
    output_buffer: &mut Vec<f64>,
    output_sample_rate: f64,
) {
    // Estimate the sample that would follow to avoid audio clicking etc.
    let [penultimate, last] = source_buffer
        .last_chunk()
        .copied()
        .expect("You can't resample less than two values you twat");
    let future_sample_guess = last + (last - penultimate);

    let in_out_ratio = source_sample_rate / output_sample_rate;
    let num_samples = (source_buffer.len() as f64 * in_out_ratio.recip()) as usize;
    let sample_indices = (0..num_samples).map(|i| i as f64 * in_out_ratio);

    for sample_index_as_float in sample_indices {
        let sample = source_buffer[sample_index_as_float as usize];
        let next_sample = source_buffer
            .get(sample_index_as_float.ceil() as usize)
            .copied()
            .unwrap_or(future_sample_guess);

        let interpolated_sample = lerp(sample, next_sample, sample_index_as_float.fract());
        output_buffer.push(interpolated_sample);
    }
}

fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn upsample() {
        let source = vec![0.0, 250.0, 500.0, 750.0];
        let mut output = Vec::<f64>::new();

        resample(&source, 4.0, &mut output, 8.0);

        let expected_output_as_int = vec![0, 125, 250, 375, 500, 625, 750, 875];
        let output_as_int: Vec<i32> = output.iter().copied().map(|x| x as i32).collect();

        assert_eq!(expected_output_as_int, output_as_int);
    }

    #[test]
    fn downsample() {
        let source = vec![0.0, 250.0, 500.0, 250.0, 0.0, -250.0, -500.0, -250.0];
        let mut output = Vec::<f64>::new();

        resample(&source, 8.0, &mut output, 4.0);

        let expected_output_as_int = vec![0, 500, 0, -500];
        let output_as_int: Vec<i32> = output.iter().copied().map(|x| x as i32).collect();

        assert_eq!(expected_output_as_int, output_as_int);
    }

    #[test]
    #[should_panic]
    fn no_samples() {
        let source = vec![];
        let mut output = Vec::<f64>::new();

        resample(&source, 8.0, &mut output, 4.0);
    }

    #[test]
    fn sine_common_values() {
        let source: Vec<f64> = (0..120)
            .map(|x| (x as f64 * 1.5f64).to_radians().sin())
            .collect();
        let mut output = Vec::<f64>::new();

        resample(&source, 120.0, &mut output, 9.0);

        let expected: Vec<f64> = (0..9)
            .map(|x| (x as f64 * 20f64).to_radians().sin())
            .collect();

        assert_eq!(output.len(), expected.len());
        for (exp, out) in expected.iter().zip(output.iter()) {
            assert!((exp - out).abs() < 0.001)
        }
    }
}

/*

   120khz
   44khz

   44/120 = 0.3666...

   1000 * 0.3666 = 367

   1000 * 0.36666

   1000 / 367 = 2.724795 index move per sample

   Just do 120/44 and skip that much index space






*/
