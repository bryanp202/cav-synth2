use std::collections::VecDeque;

use crate::audio::{MAX_POLY_COUNT, MIDI_OUTPUT_OFFSET};

pub const GATE_OUTPUT: usize = 0 * MAX_POLY_COUNT;
pub const NOTE_OUTPUT: usize = 1 * MAX_POLY_COUNT;
pub const VELOCITY_OUTPUT: usize = 2 * MAX_POLY_COUNT;
pub const TOTAL_OUTPUT_COUNT: usize = 3 * MAX_POLY_COUNT;

#[derive(Clone, Copy, Default, Debug)]
struct Voice {
    pressed: bool,
    trigger: bool,
    ready: bool,
    on: bool,
}

impl Voice {
    fn new() -> Self {
        Self {
            pressed: false,
            trigger: false,
            ready: false,
            on: false,
        }
    }

    fn update(&mut self, output: &mut [f32], voice_index: usize, note: u8, velocity: u8) {
        self.pressed = true;
        self.trigger = true;
        self.ready = false;
        self.on = true;
        output[MIDI_OUTPUT_OFFSET + NOTE_OUTPUT + voice_index] = note as f32 / 128.0;
        output[MIDI_OUTPUT_OFFSET + GATE_OUTPUT + voice_index] = 0.0;
        output[MIDI_OUTPUT_OFFSET + VELOCITY_OUTPUT + voice_index] = velocity as f32 / 128.0;
    }
}

pub struct Midi {
    // Controls
    sustain: bool,

    // Poly voices
    voices: [Voice; MAX_POLY_COUNT],
    next: usize,
    
    replace_queue: VecDeque<usize>,
}

impl Midi {
    pub fn process(&mut self, output: &mut [f32]) {
        for (i, voice) in self.voices.iter_mut().enumerate() {
            if voice.trigger {
                voice.ready = true;
                voice.trigger = false;
            } else if voice.ready {
                output[MIDI_OUTPUT_OFFSET + GATE_OUTPUT + i] = 1.0;
                voice.ready = false;
            }
        }
    }

    pub fn key_press(&mut self, output: &mut [f32], note: u8, velocity: u8) {
        // Poly
        let new_voice;
        if let Some(voice) = self.voices.iter().cycle().skip(self.next).take(MAX_POLY_COUNT).position(|voice| !voice.on ) {
            new_voice = (voice + self.next) % MAX_POLY_COUNT;
            self.next = (self.next + 1) % MAX_POLY_COUNT;
        } else {
            new_voice = self.replace_queue.pop_front().unwrap();
        }
        self.voices[new_voice].update(output, new_voice, velocity, note);
        self.replace_queue.push_back(new_voice);
    }

    pub fn key_release(&mut self, output: &mut [f32], note: u8) {
        let note_signal = note as f32 / 128.0;
        // Poly
        for (i, voice) in self.voices.iter_mut().enumerate() {
            if output[MIDI_OUTPUT_OFFSET + NOTE_OUTPUT + i] == note_signal {
                if !self.sustain && voice.on {
                    voice.ready = false;
                    voice.trigger = false;
                    output[MIDI_OUTPUT_OFFSET + GATE_OUTPUT + i] = 0.0;
                    voice.on = false;
                    
                    let queue_pos = self.replace_queue.iter().position(|voice_num| *voice_num == i).unwrap();
                    self.replace_queue.remove(queue_pos);
                }
                voice.pressed = false;
            }
        }
    }

    pub fn pedal_press(&mut self) {
        self.sustain = true;
    }

    pub fn pedal_release(&mut self, output: &mut [f32]) {
        self.sustain = false; 

        // Poly
        for (i, voice) in self.voices.iter_mut().enumerate() {
            if voice.on && !voice.pressed {
                voice.ready = false;
                voice.trigger = false;
                output[MIDI_OUTPUT_OFFSET + GATE_OUTPUT + i] = 0.0;
                voice.on = false;
                
                let queue_pos = self.replace_queue.iter().position(|voice_num| *voice_num == i).unwrap();
                self.replace_queue.remove(queue_pos);
            }
        }
    }
}

impl Midi {
    pub fn new() -> Midi {
        Self {
            sustain: false,

            voices: std::array::from_fn(|_| Voice::new()),
            replace_queue: VecDeque::with_capacity(MAX_POLY_COUNT),
            next: 0,
        }
    }
}