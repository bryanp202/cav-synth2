use std::collections::VecDeque;

use crate::audio::MAX_POLY_COUNT;

pub struct EffectsChain {
    distortion: Distortion,
    delay: Delay,
    reverb: Reverb,
    master_gain: f32,
}

impl EffectsChain {
    pub fn new() -> Self {
        Self {
            distortion: Distortion::new(),
            delay: Delay::new(),
            reverb: Reverb::new(),
            master_gain: 1.0,
        }
    }

    pub fn set_dist_drive(&mut self, drive: f32) {
        self.distortion.drive = drive;
    }

    pub fn set_dist_wet(&mut self, wet: f32) {
        self.distortion.wet = wet;
    }

    pub fn set_delay_time(&mut self, value: f32, sample_rate: f32) {
        let delay_index = (value * sample_rate) as usize;
        self.delay.delay_index = delay_index;
        self.delay.buffer.shrink_to(delay_index);
    }

    pub fn set_delay_feedback(&mut self, feedback: f32) {
        self.delay.feedback = feedback;
    }

    pub fn set_delay_wet(&mut self, wet: f32) {
        self.delay.wet = wet;
    }

    pub fn set_reverb_time(&mut self, time: f32) {
        self.reverb.time = time;
    }

    pub fn set_reverb_wet(&mut self, wet: f32) {
        self.reverb.wet = wet;
    }

    pub fn set_master_gain(&mut self, gain: f32) {
        self.master_gain = gain;
    }

    pub fn render(&mut self, inputs: &[f32; MAX_POLY_COUNT]) -> (f32, f32) {
        let input = inputs.iter().sum();
        let distorted = self.distortion.render(input);
        let delayed = self.delay.render(distorted);
        self.reverb.render(delayed * self.master_gain)
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
    feedback: f32,
    delay_index: usize,
    // State
    buffer: VecDeque<f32>,
}

impl Delay {
    fn new() -> Self {
        Self {
            wet: 0.0,
            feedback: 0.7,
            delay_index: 24000,
            buffer: VecDeque::new(),
        }
    }

    fn render(&mut self, input: f32) -> f32 {
        let wet_value = self.buffer.remove(self.delay_index).unwrap_or_default();
        let out_value = input + (wet_value - input) * self.wet;

        self.buffer.push_front((input + wet_value) * self.feedback);
        out_value
    }
}

struct Reverb {
    wet: f32,
    time: f32,
}

impl Reverb {
    fn new() -> Self {
        Self {
            wet: 0.0,
            time: 0.0,
        }
    }

    fn render(&self, input: f32) -> (f32, f32) {
        (input, input)
    }
}