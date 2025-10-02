use crate::audio::MAX_POLY_COUNT;

pub const LEVEL_INPUT: usize = 0 * MAX_POLY_COUNT;
pub const FREQUENCY_INPUT: usize = 1 * MAX_POLY_COUNT;
pub const PHASE_INPUT: usize = 2 * MAX_POLY_COUNT;
pub const AMP_INPUT: usize = 3 * MAX_POLY_COUNT;
pub const TOTAL_INPUT_COUNT: usize = 4 * MAX_POLY_COUNT;

pub const OUT_VALUE: usize = 0 * MAX_POLY_COUNT;
pub const TOTAL_OUTPUT_COUNT: usize = 1 * MAX_POLY_COUNT;

#[derive(Clone, Copy, Debug)]
pub enum WaveShape {
    Saw,
    Sine,
    Square,
    Triangle,
}

impl Default for WaveShape {
    fn default() -> Self {
        Self::Sine
    }
}

pub struct PolyAnalog<const INPUT_OFFSET: usize, const OUTPUT_OFFSET: usize> {
    shape: WaveShape,
    level: f32,
    phase: f32,
    frequency: f32,
    current_phases: [f64; MAX_POLY_COUNT],
}

impl <const INPUT_OFFSET: usize, const OUTPUT_OFFSET: usize> PolyAnalog <INPUT_OFFSET, OUTPUT_OFFSET> {
    pub fn new() -> Self {
        Self {
            shape: WaveShape::default(),
            level: 0.5,
            phase: 0.0,
            frequency: 0.0,
            current_phases: [0.0; MAX_POLY_COUNT],
        }
    }

    pub fn set_shape(&mut self, shape: WaveShape) {
        self.shape = shape;
    }

    pub fn set_freq_value(&mut self, freq: f32) {
        self.frequency = freq;
    }

    pub fn render(&mut self, inputs: &[f32], outputs: &mut [f32], sample_rate: f64) {
        for (analog, current_phase) in self.current_phases.iter_mut().enumerate() {
            let phase_input = inputs[INPUT_OFFSET + PHASE_INPUT + analog];
            let frequency_input = inputs[INPUT_OFFSET + FREQUENCY_INPUT + analog];
            let level_input = inputs[INPUT_OFFSET + LEVEL_INPUT + analog];
            let amp_input = inputs[INPUT_OFFSET + AMP_INPUT + analog];

            let level = self.level + level_input;
            let voltage = self.frequency + frequency_input;
            let frequency =  super::calculate_freq(voltage);// C-1 (midi note 0)
            let phase = (*current_phase + phase_input as f64) % 1.0;

            let phase_increment = frequency as f64 / sample_rate;

            let raw = match self.shape {
                WaveShape::Saw => 2.0 * phase - 1.0 - poly_blep(phase, phase_increment),
                WaveShape::Sine => (2.0 * std::f64::consts::PI * phase).sin(),
                WaveShape::Square => {
                    let raw = if phase < 0.5 {1.0} else {-1.0};
                    raw + poly_blep(phase, phase_increment) - poly_blep((phase + 0.5) % 1.0, phase_increment)
                },
                WaveShape::Triangle => 1.0 - 4.0 * (phase - (phase + 0.5).floor()).abs(),
            };

            *current_phase = (*current_phase + phase_increment) % 1.0;

            let scaled_raw = raw as f32 * level * amp_input;
            outputs[OUTPUT_OFFSET + OUT_VALUE + analog] = scaled_raw;
        }
    }
}

fn poly_blep(phase: f64, phase_increment: f64) -> f64 {
    if phase < phase_increment {
        let t = phase / phase_increment;
        t+t - t*t - 1.0
    } else if phase > 1.0 - phase_increment {
        let t = (phase - 1.0) / phase_increment;
        t*t + t+t + 1.0
    } else {
        0.0
    }
}