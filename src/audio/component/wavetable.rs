use std::sync::Arc;

const LEVEL: usize = 0;
const FREQUENCY: usize = 1;
const PHASE: usize = 2;
const TABLE_INPUT: usize = 3;
const INPUT_COUNT: usize = 4;

const OUT_VALUE: usize = 0;
const OUTPUT_COUNT: usize = 1;

pub type Wavetable = [f32; 2048 * 1];

pub struct WavetableOscillator {
    wavetable: Arc<Wavetable>,
    level: f32,
    frequency: f32,
    phase: f32,
    table_pos: f32,
    current_phase: f32,
    inputs: usize,
    outputs: usize,
}

impl WavetableOscillator {
    pub fn new(inputs: &mut Vec<f32>, outputs: &mut Vec<f32>, wavetable: Arc<Wavetable>) -> Self {
        let wavetable_inputs = inputs.len();
        let wavetable_outputs = outputs.len();
        inputs.resize(inputs.len() + INPUT_COUNT, 0.0);
        outputs.resize(outputs.len() + OUTPUT_COUNT, 0.0);
        Self {
            inputs: wavetable_inputs,
            outputs: wavetable_outputs,
            level: 0.0,
            frequency: 0.0,
            phase: 0.0,
            table_pos: 0.0,
            current_phase: 0.0,
            wavetable: wavetable.clone(),
        }
    }

    pub fn update_wavetable(&mut self, new_wavetable: Arc<Wavetable>) {
        self.wavetable = new_wavetable;
    }

    pub fn set_freq_value(&mut self, freq: f32) {
        self.frequency = freq;
    }

    pub fn get_output(&self) -> usize {
        self.outputs
    }

    pub fn get_freq_input(&self) -> usize {
        self.inputs + FREQUENCY
    }

    // pub fn get_phase_input(&self) -> usize {
    //     self.inputs + PHASE
    // }

    pub fn get_level_input(&self) -> usize {
        self.inputs + LEVEL
    }

    fn linear_interp(wavetable: &Wavetable, current_phase: f32, table_pos: f32) -> f32 {
        let frame = table_pos as usize;
        let index1 = current_phase as usize;
        let index2 = (index1 + 1) % 2048;
        let index_ratio = current_phase.fract();

        wavetable[index1] + (wavetable[index2] - wavetable[index1]) * index_ratio
    }
}

pub fn wavetable_oscillator_system(wavetables: &mut [WavetableOscillator], inputs: &[f32], outputs: &mut [f32], sample_rate: f32) {
    for wavetable in wavetables {
        let phase_input = inputs[wavetable.inputs + PHASE];
        let frequency_input = inputs[wavetable.inputs + FREQUENCY];
        let level_input = inputs[wavetable.inputs + LEVEL];
        let table_input = inputs[wavetable.inputs + TABLE_INPUT];

        let level = wavetable.level + level_input;
        let voltage = wavetable.frequency + frequency_input;
        let frequency =  super::calculate_freq(voltage);// C-1 (midi note 0)
        let phase = (wavetable.current_phase + phase_input) % 2048.0;
        let table_pos = wavetable.table_pos + table_input;

        let phase_increment = frequency / sample_rate * 2048.0;

        let raw = WavetableOscillator::linear_interp(&wavetable.wavetable, phase, table_pos);

        wavetable.current_phase = (wavetable.current_phase + phase_increment) % 2048.0;

        let scaled_raw = raw as f32 * level;
        outputs[wavetable.outputs + OUT_VALUE] = scaled_raw;
    }
}