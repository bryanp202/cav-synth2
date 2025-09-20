#[derive(Default)]
pub struct Cable {
    source: usize,
    target: usize,
    gain: f32,
}

impl Cable {
    pub fn new(source: usize, target: usize, gain: f32) -> Self {
        Self {
            source,
            target,
            gain,
        }
    }
}

pub fn cable_system(cables: &[Cable], inputs: &mut [f32], outputs: &[f32]) {
    inputs.fill(0.0);
    for cable in cables {
        inputs[cable.target] += cable.gain * outputs[cable.source];
    }
}