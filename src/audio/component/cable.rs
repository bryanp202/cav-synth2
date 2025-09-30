use crate::{audio::MAX_POLY_COUNT, common::ComponentVec};

pub struct Cables <const MAX_CABLES: usize> (ComponentVec <Cable, MAX_CABLES>);

impl <const MAX_CABLES: usize> Cables <MAX_CABLES> {
    pub fn new() -> Self {
        Self (ComponentVec::new())
    }

    pub fn add_cable(&mut self, source: usize, target: usize) -> Result<(), ()> {
        self.0.push(Cable::new(source, target))
    }

    pub fn run_cables(&self, inputs: &mut [f32], outputs: &[f32]) {
        inputs.fill(0.0);
        for cable in self.0.iter() {
            for i in 0..MAX_POLY_COUNT {
                inputs[cable.target + i] += outputs[cable.source + i];
            }
        }
    }
}

struct Cable {
    source: usize,
    target: usize,
}

impl Cable {
    pub fn new(source: usize, target: usize) -> Self {
        Self {
            source,
            target,
        }
    }
}