pub mod envelope;
pub mod analog;
pub mod cable;
pub mod effects;
pub mod filter;
pub mod wavetable;

pub fn calculate_freq(voltage: f32) -> f32 {
    2.0_f32.powf((128.0 * voltage - 69.0) / 12.0) * 440.0
}