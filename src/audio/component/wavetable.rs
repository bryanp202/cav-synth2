use std::sync::Arc;

use crate::audio::MAX_POLY_COUNT;

pub const LEVEL_INPUT: usize = 0 * MAX_POLY_COUNT;
pub const FREQUENCY_INPUT: usize = 1 * MAX_POLY_COUNT;
pub const PHASE_INPUT: usize = 2 * MAX_POLY_COUNT;
pub const AMP_INPUT: usize = 3 * MAX_POLY_COUNT;
pub const TOTAL_INPUT_COUNT: usize = 4 * MAX_POLY_COUNT;

pub const OUT_VALUE: usize = 0 * MAX_POLY_COUNT;
pub const TOTAL_OUTPUT_COUNT: usize = 1 * MAX_POLY_COUNT;

pub const WAVETABLE_FRAME_LENGTH: usize = 2048;
pub const WAVETABLE_VARIATION_COUNT: usize = 8;
pub type Wavetable = [f32; WAVETABLE_FRAME_LENGTH * WAVETABLE_VARIATION_COUNT];

pub struct PolyWavetable<const INPUT_OFFSET: usize, const OUTPUT_OFFSET: usize> {
    wavetable: Arc<Wavetable>,
    level: f32,
    frequency: f32,
    phase: f32,
    current_phases: [f32; MAX_POLY_COUNT],
}

impl <const INPUT_OFFSET: usize, const OUTPUT_OFFSET: usize> PolyWavetable <INPUT_OFFSET, OUTPUT_OFFSET> {
    pub fn new() -> Self {
        Self {
            wavetable: Arc::new([0.0; WAVETABLE_FRAME_LENGTH * WAVETABLE_VARIATION_COUNT]),
            level: 0.5,
            frequency: 0.0,
            phase: 0.0,
            current_phases: [0.0; MAX_POLY_COUNT],
        }
    }

    pub fn update_wavetable(&mut self, new_wavetable: Arc<Wavetable>) {
        self.wavetable = new_wavetable;
    }

    pub fn set_freq_value(&mut self, freq: f32) {
        self.frequency = (freq - 0.5) * 24.0 / 128.0;
    }

    pub fn set_phase_value(&mut self, phase: f32) {
        self.phase = phase;
    }

    pub fn set_level_value(&mut self, level: f32) {
        self.level = level;
    }

    #[inline(always)]
    pub fn render(&mut self, inputs: &[f32], outputs: &mut [f32], sample_rate: f32) {
        for (wavetable, current_phase) in self.current_phases.iter_mut().enumerate() {
            let phase_input = inputs[INPUT_OFFSET + PHASE_INPUT + wavetable] * WAVETABLE_FRAME_LENGTH as f32;
            let frequency_input = inputs[INPUT_OFFSET + FREQUENCY_INPUT + wavetable];
            let level_input = inputs[INPUT_OFFSET + LEVEL_INPUT + wavetable];
            let amp_input = inputs[INPUT_OFFSET + AMP_INPUT + wavetable];

            let level = self.level + level_input;
            let voltage = self.frequency + frequency_input;
            let frequency =  super::calculate_freq(voltage);// C-1 (midi note 0)
            let phase = (*current_phase + phase_input) % WAVETABLE_FRAME_LENGTH as f32;

            let phase_increment = frequency / sample_rate * WAVETABLE_FRAME_LENGTH as f32;
            let raw = linear_interp(&self.wavetable, phase, voltage);

            *current_phase = (*current_phase + phase_increment) % WAVETABLE_FRAME_LENGTH as f32;

            let scaled_raw = raw as f32 * level * amp_input;
            outputs[OUTPUT_OFFSET + OUT_VALUE + wavetable] = scaled_raw;
        }
    }
}

fn linear_interp(wavetable: &Wavetable, current_phase: f32, frequency_voltage: f32) -> f32 {
    let variation = ((frequency_voltage * 128.0 - 30.0) / 10.0).clamp(0.0, WAVETABLE_VARIATION_COUNT as f32 - 0.1) as usize;
    // FIX THIS TO BE BETTER ////// MAYBE IT NEEDS TO FADE?
    let variation_offset = variation * WAVETABLE_FRAME_LENGTH;
    let index1 = variation_offset + current_phase as usize;
    let index2 = variation_offset + (index1 + 1) % WAVETABLE_FRAME_LENGTH;
    let index_ratio = current_phase.fract();

    wavetable[index1] + (wavetable[index2] - wavetable[index1]) * index_ratio
}