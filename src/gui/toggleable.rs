use std::sync::mpsc;

use sdl3::{render::{Canvas, FRect, Texture}, video::Window};

use crate::{audio::AudioMessage, common::ComponentVec, gui::animation::Animation};

const MAX_TOGGLEABLE_COUNT: usize = 128;

#[derive(Clone, Copy)]
pub enum OnToggleBehavior {
    None,
}

pub struct Toggleables {
    rect: ComponentVec<FRect, MAX_TOGGLEABLE_COUNT>,
    on_left_click: ComponentVec<OnToggleBehavior, MAX_TOGGLEABLE_COUNT>,
    state: ComponentVec<usize, MAX_TOGGLEABLE_COUNT>,
    render: ComponentVec<Animation, MAX_TOGGLEABLE_COUNT>,
}

impl Toggleables {
    pub fn init() -> Self {
        Self {
            rect: ComponentVec::new(),
            on_left_click: ComponentVec::new(),
            state: ComponentVec::new(),
            render: ComponentVec::new(),
        }
    }

    pub fn spawn(
        &mut self,
        rect: FRect,
        on_left_click: OnToggleBehavior,
        state: usize,
        animation: Animation,
    ) -> Result<(), ()> {
        self.rect.push(rect)?;
        self.on_left_click.push(on_left_click)?;
        self.state.push(state)?;
        self.render.push(animation)?;
        Ok(())
    }
}

pub fn render_system(canvas: &mut Canvas<Window>, textures: &[Texture], toggleables: &Toggleables) -> Result<(), sdl3::Error> {
    for (dst, (state, animation)) in toggleables.rect.iter().zip(toggleables.state.iter().zip(toggleables.render.iter())) {
        let frame = *state;
        let (texture, src) = animation.get_frame(frame, textures);
        canvas.copy(texture, src, *dst)?;
    }
    Ok(())
}

pub fn on_left_down_system(audio_channel: &mut mpsc::Sender<AudioMessage>, toggleables: &mut Toggleables, x: f32, y: f32, clicks: u8) {
    for (i, rect) in toggleables.rect.iter().enumerate() {
        if x >= rect.x && x <= rect.x + rect.w && y >= rect.y && y <= rect.y + rect.h {
            let on_click = toggleables.on_left_click[i];
            let state = &mut toggleables.state[i];
            *state = (*state + 1) % toggleables.render[i].get_frame_count();
            match on_click {
                OnToggleBehavior::None => {},
            }
            break;
        }
    }
}