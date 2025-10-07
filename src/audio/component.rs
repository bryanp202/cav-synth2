pub mod envelope;
pub mod analog;
pub mod cable;
pub mod effects;
pub mod filter;
pub mod lfo;
pub mod wavetable;

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

#[inline(always)]
pub fn calculate_freq(voltage: f32) -> f32 {
    2.0_f32.powf((128.0 * voltage - 69.0) / 12.0) * 440.0
}