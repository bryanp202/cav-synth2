mod midi;

use std::sync::mpsc::{self, Sender};
use midir::MidiInputConnection;
use crate::audio::AudioMessage;

pub struct Synth {
    audio_channel: Sender<AudioMessage>,
    _midi_connection: MidiInputConnection<()>,
    _stream: cpal::Stream,
}

impl Synth {
    pub fn init() -> Self {
        let (audio_sender, audio_receiver) = mpsc::channel();
        let _stream = crate::audio::init(audio_receiver).expect("Failed to initialize audio thread");
        println!("Audio thread initialized");
        let _midi_connection = midi::setup_midi(audio_sender.clone()).expect("Failed to setup midi connection");
        println!("Midi connected");

        Self {
            audio_channel: audio_sender,
            _midi_connection,
            _stream,
        }
    }
}