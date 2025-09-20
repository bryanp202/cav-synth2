#[derive(Clone, Copy, Debug)]
pub enum WaveShape {
    Saw,
    Sine,
    Square,
    Triangle,
}

impl Default for WaveShape {
    fn default() -> Self {
        Self::Saw
    }
}

const LEVEL: usize = 0;
pub const FREQUENCY: usize = 1;
const PHASE: usize = 2;
pub const INPUT_COUNT: usize = 3;

const OUT_VALUE: usize = 0;
pub const OUTPUT_COUNT: usize = 1;

#[derive(Default)]
pub struct AnalogOscillator {
    sample_rate: f64,
    shape: WaveShape,
    level: f32,
    frequency: f32,
    phase: f32,
    current_phase: f64,
    inputs: usize,
    outputs: usize,
}

impl AnalogOscillator {
    pub fn new(inputs: &mut Vec<f32>, outputs: &mut Vec<f32>, sample_rate: f64) -> Self {
        let analog_inputs = inputs.len();
        let analog_outputs = outputs.len();
        inputs.resize(inputs.len() + INPUT_COUNT, 0.0);
        outputs.resize(outputs.len() + OUTPUT_COUNT, 0.0);
        Self {
            sample_rate,
            inputs: analog_inputs,
            outputs: analog_outputs,
            ..Default::default()
        }
    }

    pub fn get_outputs(&self) -> usize {
        self.outputs
    }

    pub fn get_inputs(&self) -> usize {
        self.inputs
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
}

fn calculate_freq(voltage: f32) -> f32 {
    2.0_f32.powf(127.0 / 12.0 * voltage) * 8.1757989156
}

pub fn analog_oscillator_system(analogs: &mut [AnalogOscillator], inputs: &[f32], outputs: &mut [f32]) {
    for analog in analogs {
        let phase_input = inputs[analog.inputs + PHASE];
        let frequency_input = inputs[analog.inputs + FREQUENCY];
        let level_input = inputs[analog.inputs + LEVEL];

        let level = analog.level + level_input;
        let voltage = analog.frequency + frequency_input;
        let frequency =  calculate_freq(voltage);// C-1 (midi note 0)
        let phase = (analog.current_phase + phase_input as f64) % 1.0;

        let phase_increment = frequency as f64 / analog.sample_rate;

        let raw = match analog.shape {
            WaveShape::Saw => 2.0 * phase - 1.0 - AnalogOscillator::poly_blep(phase, phase_increment),
            WaveShape::Sine => (2.0 * std::f64::consts::PI * phase).sin(),
            WaveShape::Square => {
                let raw = if phase < 0.5 {1.0} else {-1.0};
                raw + AnalogOscillator::poly_blep(phase, phase_increment) - AnalogOscillator::poly_blep((phase + 0.5) % 1.0, phase_increment)
            },
            WaveShape::Triangle => 1.0 - 4.0 * (phase - (phase + 0.5).floor()).abs(),
        };

        analog.current_phase = (analog.current_phase + phase_increment) % 1.0;

        let scaled_raw = raw as f32 * level;
        outputs[analog.outputs + OUT_VALUE] = scaled_raw;
    }
}

