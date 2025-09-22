use std::collections::VecDeque;

const POLY_VOICE_COUNT: usize = 32;
const GATE: usize = 0;
const NOTE: usize = 1;
const VELOCITY: usize = 2;
const OUTPUT_COUNT: usize = 3;

#[derive(Clone, Copy, Default, Debug)]
struct Voice {
    pressed: bool,
    outputs: usize,
    trigger: bool,
    ready: bool,
    on: bool,
}

impl Voice {
    fn new(output: &mut Vec<f32>) -> Self {
        let output_pos = output.len();
        output.resize(output.len() + OUTPUT_COUNT, 0.0);
        Self {
            pressed: false,
            outputs: output_pos,
            trigger: false,
            ready: false,
            on: false,
        }
    }

    fn update(&mut self, output: &mut [f32], note: u8, velocity: u8) {
        self.pressed = true;
        self.trigger = true;
        self.ready = false;
        self.on = true;
        output[self.outputs + NOTE] = note as f32 / 127.0;
        output[self.outputs + GATE] = 0.0;
        output[self.outputs + VELOCITY] = velocity as f32 / 127.0;
    }
}

pub struct Midi {
    trigger: bool,
    ready: bool,
    outputs: usize,

    // Controls
    sustain: bool,
    pressed: bool,

    // Poly voices
    voices: [Voice; POLY_VOICE_COUNT],
    next: usize,
    
    replace_queue: VecDeque<usize>,
}

impl Midi {
    pub fn process(&mut self, output: &mut [f32]) {
        if self.trigger {
            self.ready = true;
            self.trigger = false;
        } else if self.ready {
            output[self.outputs + GATE] = 1.0;
            self.ready = false;
        }
        
        for voice in &mut self.voices {
            if voice.trigger {
                voice.ready = true;
                voice.trigger = false;
            } else if voice.ready {
                output[voice.outputs + GATE] = 1.0;
                voice.ready = false;
            }
        }
    }

    pub fn key_press(&mut self, output: &mut [f32], note: u8, velocity: u8) {
        // Mono
        self.pressed = true;
        output[self.outputs + GATE] = 0.0;
        self.trigger = true;
        output[self.outputs + NOTE] = note as f32 / 127.0;
        output[self.outputs + VELOCITY] = velocity as f32 / 127.0;

        // Poly
        let new_voice;
        if let Some(voice) = self.voices.iter().cycle().skip(self.next).take(POLY_VOICE_COUNT).position(|voice| !voice.on ) {
            new_voice = (voice + self.next) % POLY_VOICE_COUNT;
            self.next = (self.next + 1) % POLY_VOICE_COUNT;
        } else {
            new_voice = self.replace_queue.pop_front().unwrap();
        }
        self.voices[new_voice].update(output, velocity, note);
        self.replace_queue.push_back(new_voice);
    }

    pub fn key_release(&mut self, output: &mut [f32], note: u8) {
        let note_signal = note as f32 / 127.0;
        if output[self.outputs + NOTE] == note_signal {
            if !self.sustain {
                output[self.outputs + GATE] = 0.0;
                self.ready = false;
                self.trigger = false;
            }
            self.pressed = false;
        }

        // Poly
        for (i, voice) in self.voices.iter_mut().enumerate() {
            if output[voice.outputs + NOTE] == note_signal {
                if !self.sustain && voice.on {
                    voice.ready = false;
                    voice.trigger = false;
                    output[voice.outputs + GATE] = 0.0;
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
        if !self.pressed {
            output[self.outputs + GATE] = 0.0;
            self.ready = false;
            self.trigger = false;
        }

        // Poly
        for (i, voice) in self.voices.iter_mut().enumerate() {
            if voice.on && !voice.pressed {
                voice.ready = false;
                voice.trigger = false;
                output[voice.outputs + GATE] = 0.0;
                voice.on = false;
                
                let queue_pos = self.replace_queue.iter().position(|voice_num| *voice_num == i).unwrap();
                self.replace_queue.remove(queue_pos);
            }
        }
    }
}

impl Midi {
    pub fn new(output: &mut Vec<f32>) -> Midi {
        let output_pos = output.len();
        output.resize(output.len() + OUTPUT_COUNT, 0.0);
        Self {
            outputs: output_pos,
            trigger: false,
            ready: false,
            sustain: false,
            pressed: false,

            voices: std::array::from_fn(|_| Voice::new(output)),
            replace_queue: VecDeque::with_capacity(POLY_VOICE_COUNT),
            next: 0,
        }
    }

    pub fn voices(&self) -> usize {
        self.voices.len()
    }

    pub fn get_voice_gate(&self, index: usize) -> usize {
        self.voices[index].outputs + GATE
    }

    pub fn get_voice_note(&self, index: usize) -> usize {
        self.voices[index].outputs + NOTE
    }

    pub fn get_voice_velocity(&self, index: usize) -> usize {
        self.voices[index].outputs + VELOCITY
    }
}