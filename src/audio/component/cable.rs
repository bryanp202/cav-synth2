use crate::{audio::{InputJack, OutputJack, MAX_POLY_COUNT}, common::ComponentVec};

pub struct Cables <const MAX_CABLES: usize> (ComponentVec <Cable, MAX_CABLES>);

impl <const MAX_CABLES: usize> Cables <MAX_CABLES> {
    pub fn new() -> Self {
        Self (ComponentVec::new())
    }

    pub fn add_cable(&mut self, source: OutputJack, target: InputJack) -> Result<(), ()> {
        self.0.push(Cable::new(source, target))
    }

    pub fn remove_cable(&mut self, cable_index: usize) {
        self.0.remove(cable_index);
    }

    #[inline(always)]
    pub fn run_cables(&self, inputs: &mut [f32], outputs: &[f32]) {
        inputs.fill(0.0);
        for cable in self.0.iter() {
            for i in 0..MAX_POLY_COUNT {
                inputs[cable.target as usize + i] += outputs[cable.source as usize + i] * cable.gain;
            }
        }
    }

    pub fn attenaute(&mut self, cable_index: usize, new_value: f32) {
        self.0[cable_index].gain = new_value;
    }
}

#[derive(Debug)]
struct Cable {
    source: OutputJack,
    target: InputJack,
    gain: f32,
}

impl Cable {
    pub fn new(source: OutputJack, target: InputJack) -> Self {
        Self {
            source,
            target,
            gain: 1.0,
        }
    }
}