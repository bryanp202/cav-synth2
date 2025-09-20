mod component;
mod midi;

use std::ops::{Deref, DerefMut};
use std::sync::mpsc;
use std::time::{Duration, Instant};

use component::envelope::Envelope;
use component::analog::AnalogOscillator;
use component::cable::Cable;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, FromSample, Host, SizedSample, StreamConfig, SupportedStreamConfig, I24};

use crate::audio::component::{analog, envelope};
use crate::audio::midi::Midi;

pub fn init(receiver: mpsc::Receiver<AudioMessage>) -> Result<cpal::Stream, String> {
    let stream = stream_setup(receiver)?;
    stream.play().map_err(|err| format!("Error on output stream play: {err}"))?;
    Ok(stream)
}

fn stream_setup(receiver: mpsc::Receiver<AudioMessage>) -> Result<cpal::Stream, String> {
    let (_host, device, config) = host_device_setup()?;

     match config.sample_format() {
        cpal::SampleFormat::I8 => make_stream::<i8>(receiver, &device, &config.into()),
        cpal::SampleFormat::I16 => make_stream::<i16>(receiver, &device, &config.into()),
        cpal::SampleFormat::I24 => make_stream::<I24>(receiver, &device, &config.into()),
        cpal::SampleFormat::I32 => make_stream::<i32>(receiver, &device, &config.into()),
        cpal::SampleFormat::I64 => make_stream::<i64>(receiver, &device, &config.into()),
        cpal::SampleFormat::U8 => make_stream::<u8>(receiver, &device, &config.into()),
        cpal::SampleFormat::U16 => make_stream::<u16>(receiver, &device, &config.into()),
        cpal::SampleFormat::U32 => make_stream::<u32>(receiver, &device, &config.into()),
        cpal::SampleFormat::U64 => make_stream::<u64>(receiver, &device, &config.into()),
        cpal::SampleFormat::F32 => make_stream::<f32>(receiver, &device, &config.into()),
        cpal::SampleFormat::F64 => make_stream::<f64>(receiver, &device, &config.into()),
        sample_format => Err(format!(
            "Unsupported sample format '{sample_format}'"
        )),
    }
}

fn host_device_setup() -> Result<(Host, Device, SupportedStreamConfig), String> {
    let host = cpal::default_host();
    let device = host.default_output_device().ok_or_else(|| String::from("Default output device is not available"))?;
    let config = device
        .default_output_config()
        .map_err(|err| format!("Default output config is unavailable: {}", err))?;
    Ok((host, device, config))
}

fn make_stream<T>(receiver: mpsc::Receiver<AudioMessage>, device: &Device, config: &StreamConfig) -> Result<cpal::Stream, String>
where 
    T: SizedSample + FromSample<f32>,
{
    let num_channels = config.channels as usize;
    assert!(num_channels == 2);
    let sample_rate = config.sample_rate.0 as usize;
    let mut audio_state = AudioState::new(receiver, sample_rate);

    let err_fn = |err| eprintln!("Erroring building output sound stream: {err}");
    device.build_output_stream(
        config,
        move |output: &mut [T], _: &cpal::OutputCallbackInfo| {
            for i in 0..output.len() / 2 {
                let (left, right) = audio_state.process();
                output[i*2] = T::from_sample(left);
                output[i*2 + 1] = T::from_sample(right);
            }
            audio_state.update();
        },
        err_fn,
        None
    ).map_err(|err| format!("Failed to create audio output stream: {err}"))
}

const BUFFER_SIZE: usize = 256;
const MAX_OSCILLATORS: usize = 16;
const MAX_ENVELOPES: usize = 16;
const MAX_CABLES: usize = 2048;

#[derive(Clone, Copy, Debug)]
pub enum AudioMessage {
    KeyPress(u8, u8),
    KeyRelease(u8),
    PedalPress,
    PedalRelease,
}

struct ComponentVec<T: Default, const MAX: usize> {
    components: [T; MAX],
    count: usize,
}

impl <T: Default, const MAX: usize> ComponentVec<T, MAX> {
    pub fn new() -> Self {
        Self {
            components: std::array::from_fn(|_| T::default()),
            count: 0,
        }
    }

    pub fn push(&mut self, new_component: T) -> Result<(), ()> {
        if self.count == MAX {
            return Err(());
        }
        self.components[self.count] = new_component;
        self.count += 1;
        Ok(())
    }

    pub fn remove(&mut self, index: usize) -> Result<T, ()> {
        if index >= self.count {
            return Err(());
        }
        let removed = std::mem::take(&mut self.components[index]);
        if index + 1 != self.count {
            self.count -= 1;
            self.components.swap(index, self.count);
        }
        Ok(removed)
    }

    pub fn get(&self, index: usize) -> &T {
        &self.components[index]
    }

    pub fn get_mut(&mut self, index: usize) -> &mut T {
        &mut self.components[index]
    }
}

impl <T: Default, const MAX: usize> Deref for ComponentVec<T, MAX> {
    type Target = [T];
    fn deref(&self) -> &Self::Target {
        &self.components[0..self.count]   
    }
}

impl <T: Default, const MAX: usize> DerefMut for ComponentVec<T, MAX> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.components[0..self.count]
    }
}

struct AudioState {
    receiver: mpsc::Receiver<AudioMessage>,
    sample_rate: usize,
    inputs: Vec<f32>,
    outputs: Vec<f32>,
    midi: Midi,
    analogs: ComponentVec<AnalogOscillator, MAX_OSCILLATORS>,
    envelopes: ComponentVec<Envelope, MAX_ENVELOPES>,
    cables: ComponentVec<Cable, MAX_CABLES>,
}

impl AudioState {
    pub fn new(receiver: mpsc::Receiver<AudioMessage>, sample_rate: usize) -> Self {
        let mut outputs = Vec::new();
        let midi = Midi::new(&mut outputs);
        let mut new_state = Self {
            receiver,
            sample_rate,
            inputs: vec![0.0, 0.0],
            outputs,
            midi,
            analogs: ComponentVec::new(),
            cables: ComponentVec::new(),
            envelopes: ComponentVec::new(),
        };

        for i in 0..new_state.midi.voices() {
            new_state.envelopes.push(Envelope::new(&mut new_state.inputs, &mut new_state.outputs)).unwrap();
            new_state.cables.push(
                Cable::new(new_state.midi.get_voice_gate(i), new_state.envelopes[i].get_inputs() + envelope::GATE, 1.0)
            ).unwrap();
            new_state.cables.push(
                Cable::new(new_state.midi.get_voice_velocity(i), new_state.envelopes[i].get_inputs() + envelope::VELOCITY, 1.0)
            ).unwrap();
        }
        for i in 0..new_state.midi.voices() {
            new_state.analogs.push(AnalogOscillator::new(&mut new_state.inputs, &mut new_state.outputs, new_state.sample_rate as f64)).unwrap();
            new_state.cables.push(
                Cable::new(new_state.midi.get_voice_note(i), new_state.analogs[i].get_inputs() + analog::FREQUENCY, 1.0)
            ).unwrap();
            new_state.cables.push(
                Cable::new(new_state.envelopes[i].get_outputs(), new_state.analogs[i].get_inputs(), 1.0)
            ).unwrap();
        }
        for analog in new_state.analogs.iter() {
            new_state.cables.push(Cable::new(analog.get_outputs(), 0, 0.3)).unwrap();
            new_state.cables.push(Cable::new(analog.get_outputs(), 1, 0.3)).unwrap();
        }
        new_state
    }
}

impl AudioState {
    fn process(&mut self) -> (f32, f32) {
        self.midi.process(&mut self.outputs);
        component::analog::analog_oscillator_system(&mut self.analogs, &self.inputs, &mut self.outputs);
        component::envelope::envelope_system(&mut self.envelopes, &self.inputs, &mut self.outputs);
        component::cable::cable_system(&self.cables, &mut self.inputs, &self.outputs);
        (self.inputs[0], self.inputs[1])
    }

    fn update(&mut self) {
        for msg in self.receiver.try_iter() {
            match msg {
                AudioMessage::KeyPress(velocity, note) => self.midi.key_press(&mut self.outputs, note, velocity),
                AudioMessage::KeyRelease(note) => self.midi.key_release(&mut self.outputs, note),
                AudioMessage::PedalPress => self.midi.pedal_press(),
                AudioMessage::PedalRelease => self.midi.pedal_release(&mut self.outputs),
            }
        }
    }
}


