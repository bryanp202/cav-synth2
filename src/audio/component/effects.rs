use std::collections::VecDeque;

use crate::audio::MAX_POLY_COUNT;

pub struct EffectsChain {
    distortion: Distortion,
    delay: Delay,
    reverb: Reverb,
}

impl EffectsChain {
    pub fn new() -> Self {
        Self {
            distortion: Distortion::new(),
            delay: Delay::new(),
            reverb: Reverb::new(),
        }
    }

    pub fn set_delay_time(&mut self, delay_index: usize) {
        self.delay.delay_index = delay_index;
        self.delay.buffer.shrink_to(delay_index);
    }

    pub fn set_delay_wet_ratio(&mut self, wet: f32) {
        self.delay.wet = wet;
    }

    pub fn render(&mut self, inputs: &[f32; MAX_POLY_COUNT]) -> (f32, f32) {
        let input = inputs.iter().sum();
        let distorted = self.distortion.render(input);
        let delayed = self.delay.render(distorted);
        self.reverb.render(delayed)
    }
}

struct Distortion {
    wet: f32,
    drive: f32,
}

impl Distortion {
    fn new() -> Self {
        Self {
            wet: 0.0,
            drive: 1.0,
        }
    }

    fn render(&self, input: f32) -> f32 {
        let drive_value = self.drive * input;
        let wet_value = drive_value.powf(3.0);
        let out_value = input + (wet_value - input) * self.wet;
        out_value
    }
}

struct Delay {
    wet: f32,
    delay_index: usize,
    // State
    buffer: VecDeque<f32>,
}

impl Delay {
    fn new() -> Self {
        Self {
            wet: 0.0,
            delay_index: 24000,
            buffer: VecDeque::new(),
        }
    }

    fn render(&mut self, input: f32) -> f32 {
        let wet_value = self.buffer.remove(self.delay_index).unwrap_or_default();
        let out_value = input + (wet_value - input) * self.wet;

        self.buffer.push_front(out_value);
        out_value
    }
}

struct Reverb {
    wet: f32,
}

impl Reverb {
    fn new() -> Self {
        Self {
            wet: 0.0
        }
    }

    fn render(&self, input: f32) -> (f32, f32) {
        (input, input)
    }
}