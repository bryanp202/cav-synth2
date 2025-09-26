mod component;
mod midi;

use std::sync::mpsc;

use component::envelope::Envelope;
use component::analog::AnalogOscillator;
use component::cable::Cable;
use component::filter::Filter;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, FromSample, Host, SizedSample, StreamConfig, SupportedStreamConfig, I24};
use crate::common::ComponentVec;

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
    let sample_rate = config.sample_rate.0 as f64;
    let mut audio_state = AudioState::new(receiver, sample_rate);

    let err_fn = |err| eprintln!("Erroring building output sound stream: {err}");
    device.build_output_stream(
        config,
        move |output: &mut [T], _: &cpal::OutputCallbackInfo| {
            for i in 0..output.len() / 2 {
                let (left, right) = audio_state.process();
                output[i*2] = T::from_sample(left.clamp(-1.0, 1.0));
                output[i*2 + 1] = T::from_sample(right.clamp(-1.0, 1.0));
            }
            audio_state.update();
        },
        err_fn,
        None
    ).map_err(|err| format!("Failed to create audio output stream: {err}"))
}

const MAX_OSCILLATORS: usize = 128;
const MAX_ENVELOPES: usize = 256;
const MAX_FILTERS: usize = 128;
const MAX_CABLES: usize = 4096;

#[derive(Clone, Copy, Debug)]
pub enum AudioMessage {
    Osc1Freq(f32),
    KeyPress(u8, u8),
    KeyRelease(u8),
    PedalPress,
    PedalRelease,
}

struct AudioState {
    receiver: mpsc::Receiver<AudioMessage>,
    sample_rate: f64,
    inputs: Vec<f32>,
    outputs: Vec<f32>,
    midi: Midi,
    analogs: ComponentVec<AnalogOscillator, MAX_OSCILLATORS>,
    envelopes: ComponentVec<Envelope, MAX_ENVELOPES>,
    filters: ComponentVec<Filter, MAX_FILTERS>,
    cables: ComponentVec<Cable, MAX_CABLES>,
}

impl AudioState {
    pub fn new(receiver: mpsc::Receiver<AudioMessage>, sample_rate: f64) -> Self {
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
            filters: ComponentVec::new(),
        };

        new_state.init();
        new_state
    }

    pub fn init(&mut self) {
        for i in 0..self.midi.voices() {
            self.envelopes.push(Envelope::new(&mut self.inputs, &mut self.outputs)).unwrap();
            self.cables.push(
                Cable::new(self.midi.get_voice_gate(i), self.envelopes[i].get_gate_input(), 1.0)
            ).unwrap();
            self.cables.push(
                Cable::new(self.midi.get_voice_velocity(i), self.envelopes[i].get_velocity_input(), 1.0)
            ).unwrap();
        }
        for i in 0..self.midi.voices() {
            self.analogs.push(AnalogOscillator::new(&mut self.inputs, &mut self.outputs)).unwrap();
            self.cables.push(
                Cable::new(self.midi.get_voice_note(i), self.analogs[i].get_freq_input(), 1.0)
            ).unwrap();
            self.cables.push(
                Cable::new(self.envelopes[i].get_output(), self.analogs[i].get_level_input(), 1.0)
            ).unwrap();
        }
        for i in 0..self.midi.voices() {
            self.envelopes.push(Envelope::new(&mut self.inputs, &mut self.outputs)).unwrap();
            self.cables.push(
                Cable::new(self.midi.get_voice_gate(i), self.envelopes[i + self.midi.voices()].get_gate_input(), 1.0)
            ).unwrap();
            self.cables.push(
                Cable::new(self.midi.get_voice_velocity(i), self.envelopes[i + self.midi.voices()].get_velocity_input(), 1.0)
            ).unwrap();
            self.filters.push(Filter::new(&mut self.inputs, &mut self.outputs)).unwrap();
            self.cables.push(
                Cable::new(self.analogs[i].get_output(), self.filters[i].get_value_input(), 1.0)
            ).unwrap();
            self.cables.push(
                Cable::new(self.envelopes[i + self.midi.voices()].get_output(), self.filters[i].get_freq_input(), 0.7)
            ).unwrap();
            self.cables.push(
                Cable::new(self.midi.get_voice_note(i), self.filters[i].get_freq_input(), 1.0)
            ).unwrap();
        }
        for filter in self.filters.iter() {
            self.cables.push(Cable::new(filter.get_output(), 0, 0.3)).unwrap();
            self.cables.push(Cable::new(filter.get_output(), 1, 0.3)).unwrap();
        }
    }
}

impl AudioState {
    fn process(&mut self) -> (f32, f32) {
        self.midi.process(&mut self.outputs);
        component::analog::analog_oscillator_system(&mut self.analogs, &self.inputs, &mut self.outputs, self.sample_rate);
        component::envelope::envelope_system(&mut self.envelopes, &self.inputs, &mut self.outputs);
        component::filter::butterworth_system(&mut self.filters, &self.inputs, &mut self.outputs, self.sample_rate);
        component::cable::cable_system(&self.cables, &mut self.inputs, &self.outputs);
        (self.inputs[0], self.inputs[1])
    }

    fn update(&mut self) {
        for msg in self.receiver.try_iter() {
            match msg {
                AudioMessage::Osc1Freq(freq) => {
                    for analog in &mut self.analogs as &mut [AnalogOscillator] {
                        analog.set_freq_value((freq - 0.5) / 10.0);
                    }
                },
                AudioMessage::KeyPress(velocity, note) => self.midi.key_press(&mut self.outputs, note, velocity),
                AudioMessage::KeyRelease(note) => self.midi.key_release(&mut self.outputs, note),
                AudioMessage::PedalPress => self.midi.pedal_press(),
                AudioMessage::PedalRelease => self.midi.pedal_release(&mut self.outputs),
            }
        }
    }
}


