use std::time::Instant;

const GATE: usize = 0;
const VELOCITY: usize = 1;
// const ATTACK: usize = 2;
// const DECAY: usize = 3;
// const RELEASE: usize = 4;
// const SUSTAIN: usize = 5;
const INPUT_COUNT: usize = 2;

const OUT_VALUE: usize = 0;
const OUTPUT_COUNT: usize = 1;

#[derive(Default)]
pub struct Envelope {
    start: Option<Instant>,
    released: Option<Instant>,
    release_start_value: f32,
    attack: f32,
    decay: f32,
    release: f32,
    sustain: f32,
    inputs: usize,
    outputs: usize,
}

impl Envelope {
    pub fn new(inputs: &mut Vec<f32>, outputs: &mut Vec<f32>) -> Self {
        let env_inputs = inputs.len();
        let env_outputs = outputs.len();
        inputs.resize(inputs.len() + INPUT_COUNT, 0.0);
        outputs.resize(outputs.len() + OUTPUT_COUNT, 0.0);
        Self {
            inputs: env_inputs,
            outputs: env_outputs,
            attack: 0.02,
            decay: 1.6,
            release: 0.5,
            sustain: 0.0,
            ..Default::default()
        }
    }

    pub fn get_output(&self) -> usize {
        self.outputs
    }

    pub fn get_gate_input(&self) -> usize {
        self.inputs + GATE
    }

    pub fn get_velocity_input(&self) -> usize {
        self.inputs + VELOCITY
    }
}

pub fn envelope_system(envelopes: &mut [Envelope], inputs: &[f32], outputs: &mut [f32]) {
    for envelope in envelopes {
        let velocity = inputs[envelope.inputs + VELOCITY];
        let gate = inputs[envelope.inputs + GATE];
        if gate != 0.0 {
            if let None = envelope.start {
                envelope.start = Some(Instant::now());
                envelope.released = None;
            }
        } else {
            if let None = envelope.released {
                if let Some(_) = envelope.start {
                    envelope.start = None;
                    envelope.released = Some(Instant::now());
                    envelope.release_start_value = outputs[envelope.outputs] / velocity;
                }
            }
        }
        let out = if let Some(start_time) = envelope.start {
            let elapsed = start_time.elapsed().as_secs_f32();
            if elapsed < envelope.attack {
                1.0 * elapsed / envelope.attack
            } else if elapsed - envelope.attack < envelope.decay {
                let since_decay = elapsed - envelope.attack;
                let peak_sustain_delta = 1.0 - envelope.sustain;

                1.0 - peak_sustain_delta * since_decay / envelope.decay
            } else {
                envelope.sustain
            }
        } else if let Some(released_time) = envelope.released {
            let elapsed = released_time.elapsed().as_secs_f32();
            let elapsed_ratio = elapsed / envelope.release;
    
            if elapsed_ratio < 1.0 {
                envelope.release_start_value * (1.0 - (elapsed_ratio).powf(0.4))
            } else {
                envelope.released = None;
                0.0
            }
        } else {
            0.0
        };

        outputs[envelope.outputs + OUT_VALUE] = out * velocity;
    }
}