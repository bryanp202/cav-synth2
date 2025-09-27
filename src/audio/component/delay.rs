use std::collections::VecDeque;

const VALUE_INPUT: usize = 0;
const INPUT_COUNT: usize = 1;

const VALUE_OUTPUT: usize = 0;
const OUTPUT_COUNT: usize = 1;

#[derive(Default)]
pub struct Delay {
    feedback: f32,
    delay_index: usize,
    inputs: usize,
    outputs: usize,
    // State
    buffer: VecDeque<f32>,
}

impl Delay {
    pub fn new(inputs: &mut Vec<f32>, outputs: &mut Vec<f32>) -> Self {
        let delay_inputs = inputs.len();
        let delay_outputs = outputs.len();
        inputs.resize(inputs.len() + INPUT_COUNT, 0.0);
        outputs.resize(outputs.len() + OUTPUT_COUNT, 0.0);
        Self {
            inputs: delay_inputs,
            outputs: delay_outputs,
            feedback: 0.5,
            delay_index: 24000,
            buffer: VecDeque::new(),
        }
    }

    pub fn set_delay_time(&mut self, delay_index: usize) {
        self.delay_index = delay_index;
        self.buffer.shrink_to(delay_index);
    }

    pub fn get_output(&self) -> usize {
        self.outputs + VALUE_OUTPUT
    }

    pub fn get_value_input(&self) -> usize {
        self.inputs + VALUE_INPUT
    }
}

pub fn delay_system(delays: &mut [Delay], inputs: &[f32], outputs: &mut [f32]) {
    for delay in delays {
        let value_input = inputs[delay.inputs + VALUE_INPUT];

        let delay_value = delay.buffer.remove(delay.delay_index).unwrap_or_default();
        let out_value = (delay.feedback - 1.0) * value_input + delay.feedback * delay_value;

        delay.buffer.push_front(out_value);
        outputs[delay.outputs] = out_value;
    }
}