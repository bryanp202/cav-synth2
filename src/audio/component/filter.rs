use crate::audio::MAX_POLY_COUNT;

pub const VALUE_INPUT: usize = 0 * MAX_POLY_COUNT;
pub const FREQUENCY_INPUT: usize = 1 * MAX_POLY_COUNT;
pub const TOTAL_INPUT_COUNT: usize = 2 * MAX_POLY_COUNT;

pub const VALUE_OUTPUT: usize = 0 * MAX_POLY_COUNT;
pub const TOTAL_OUTPUT_COUNT: usize = 1 * MAX_POLY_COUNT;

#[derive(Clone, Copy, Default)]
struct BufferData {
    x_minus: f32,
    x_minus2: f32,
    y_minus: f32,
    y_minus2: f32,
}

pub struct PolyFilter<const INPUT_OFFSET: usize, const OUTPUT_OFFSET: usize> {
    frequency: f32,
    buffers: [BufferData; MAX_POLY_COUNT],
}

impl <const INPUT_OFFSET: usize, const OUTPUT_OFFSET: usize> PolyFilter <INPUT_OFFSET, OUTPUT_OFFSET> {
    pub fn new() -> Self {
        Self {
            frequency: 0.4,
            buffers: [BufferData::default(); MAX_POLY_COUNT],
        }
    }

    pub fn set_freq_value(&mut self, freq: f32) {
        self.frequency = freq;
    }

    pub fn render(&mut self, inputs: &[f32], outputs: &mut [f32], sample_rate: f32) {
        for (filter, buffer) in self.buffers.iter_mut().enumerate() {
            let value_input = inputs[INPUT_OFFSET + VALUE_INPUT + filter];

            if value_input != 0.0 {
                let frequency_input = inputs[INPUT_OFFSET + FREQUENCY_INPUT + filter];

                let voltage = (self.frequency + frequency_input).min(1.0).max(0.0);
                let frequency = super::calculate_freq(voltage);

                let c = 1.0 / (std::f32::consts::PI * frequency / sample_rate).tan();
                let a0 = 1.0 / (1.0 + 2.0_f32.sqrt() * c + c * c);
                let a1 = 2.0 * a0;
                let a2 = a0;
                let b1 = 2.0 * a0 * (1.0 - c * c);
                let b2 = a0 * (1.0 - 2.0_f32.sqrt() * c + c * c);

                let output_value = value_input * a0 + buffer.x_minus * a1 + buffer.x_minus2 * a2 - buffer.y_minus * b1 - buffer.y_minus2 * b2;

                buffer.x_minus2 = buffer.x_minus;
                buffer.x_minus = value_input;
                buffer.y_minus2 = buffer.y_minus;
                buffer.y_minus = output_value;

                outputs[OUTPUT_OFFSET + VALUE_OUTPUT + filter] = output_value;
            } else {
                outputs[OUTPUT_OFFSET + VALUE_OUTPUT + filter] = 0.0;
            }
        }
    }
}