use core::f64;
use std::collections::VecDeque;

use crate::audio::MAX_POLY_COUNT;

pub struct EffectsChain {
    distortion: Distortion,
    delay: Delay,
    reverb: Reverb,
    master_gain: f32,
}

impl EffectsChain {
    pub fn new(sample_rate: f64) -> Self {
        Self {
            distortion: Distortion::new(),
            delay: Delay::new(),
            reverb: Reverb::new(sample_rate),
            master_gain: 0.7,
        }
    }

    pub fn set_dist_drive(&mut self, drive: f32) {
        self.distortion.drive = 1.0 + drive * 5.0;
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

    pub fn set_reverb_damp(&mut self, damp: f32) {
        self.reverb.damp = 0.1 + 0.5 * damp;
    }

    pub fn set_reverb_spread(&mut self, spread: f32) {
        self.reverb.stereo_spread = (100 as f32 * spread) as usize;
        self.set_reverb_wet(1.0 - self.reverb.dry);
    }

    pub fn set_reverb_space(&mut self, space: f32) {
        self.reverb.space = 0.74 + 0.24 * space;
    }

    pub fn set_reverb_wet(&mut self, wet: f32) {
        self.reverb.wet1 = wet * (self.reverb.width / 2.0 + 0.5) / 2.0;
        self.reverb.wet2 = wet * (1.0 - self.reverb.width) / 2.0;
        self.reverb.dry = 1.0 - wet;
    }

    pub fn set_master_gain(&mut self, gain: f32) {
        self.master_gain = gain;
    }

    #[inline(always)]
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

    #[inline(always)]
    fn render(&self, input: f32) -> f32 {
        let drive_value = self.drive * input;
        let wet_value = drive_value.signum() * (1.0 - (-(drive_value.abs())).exp());
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

    #[inline(always)]
    fn render(&mut self, input: f32) -> f32 {
        let wet_value = self.buffer.remove(self.delay_index).unwrap_or_default();
        let out_value = input + (wet_value - input) * self.wet;

        self.buffer.push_front((input + wet_value) * self.feedback);
        out_value
    }
}

const COMB_FB: f32 = 0.98;
const COMB_DAMP: f32 = 0.10;
const COMB_N: [usize; 8] = [1557, 1617, 1491, 1422, 1277, 1356, 1188, 1116];

const ALLPASS_FB: f32 = 0.5;
const ALLPASS_N: [usize; 4] = [225, 556, 441, 341];

const INIT_SPREAD: usize = 23;
const MOD_RANGE: usize = 20;
const MOD_FREQ: f64 = 3.0;

struct Reverb {
    wet1: f32,
    wet2: f32,
    dry: f32,
    damp: f32,
    width: f32,
    stereo_spread: usize,
    space: f32,

    lfo_current_phase: f64,
    lfo_phase_step: f64,

    combs: [(VecDeque<f32>, VecDeque<f32>); 8],
    comb_state: [(f32, f32); 8],
    allpass: [(VecDeque<f32>, VecDeque<f32>); 4],
}

impl Reverb {
    fn new(sample_rate: f64) -> Self {
        Self {
            wet1: 0.0,
            wet2: 0.0,
            dry: 1.0,
            width: 1.0,
            space: COMB_FB,
            damp: COMB_DAMP,
            stereo_spread: INIT_SPREAD,
            combs: std::array::from_fn(|i| (VecDeque::with_capacity(COMB_N[i] + MOD_RANGE), VecDeque::with_capacity(COMB_N[i] + MOD_RANGE))),
            comb_state: [(0.0, 0.0); 8],
            allpass: std::array::from_fn(|i| (VecDeque::with_capacity(ALLPASS_N[i]), VecDeque::with_capacity(ALLPASS_N[i]))),

            lfo_current_phase: 0.0,
            lfo_phase_step: MOD_FREQ / sample_rate,
        }
    }

    #[inline(always)]
    fn render(&mut self, input: f32) -> (f32, f32) {
        let input_scaled = input / 16.0;
        let mut out_l = 0.0;
        let mut out_r = 0.0;

        let lfo_value = 1.0 - 4.0 * (self.lfo_current_phase - (self.lfo_current_phase + 0.5).floor()).abs();
        let mod_offset = (lfo_value * MOD_RANGE as f64 / 2.0) as isize;
        self.lfo_current_phase = (self.lfo_current_phase + self.lfo_phase_step) % 1.0;

        for (i, (comb_l, comb_r)) in self.combs.iter_mut().enumerate() {
            out_l += comb_process(input_scaled, comb_l, &mut self.comb_state[i].0, self.space, COMB_N[i], mod_offset, self.damp);
            out_r += comb_process(input_scaled, comb_r, &mut self.comb_state[i].1, self.space, COMB_N[i] + self.stereo_spread, mod_offset, self.damp);
        }

        for (i, (allpass_l, allpass_r)) in self.allpass.iter_mut().enumerate() {
            out_l = allpass_process(out_l, allpass_l, ALLPASS_FB, ALLPASS_N[i]);
            out_r = allpass_process(out_r, allpass_r, ALLPASS_FB, ALLPASS_N[i] + self.stereo_spread);
        }

        let left = out_l * self.wet1 + out_r * self.wet2 + input * self.dry;
        let right = out_r * self.wet1 + out_l * self.wet2 + input * self.dry;

        (left, right)
    }
}

#[inline]
fn comb_process(input: f32, buf: &mut VecDeque<f32>, filter_state: &mut f32, fb: f32, n: usize, mod_offset: isize, d: f32) -> f32 {
    let y_delayed = buf.get((n as isize - 1 + mod_offset) as usize).unwrap_or(&0.0);
    *filter_state = (1.0 - d) * y_delayed + d * *filter_state;
    let output = input + fb * *filter_state;
    buf.remove(n - 1 + MOD_RANGE / 2);
    buf.push_front(output);

    output
}

#[inline]
fn allpass_process(input: f32, buf: &mut VecDeque<f32>, fb: f32, n: usize) -> f32 {
    let bufout = buf.remove(n - 1).unwrap_or_default();
    let output = -fb *input + bufout;

    buf.push_front(input + bufout * fb);

    output
}