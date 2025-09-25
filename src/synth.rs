mod midi;

use std::sync::mpsc;
use midir::MidiInputConnection;
use sdl3::{event::Event, mouse::MouseButton, render::{Canvas, TextureCreator}, video::{Window, WindowContext}, Error, EventPump};
use crate::gui::Gui;

pub struct Synth<'a> {
    canvas: Canvas<Window>,
    event_pump: EventPump,

    gui: Gui<'a>,
    _midi_connection: Option<MidiInputConnection<()>>,
    _stream: cpal::Stream,

    should_quit: bool,
}

impl <'a> Synth<'a> {
    pub fn init(canvas: Canvas<Window>, event_pump: EventPump, texture_creator: &'a TextureCreator<WindowContext>) -> Self {
        let (audio_sender, audio_receiver) = mpsc::channel();
        let _stream = crate::audio::init(audio_receiver).expect("Failed to initialize audio thread");
        let _midi_connection = midi::setup_midi(audio_sender.clone())
            .map_err(|err| eprintln!("{}", err))
            .ok();
        let gui = Gui::new(audio_sender, texture_creator);

        let mut new_synth = Self {
            canvas,
            event_pump,
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
                        _ => {},
                    };
                },
                _ => {},
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