const VALUE_INPUT: usize = 0;
const FREQUENCY_INPUT: usize = 1;
const INPUT_COUNT: usize = 2;

const VALUE_OUTPUT: usize = 0;
const OUTPUT_COUNT: usize = 1;

#[derive(Default)]
pub struct Filter {
    frequency: f32,
    inputs: usize,
    outputs: usize,
    // State
    x_minus: f32,
    x_minus2: f32,
    y_minus: f32,
    y_minus2: f32,
}

impl Filter {
    pub fn new(inputs: &mut Vec<f32>, outputs: &mut Vec<f32>) -> Self {
        let filter_inputs = inputs.len();
        let filter_outputs = outputs.len();
        inputs.resize(inputs.len() + INPUT_COUNT, 0.0);
        outputs.resize(outputs.len() + OUTPUT_COUNT, 0.0);
        Self {
            frequency: 0.0,
            inputs: filter_inputs,
            outputs: filter_outputs,
            ..Default::default()
        }
    }

    pub fn get_output(&self) -> usize {
        self.outputs
    }

    pub fn get_freq_input(&self) -> usize {
        self.inputs + FREQUENCY_INPUT
    }

    pub fn get_value_input(&self) -> usize {
        self.inputs + VALUE_INPUT
    }
}

pub fn butterworth_system(filters: &mut [Filter], inputs: &[f32], outputs: &mut [f32], sample_rate: f64) {
    let sample_rate = sample_rate as f32;
    for filter in filters {
        let value_input = inputs[filter.inputs + VALUE_INPUT];

        if value_input != 0.0 {
            let frequency_input = inputs[filter.inputs + FREQUENCY_INPUT];

            let voltage = (filter.frequency + frequency_input).min(1.0).max(0.0);
            let frequency = super::calculate_freq(voltage);

            let c = 1.0 / (std::f32::consts::PI * frequency / sample_rate).tan();
            let a0 = 1.0 / (1.0 + 2.0_f32.sqrt() * c + c * c);
            let a1 = 2.0 * a0;
            let a2 = a0;
            let b1 = 2.0 * a0 * (1.0 - c * c);
            let b2 = a0 * (1.0 - 2.0_f32.sqrt() * c + c * c);

            let output_value = value_input * a0 + filter.x_minus * a1 + filter.x_minus2 * a2 - filter.y_minus * b1 - filter.y_minus2 * b2;

            filter.x_minus2 = filter.x_minus;
            filter.x_minus = value_input;
            filter.y_minus2 = filter.y_minus;
            filter.y_minus = output_value;

            outputs[filter.outputs + VALUE_OUTPUT] = output_value;
        } else {
            outputs[filter.outputs + VALUE_OUTPUT] = 0.0;
        }
    }
}