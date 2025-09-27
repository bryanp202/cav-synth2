pub mod envelope;
pub mod analog;
pub mod cable;
pub mod delay;
pub mod filter;
pub mod wavetable;

pub fn calculate_freq(voltage: f32) -> f32 {
    2.0_f32.powf(127.0 / 12.0 * voltage) * 8.1757989156
}