use std::time::Instant;

use crate::audio::MAX_POLY_COUNT;

pub const GATE_INPUT: usize = 0 * MAX_POLY_COUNT;
pub const VELOCITY_INPUT: usize = 1 * MAX_POLY_COUNT;
pub const ATTACK_INPUT: usize = 2 * MAX_POLY_COUNT;
pub const DECAY_INPUT: usize = 3 * MAX_POLY_COUNT;
pub const RELEASE_INPUT: usize = 4 * MAX_POLY_COUNT;
pub const SUSTAIN_INPUT: usize = 5 * MAX_POLY_COUNT;
pub const TOTAL_INPUT_COUNT: usize = 6 * MAX_POLY_COUNT;

pub const OUT_VALUE: usize = 0 * MAX_POLY_COUNT;
pub const TOTAL_OUTPUT_COUNT: usize = 1 * MAX_POLY_COUNT;

#[derive(Clone, Copy, Default)]
struct EnvelopeMetaData {
    start: Option<Instant>,
    released: Option<Instant>,
    release_start_value: f32,
}

pub struct PolyEnvelope<const INPUT_OFFSET: usize, const OUTPUT_OFFSET: usize> {
    envelopes: [EnvelopeMetaData; MAX_POLY_COUNT],
    attack: f32,
    decay: f32,
    release: f32,
    sustain: f32,
}

impl <const INPUT_OFFSET: usize, const OUTPUT_OFFSET: usize> PolyEnvelope <INPUT_OFFSET, OUTPUT_OFFSET> {
    pub fn new() -> Self {
        Self {
            envelopes: [EnvelopeMetaData::default(); MAX_POLY_COUNT],
            attack: 0.015,
            decay: 2.6,
            release: 0.5,
            sustain: 1.0,
        }
    }

    pub fn set_attack_value(&mut self, attack: f32) {
        self.attack = attack;
    }

    pub fn set_decay_value(&mut self, decay: f32) {
        self.decay = decay;
        println!("decay: {}", decay);
    }

    pub fn set_sustain_value(&mut self, sustain: f32) {
        self.sustain = sustain;
    }

    pub fn set_release_value(&mut self, release: f32) {
        self.release = release;
    }

    pub fn render(&mut self, inputs: &[f32], outputs: &mut [f32]) {
        for (envelope, meta) in self.envelopes.iter_mut().enumerate() {
            let velocity = inputs[INPUT_OFFSET + VELOCITY_INPUT + envelope];
            let gate = inputs[INPUT_OFFSET + GATE_INPUT + envelope];
            let attack = self.attack + inputs[INPUT_OFFSET + ATTACK_INPUT + envelope];
            let decay = self.decay + inputs[INPUT_OFFSET + DECAY_INPUT + envelope];
            let sustain = self.sustain + inputs[INPUT_OFFSET + SUSTAIN_INPUT + envelope];
            let release = self.release + inputs[INPUT_OFFSET + RELEASE_INPUT + envelope];

            let raw = if let Some(start_time) = meta.start {
                let elapsed = start_time.elapsed().as_secs_f32();
                if elapsed < attack {
                    1.0 * elapsed / attack
                } else if elapsed - attack < decay {
                    let since_decay = elapsed - attack;
                    let peak_sustain_delta = 1.0 - sustain;

                    1.0 - peak_sustain_delta * since_decay / decay
                } else {
                    sustain
                }
            } else if let Some(released_time) = meta.released {
                let elapsed = released_time.elapsed().as_secs_f32();
                let elapsed_ratio = elapsed / release;
        
                if elapsed_ratio < 1.0 {
                    meta.release_start_value * (1.0 - (elapsed_ratio).powf(0.4))
                } else {
                    meta.released = None;
                    0.0
                }
            } else {
                0.0
            };

            if gate > 0.0 {
                if let None = meta.start {
                    meta.start = Some(Instant::now());
                    meta.released = None;
                }
            } else {
                if let None = meta.released {
                    if let Some(_) = meta.start {
                        meta.start = None;
                        meta.released = Some(Instant::now());
                        meta.release_start_value = raw;
                    }
                }
            }

            outputs[OUTPUT_OFFSET + OUT_VALUE + envelope] = raw * velocity;
        }
    }
}