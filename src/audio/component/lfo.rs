use crate::audio::MAX_POLY_COUNT;
use super::WaveShape;

pub const TOTAL_INPUT_COUNT: usize = 0 * MAX_POLY_COUNT;

pub const OUT_VALUE: usize = 0 * MAX_POLY_COUNT;
pub const TOTAL_OUTPUT_COUNT: usize = 1 * MAX_POLY_COUNT;

pub struct PolyLfo<const INPUT_OFFSET: usize, const OUTPUT_OFFSET: usize> {
    shape: WaveShape,
    phase: f64,
    frequency: f64,
    current_phases: [f64; MAX_POLY_COUNT],
}

impl <const INPUT_OFFSET: usize, const OUTPUT_OFFSET: usize> PolyLfo <INPUT_OFFSET, OUTPUT_OFFSET> {
    pub fn new() -> Self {
        Self {
            shape: WaveShape::default(),
            phase: 0.0,
            frequency: 1.0,
            current_phases: [0.0; MAX_POLY_COUNT],
        }
    }

    pub fn set_shape(&mut self, shape: WaveShape) {
        self.shape = shape;
    }

    pub fn set_freq_value(&mut self, freq: f32) {
        self.frequency = (freq as f64).powf(3.0) * 99.9 + 0.1;
    }

    #[allow(dead_code)]
    pub fn set_phase_value(&mut self, phase: f32) {
        self.phase = phase as f64;
    }

    pub fn render(&mut self, _inputs: &[f32], outputs: &mut [f32], sample_rate: f64) {
        for (lfo, current_phase) in self.current_phases.iter_mut().enumerate() {
            let phase_increment = self.frequency / sample_rate;
            let phase = *current_phase;

            let raw = match self.shape {
                WaveShape::Saw => 2.0 * phase - 1.0,
                WaveShape::Sine => (2.0 * std::f64::consts::PI * phase).sin(),
                WaveShape::Square => if phase < 0.5 {1.0} else {-1.0},
                WaveShape::Triangle => 1.0 - 4.0 * (phase - (phase + 0.5).floor()).abs(),
            };

            *current_phase = (*current_phase + phase_increment) % 1.0;
            outputs[OUTPUT_OFFSET + OUT_VALUE + lfo] = raw as f32 / 16.0;
        }
    }
}