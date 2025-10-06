mod midi;

use std::sync::mpsc;
use midir::MidiInputConnection;
use sdl3::{event::Event, mouse::MouseButton, render::{Canvas, TextureCreator}, video::{Window, WindowContext}, Error, EventPump};
use crate::gui::Gui;

pub enum SynthMessage {
    MasterMeter(f32, f32), // Need to be sqrted on use
}

pub struct Synth<'a> {
    canvas: Canvas<Window>,
    event_pump: EventPump,
    audio_events: mpsc::Receiver<SynthMessage>,

    gui: Gui<'a>,
    _midi_connection: Option<MidiInputConnection<()>>,
    _stream: cpal::Stream,

    should_quit: bool,
}

impl <'a> Synth<'a> {
    pub fn init(canvas: Canvas<Window>, event_pump: EventPump, texture_creator: &'a TextureCreator<WindowContext>) -> Self {
        let (audio_sender, audio_receiver) = mpsc::channel();
        let (synth_sender, audio_events) = mpsc::channel();
        let _stream = crate::audio::init(audio_receiver, synth_sender).expect("Failed to initialize audio thread");
        let _midi_connection = midi::setup_midi(audio_sender.clone())
            .map_err(|err| eprintln!("{}", err))
            .ok();
        audio_sender.send(crate::audio::AudioMessage::KeyPress(60, 60)).unwrap();
        let gui = Gui::new(audio_sender, texture_creator);

        let mut new_synth = Self {
            canvas,
            event_pump,
            audio_events,
            _midi_connection,
            _stream,
            gui,
            should_quit: false,
        };
        new_synth.gui.init();
        new_synth
    }

    pub fn update(&mut self) {
        for event in self.event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => self.should_quit = true,
                Event::TextInput { text, .. } => self.gui.text_input(text),
                Event::MouseButtonDown { mouse_btn, clicks, x, y, .. } => {
                    match mouse_btn {
                        MouseButton::Left => self.gui.left_mouse_down(x, y, clicks),
                        MouseButton::Right => self.gui.right_mouse_down(x, y, clicks),
                        _ => {},
                    }
                },
                Event::MouseMotion { x, y, xrel, yrel, .. } => self.gui.mouse_move(x, y, xrel, yrel),
                Event::MouseButtonUp { mouse_btn, clicks, .. } => {
                    match mouse_btn {
                        MouseButton::Left => self.gui.left_mouse_up(clicks),
                        MouseButton::Right => self.gui.right_mouse_up(clicks),
                        _ => {},
                    };
                },
                _ => {},
            }
        }
        for msg in self.audio_events.try_iter() {
            match msg {
                SynthMessage::MasterMeter(left, right) => self.gui.master_meter(left, right),
            }
        }
    }

    pub fn render(&mut self) -> Result<(), Error> {
        self.gui.render(&mut self.canvas)
    }

    pub fn should_quit(&self) -> bool {
        self.should_quit
    }
}