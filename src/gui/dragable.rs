use std::sync::mpsc;

use sdl3::{render::{Canvas, FRect, Texture}, video::Window};

use crate::{audio::{self, AudioMessage, WaveShape}, common::{point_in_frect, ComponentVec}, gui::animation::Animation};

const MAX_DRAGABLE_COUNT: usize = 228;

type DraggingInfo = Option<(usize, (DragType, OnDragBehavior))>;

#[derive(Clone, Copy)]
pub enum DragType {
    VERTICAL,
    HORIZONTAL,
}

#[derive(Clone, Copy)]
pub enum OnDragBehavior {
    Osc1Freq,
    Osc1Shape,
    Osc1Level,
    Osc1Phase,
    Osc2Level,
    Osc2Freq,
    Osc2Phase,
    Lfo1Shape,
    Lfo1Freq,
    Lfo2Shape,
    Lfo2Freq,
    Filter1Freq,
    Filter2Freq,
    Env1Attack,
    Env1Decay,
    Env1Release,
    Env1Sustain,
    Env2Attack,
    Env2Decay,
    Env2Release,
    Env2Sustain,
    Env3Attack,
    Env3Decay,
    Env3Release,
    Env3Sustain,
    EffectDistDrive,
    EffectDistWet,
    EffectDelayFeedback,
    EffectDelayTime,
    EffectDelayWet,
    EffectReverbTime,
    EffectReverbWet,
    MasterGain,
}

#[derive(Clone, Copy)]
pub enum OnDoubleClickBehavior {
    SetTo(f32),
}

pub struct Dragables {
    dragging_info: DraggingInfo,
    rect: ComponentVec<FRect, MAX_DRAGABLE_COUNT>,
    value: ComponentVec<f32, MAX_DRAGABLE_COUNT>,
    on_drag: ComponentVec<(DragType, OnDragBehavior), MAX_DRAGABLE_COUNT>,
    on_double_click: ComponentVec<OnDoubleClickBehavior, MAX_DRAGABLE_COUNT>,
    render: ComponentVec<Animation, MAX_DRAGABLE_COUNT>,
}

impl Dragables {
    pub fn init() -> Self {
        Self {
            dragging_info: None,
            rect: ComponentVec::new(),
            value: ComponentVec::new(),
            on_drag: ComponentVec::new(),
            on_double_click: ComponentVec::new(),
            render: ComponentVec::new(),
        }
    }

    pub fn spawn(
        &mut self,
        rect: FRect,
        inital_value: f32,
        on_drag: (DragType, OnDragBehavior),
        on_click: OnDoubleClickBehavior,
        animation: Animation,
    ) -> Result<(), ()> {
        self.rect.push(rect)?;
        self.on_drag.push(on_drag)?;
        self.on_double_click.push(on_click)?;
        self.value.push(inital_value)?;
        self.render.push(animation)?;
        Ok(())
    }
}

pub fn on_left_down_system(audio_channel: &mut mpsc::Sender<AudioMessage>, dragables: &mut Dragables, x: f32, y: f32, clicks: u8) {
    for (i, rect) in dragables.rect.iter().enumerate() {
        if point_in_frect(rect, x, y) {
            if clicks == 2 {
                match dragables.on_double_click[i] {
                    OnDoubleClickBehavior::SetTo(value) => {
                        let (_, on_drag) = dragables.on_drag[i];
                        let animation_frames = dragables.render[i].get_frame_count();
                        on_drag_behavior(audio_channel, &mut dragables.value[i], on_drag, value, animation_frames);
                    },
                }
            }
            dragables.dragging_info = Some((i, dragables.on_drag[i]));
            break;
        }
    }
}

pub fn on_mouse_move_system(audio_channel: &mut mpsc::Sender<AudioMessage>, dragables: &mut Dragables, xrel: f32, yrel: f32) {
    if let Some((dragable_index, (drag_type, on_drag))) = dragables.dragging_info {
        let drag_amt = match drag_type {
            DragType::HORIZONTAL =>  xrel/200.0,
            DragType::VERTICAL => -yrel/200.0,
        };
        let new_value = (dragables.value[dragable_index] + drag_amt).clamp(0.0, 1.0);
        let animation_frames = dragables.render[dragable_index].get_frame_count();
        on_drag_behavior(audio_channel, &mut dragables.value[dragable_index], on_drag, new_value, animation_frames);
    }
}

pub fn on_left_release_system(dragables: &mut Dragables) {
    dragables.dragging_info = None;
}

pub fn render_system(canvas: &mut Canvas<Window>, textures: &[Texture], dragables: &Dragables) -> Result<(), sdl3::Error> {
    for (dst, (value, animation)) in dragables.rect.iter().zip(dragables.value.iter().zip(dragables.render.iter())) {
        let animation_frame = ((animation.get_frame_count() - 1) as f32 * value) as usize;
        let (texture, src) = animation.get_frame(animation_frame, textures);
        canvas.copy(texture, src, *dst)?;
    }
    Ok(())
}

fn on_drag_behavior(audio_channel: &mut mpsc::Sender<AudioMessage>, value: &mut f32, on_drag: OnDragBehavior, new_value: f32, animation_frames: usize) {
    let old_frame = ((animation_frames - 1) as f32 * *value) as usize;
    let new_frame = (animation_frames - 1) as f32 * new_value;
    *value = new_value;
    if old_frame != new_frame as usize {
        let send_value = new_frame / animation_frames as f32;
        let result = match on_drag {
            // Osc1
            OnDragBehavior::Osc1Freq => audio_channel.send(AudioMessage::Osc1Freq(send_value)),
            OnDragBehavior::Osc1Shape => {
                let shape = match (send_value * 4.0) as usize {
                    0 => WaveShape::Sine,
                    1 => WaveShape::Triangle,
                    2 => WaveShape::Square,
                    _ => WaveShape::Saw,
                };
                audio_channel.send(AudioMessage::Osc1Shape(shape))
            },
            OnDragBehavior::Osc1Level => audio_channel.send(AudioMessage::Osc1Level(send_value)),
            OnDragBehavior::Osc1Phase => audio_channel.send(AudioMessage::Osc1Phase(send_value)),
            // Osc2
            OnDragBehavior::Osc2Freq => audio_channel.send(AudioMessage::Osc2Freq(send_value)),
            OnDragBehavior::Osc2Level => audio_channel.send(AudioMessage::Osc2Level(send_value)),
            OnDragBehavior::Osc2Phase => audio_channel.send(AudioMessage::Osc2Phase(send_value)),

            // Lfo1
            OnDragBehavior::Lfo1Freq => audio_channel.send(AudioMessage::Lfo1Freq(send_value)),
            OnDragBehavior::Lfo1Shape => {
                let shape = match (send_value * 4.0) as usize {
                    0 => WaveShape::Sine,
                    1 => WaveShape::Triangle,
                    2 => WaveShape::Square,
                    _ => WaveShape::Saw,
                };
                audio_channel.send(AudioMessage::Lfo1Shape(shape))
            },
            // Lfo2
            OnDragBehavior::Lfo2Freq => audio_channel.send(AudioMessage::Lfo2Freq(send_value)),
            OnDragBehavior::Lfo2Shape => {
                let shape = match (send_value * 4.0) as usize {
                    0 => WaveShape::Sine,
                    1 => WaveShape::Triangle,
                    2 => WaveShape::Square,
                    _ => WaveShape::Saw,
                };
                audio_channel.send(AudioMessage::Lfo2Shape(shape))
            },

            // Filter1
            OnDragBehavior::Filter1Freq => audio_channel.send(AudioMessage::Filter1Freq(send_value)),
            // Filter2
            OnDragBehavior::Filter2Freq => audio_channel.send(AudioMessage::Filter2Freq(send_value)),

            // Env1
            OnDragBehavior::Env1Attack => audio_channel.send(AudioMessage::Env1Attack(send_value)),
            OnDragBehavior::Env1Decay => audio_channel.send(AudioMessage::Env1Decay(send_value)),
            OnDragBehavior::Env1Sustain => audio_channel.send(AudioMessage::Env1Sustain(send_value)),
            OnDragBehavior::Env1Release => audio_channel.send(AudioMessage::Env1Release(send_value)),
            // Env2
            OnDragBehavior::Env2Attack => audio_channel.send(AudioMessage::Env2Attack(send_value)),
            OnDragBehavior::Env2Decay => audio_channel.send(AudioMessage::Env2Decay(send_value)),
            OnDragBehavior::Env2Sustain => audio_channel.send(AudioMessage::Env2Sustain(send_value)),
            OnDragBehavior::Env2Release => audio_channel.send(AudioMessage::Env2Release(send_value)),
            // Env3
            OnDragBehavior::Env3Attack => audio_channel.send(AudioMessage::Env3Attack(send_value)),
            OnDragBehavior::Env3Decay => audio_channel.send(AudioMessage::Env3Decay(send_value)),
            OnDragBehavior::Env3Sustain => audio_channel.send(AudioMessage::Env3Sustain(send_value)),
            OnDragBehavior::Env3Release => audio_channel.send(AudioMessage::Env3Release(send_value)),

            // Effects
            // Distortion
            OnDragBehavior::EffectDistDrive => audio_channel.send(AudioMessage::DistDrive(send_value)),
            OnDragBehavior::EffectDistWet => audio_channel.send(AudioMessage::DistWet(send_value)),
            // Delay
            OnDragBehavior::EffectDelayFeedback => audio_channel.send(AudioMessage::DelayFeedback(send_value)),
            OnDragBehavior::EffectDelayTime => audio_channel.send(AudioMessage::DelayTime(send_value)),
            OnDragBehavior::EffectDelayWet => audio_channel.send(AudioMessage::DelayWet(send_value)),
            // Reverb
            OnDragBehavior::EffectReverbTime => audio_channel.send(AudioMessage::ReverbTime(send_value)),
            OnDragBehavior::EffectReverbWet => audio_channel.send(AudioMessage::ReverbWet(send_value)),

            // Master
            OnDragBehavior::MasterGain => audio_channel.send(AudioMessage::MasterGain(send_value)),
        };

        result.unwrap();
    }
}