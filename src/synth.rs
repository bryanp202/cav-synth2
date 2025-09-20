mod midi;

use std::sync::mpsc::{self, Sender};
use midir::MidiInputConnection;
use sdl3::AudioSubsystem;
use crate::audio::AudioMessage;

pub struct Synth {
    audio_channel: Sender<AudioMessage>,
    midi_connection: MidiInputConnection<()>,
    stream: cpal::Stream,
}

impl Synth {
    pub fn init() -> Self {
        let (audio_sender, audio_receiver) = mpsc::channel();
        let stream = crate::audio::init(audio_receiver).expect("Failed to initialize audio thread");
        println!("Audio thread initialized");
        let midi_connection = midi::setup_midi(audio_sender.clone()).expect("Failed to setup midi connection");
        println!("Midi connected");

        Self {
            audio_channel: audio_sender,
            midi_connection,
            stream,
        }
    }
}