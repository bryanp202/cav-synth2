mod component;
mod midi;

use std::sync::{mpsc, Arc};

use component::envelope::PolyEnvelope;
use component::analog::PolyAnalog;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, FromSample, Host, SizedSample, StreamConfig, SupportedStreamConfig, I24};
use crate::audio::component::cable::Cables;
use crate::audio::component::effects::EffectsChain;
use crate::audio::component::filter::PolyFilter;
use crate::audio::component::{analog, envelope, filter};
use crate::audio::component::wavetable::{self, PolyWavetable};

use crate::audio::midi::Midi;
pub use analog::WaveShape;
pub use wavetable::Wavetable;
pub use wavetable::WAVETABLE_FRAME_LENGTH;

pub const MAX_POLY_COUNT: usize = 16;
const MAX_CABLES: usize = 256;

#[derive(Debug)]
pub enum AudioMessage {
    Osc1Freq(f32),
    Osc1Shape(analog::WaveShape),
    Osc2Freq(f32),
    Osc2WavetableUpdate(Arc<wavetable::Wavetable>),
    DelayTime(f32),
    KeyPress(u8, u8),
    KeyRelease(u8),
    PedalPress,
    PedalRelease,
}

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

const EFFECTS_CHAIN_INPUT_OFFSET: usize = 0;
const OSC1_INPUT_OFFSET: usize = MAX_POLY_COUNT;
const OSC2_INPUT_OFFSET: usize = analog::TOTAL_INPUT_COUNT;
const ENV1_INPUT_OFFSET: usize = OSC2_INPUT_OFFSET + wavetable::TOTAL_INPUT_COUNT;
const ENV2_INPUT_OFFSET: usize = ENV1_INPUT_OFFSET + envelope::TOTAL_INPUT_COUNT;
const ENV3_INPUT_OFFSET: usize = ENV2_INPUT_OFFSET + envelope::TOTAL_INPUT_COUNT;
const FILTER1_INPUT_OFFSET: usize = ENV3_INPUT_OFFSET + envelope::TOTAL_INPUT_COUNT;
const FILTER2_INPUT_OFFSET: usize = FILTER1_INPUT_OFFSET + filter::TOTAL_INPUT_COUNT;
const TOTAL_INPUT_COUNT: usize = FILTER2_INPUT_OFFSET + filter::TOTAL_INPUT_COUNT;

const MIDI_OUTPUT_OFFSET: usize = 0;
const OSC1_OUTPUT_OFFSET: usize =  MIDI_OUTPUT_OFFSET + midi::TOTAL_OUTPUT_COUNT;
const OSC2_OUTPUT_OFFSET: usize = OSC1_OUTPUT_OFFSET + analog::TOTAL_OUTPUT_COUNT;
const ENV1_OUTPUT_OFFSET: usize = OSC2_OUTPUT_OFFSET + wavetable::TOTAL_OUTPUT_COUNT;
const ENV2_OUTPUT_OFFSET: usize = ENV1_OUTPUT_OFFSET + envelope::TOTAL_OUTPUT_COUNT;
const ENV3_OUTPUT_OFFSET: usize = ENV2_OUTPUT_OFFSET + envelope::TOTAL_OUTPUT_COUNT;
const FILTER1_OUTPUT_OFFSET: usize = ENV3_OUTPUT_OFFSET + envelope::TOTAL_OUTPUT_COUNT;
const FILTER2_OUTPUT_OFFSET: usize = FILTER1_OUTPUT_OFFSET + filter::TOTAL_OUTPUT_COUNT;
const TOTAL_OUTPUT_COUNT: usize = FILTER2_OUTPUT_OFFSET + filter::TOTAL_OUTPUT_COUNT;

struct AudioState {
    receiver: mpsc::Receiver<AudioMessage>,
    sample_rate: f64,
    inputs: [f32; TOTAL_INPUT_COUNT],
    outputs: [f32; TOTAL_OUTPUT_COUNT],
    midi: Midi,
    osc1: PolyAnalog<OSC1_INPUT_OFFSET, OSC1_OUTPUT_OFFSET>,
    osc2: PolyWavetable<OSC2_INPUT_OFFSET, OSC2_OUTPUT_OFFSET>,
    env1: PolyEnvelope<ENV1_INPUT_OFFSET, ENV1_OUTPUT_OFFSET>,
    env2: PolyEnvelope<ENV2_INPUT_OFFSET, ENV2_OUTPUT_OFFSET>,
    env3: PolyEnvelope<ENV3_INPUT_OFFSET, ENV3_OUTPUT_OFFSET>,
    filter1: PolyFilter<FILTER1_INPUT_OFFSET, FILTER1_OUTPUT_OFFSET>,
    filter2: PolyFilter<FILTER2_INPUT_OFFSET, FILTER2_OUTPUT_OFFSET>,
    effects_chain: EffectsChain,
    cables: Cables<MAX_CABLES>,
}

impl AudioState {
    pub fn new(receiver: mpsc::Receiver<AudioMessage>, sample_rate: f64) -> Self {
        let mut new_state = Self {
            receiver,
            sample_rate,
            inputs: [0.0; TOTAL_INPUT_COUNT],
            outputs: [0.0; TOTAL_OUTPUT_COUNT],
            midi: Midi::new(),
            osc1: PolyAnalog::new(),
            osc2: PolyWavetable::new(),
            env1: PolyEnvelope::new(),
            env2: PolyEnvelope::new(),
            env3: PolyEnvelope::new(),
            filter1: PolyFilter::new(),
            filter2: PolyFilter::new(),
            effects_chain: EffectsChain::new(),
            cables: Cables::new(),
        };

        new_state.init();
        new_state
    }

    pub fn init(&mut self) {
        self.cables.add_cable(MIDI_OUTPUT_OFFSET + midi::NOTE_OUTPUT, OSC1_INPUT_OFFSET + analog::FREQUENCY_INPUT).unwrap();
        self.cables.add_cable(MIDI_OUTPUT_OFFSET + midi::NOTE_OUTPUT, OSC2_INPUT_OFFSET + analog::FREQUENCY_INPUT).unwrap();
        self.cables.add_cable(MIDI_OUTPUT_OFFSET + midi::GATE_OUTPUT, ENV1_INPUT_OFFSET + envelope::GATE_INPUT).unwrap();
        self.cables.add_cable(MIDI_OUTPUT_OFFSET + midi::VELOCITY_OUTPUT, ENV1_INPUT_OFFSET + envelope::VELOCITY_INPUT).unwrap();
        self.cables.add_cable(ENV1_OUTPUT_OFFSET + envelope::OUT_VALUE, OSC1_INPUT_OFFSET + analog::LEVEL_INPUT).unwrap();
        self.cables.add_cable(ENV1_OUTPUT_OFFSET + envelope::OUT_VALUE, OSC2_INPUT_OFFSET + analog::LEVEL_INPUT).unwrap();
        self.cables.add_cable(ENV2_OUTPUT_OFFSET + envelope::OUT_VALUE, FILTER1_INPUT_OFFSET + analog::FREQUENCY_INPUT).unwrap();
        self.cables.add_cable(MIDI_OUTPUT_OFFSET + midi::GATE_OUTPUT, ENV2_INPUT_OFFSET + envelope::GATE_INPUT).unwrap();
        self.cables.add_cable(MIDI_OUTPUT_OFFSET + midi::VELOCITY_OUTPUT, ENV2_INPUT_OFFSET + envelope::VELOCITY_INPUT).unwrap();
        self.cables.add_cable(ENV3_OUTPUT_OFFSET + envelope::OUT_VALUE, FILTER2_INPUT_OFFSET + analog::FREQUENCY_INPUT).unwrap();
        self.cables.add_cable(MIDI_OUTPUT_OFFSET + midi::GATE_OUTPUT, ENV3_INPUT_OFFSET + envelope::GATE_INPUT).unwrap();
        self.cables.add_cable(MIDI_OUTPUT_OFFSET + midi::VELOCITY_OUTPUT, ENV3_INPUT_OFFSET + envelope::VELOCITY_INPUT).unwrap();
        //self.cables.add_cable(OSC1_OUTPUT_OFFSET + analog::OUT_VALUE, FILTER1_INPUT_OFFSET).unwrap();
        //self.cables.add_cable(OSC2_OUTPUT_OFFSET + analog::OUT_VALUE, FILTER1_INPUT_OFFSET).unwrap();
        self.cables.add_cable(OSC2_OUTPUT_OFFSET + analog::OUT_VALUE, EFFECTS_CHAIN_INPUT_OFFSET).unwrap();
        self.cables.add_cable(FILTER1_OUTPUT_OFFSET + analog::OUT_VALUE, FILTER2_INPUT_OFFSET).unwrap();
        self.cables.add_cable(FILTER2_OUTPUT_OFFSET + analog::OUT_VALUE, EFFECTS_CHAIN_INPUT_OFFSET).unwrap();
    }
}

impl AudioState {
    fn process(&mut self) -> (f32, f32) {
        self.midi.process(&mut self.outputs);
        self.osc1.render(&self.inputs, &mut self.outputs, self.sample_rate);
        self.osc2.render(&self.inputs, &mut self.outputs, self.sample_rate as f32);
        self.env1.render(&self.inputs, &mut self.outputs);
        self.env2.render(&self.inputs, &mut self.outputs);
        self.env3.render(&self.inputs, &mut self.outputs);
        self.filter1.render(&self.inputs, &mut self.outputs, self.sample_rate as f32);
        self.filter2.render(&self.inputs, &mut self.outputs, self.sample_rate as f32);
        self.cables.run_cables(&mut self.inputs, &self.outputs);
        self.effects_chain.render(unsafe { self.inputs[0..MAX_POLY_COUNT].try_into().unwrap_unchecked() })
    }

    fn update(&mut self) {
        for msg in self.receiver.try_iter() {
            match msg {
                AudioMessage::Osc1Freq(freq) => {
                    self.osc1.set_freq_value((freq - 0.5) / 10.0);
                },
                AudioMessage::Osc1Shape(shape) => {
                    self.osc1.set_shape(shape);
                },
                AudioMessage::DelayTime(time) => {
                    self.effects_chain.set_delay_time((time as f64 * self.sample_rate) as usize);
                },
                AudioMessage::KeyPress(velocity, note) => self.midi.key_press(&mut self.outputs, note, velocity),
                AudioMessage::KeyRelease(note) => self.midi.key_release(&mut self.outputs, note),
                AudioMessage::PedalPress => self.midi.pedal_press(),
                AudioMessage::PedalRelease => self.midi.pedal_release(&mut self.outputs),
                AudioMessage::Osc2Freq(freq) => {
                    self.osc2.set_freq_value((freq - 0.5) / 10.0);
                }
                AudioMessage::Osc2WavetableUpdate(new_wavetable) => {
                    self.osc2.update_wavetable(new_wavetable);
                }
            }
        }
    }
}


