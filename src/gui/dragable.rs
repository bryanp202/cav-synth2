use std::sync::mpsc;

use sdl3::{render::{Canvas, FRect, Texture}, video::Window};

use crate::{audio::AudioMessage, common::{point_in_frect, ComponentVec}, gui::animation::Animation};

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
        match on_drag {
            OnDragBehavior::Osc1Freq => audio_channel.send(AudioMessage::Osc1Freq(send_value)).unwrap(),
        }
    }
}