use std::{cmp::Ordering, sync::{mpsc, Arc}};

use sdl3::{pixels::Color, render::{Canvas, FPoint, FRect}, video::Window};

use crate::{audio::AudioMessage, common::{point_in_frect, ComponentVec}};

const MAX_DRAWABLE_COUNT: usize = 2;

#[derive(Clone, Copy)]
pub enum OnReleaseBehavior {
    Osc2WavetableTimeDomain,
}

pub struct Drawables {
    active_drawable: Option<(usize, f32, FRect, usize, f32)>, // Index, center y, rect, last_x, last_y
    rect: ComponentVec<FRect, MAX_DRAWABLE_COUNT>,
    on_release: ComponentVec<OnReleaseBehavior, MAX_DRAWABLE_COUNT>,
    values: ComponentVec<Vec<FRect>, MAX_DRAWABLE_COUNT>,
}

impl Drawables {
    pub fn new() -> Self {
        Self {
            active_drawable: None,
            rect: ComponentVec::new(),
            on_release: ComponentVec::new(),
            values: ComponentVec::new(),
        }
    }

    pub fn spawn(&mut self, rect: FRect, on_release: OnReleaseBehavior) -> Result<(), ()> {
        self.rect.push(rect)?;
        self.on_release.push(on_release)?;
        let center_line = rect.y + rect.h / 2.0;
        let iter = (0..rect.w as usize).map(|x| {
            FRect::new(x as f32 + rect.x, center_line, 1.0, 0.0)
        });
        self.values.push(Vec::from_iter(iter))?;
        Ok(())
    }
}

pub fn on_left_down_system(drawables: &mut Drawables, x: f32, y: f32) {
    for (i, rect) in drawables.rect.iter().enumerate() {
        if point_in_frect(rect, x, y) {
            let center_line = rect.y + rect.h / 2.0;
            let (last_x, last_y) = update_value_point(&mut drawables.values[i], *rect, center_line, x, y);
            drawables.active_drawable = Some((i, center_line, *rect, last_x, last_y));
            break;
        }
    }
}

pub fn on_left_release_system(audio_channel: &mut mpsc::Sender<AudioMessage>, drawables: &mut Drawables) {
    if let Some((i, _, rect, _, _)) = drawables.active_drawable {
        let on_release = drawables.on_release[i];
        let values = &drawables.values[i];
        on_release_behavior(audio_channel, on_release, values, rect.h);
        drawables.active_drawable = None;
    }
}

pub fn on_mouse_move_system(drawables: &mut Drawables, x: f32, y: f32) {
    if let Some((i, center_line, rect, last_x, last_y)) = drawables.active_drawable {
        let (last_x, last_y) = update_value_interp(&mut drawables.values[i], rect, last_x, last_y, center_line, x, y);
        drawables.active_drawable = Some((i, center_line, rect, last_x, last_y));
    }
}
pub fn render_system(canvas: &mut Canvas<Window>, drawables: &Drawables) -> Result<(), sdl3::Error> {
    canvas.set_draw_color(Color::CYAN);
    for values in drawables.values.iter() {
        canvas.draw_rects(&values)?;
    }
    Ok(())
}


fn on_release_behavior(audio_channel: &mut mpsc::Sender<AudioMessage>, on_release: OnReleaseBehavior, values: &Vec<FRect>, height: f32) {
    match on_release {
        OnReleaseBehavior::Osc2WavetableTimeDomain => {
            let new_wavetable: Arc<[f32; 2048]> = Arc::new(std::array::from_fn(|i| {
                let index1 = i / 8;
                let index2 = ((i + 1) / 8) % 256;
                let ratio = (i % 8) as f32 / 8.0;
                2.0 * (values[index1].h + (values[index2].h - values[index1].h) * ratio) / height
            }));
            audio_channel.send(AudioMessage::WavetableUpdate(new_wavetable)).unwrap();
        },
    }
}

fn update_value_interp(values: &mut Vec<FRect>, rect: FRect, last_x: usize, last_height: f32, center_line: f32, x: f32, y: f32) -> (usize, f32) {
    let (changed_index, new_height) = update_value_point(values, rect, center_line, x, y);
    
    let (b, slope, min, max) = match last_x.cmp(&changed_index) {
        Ordering::Less => {
            let slope = (new_height - last_height) / (changed_index - last_x) as f32;
            (last_height, slope, last_x, changed_index)
        },
        _ => {
            let slope = (last_height - new_height) / (last_x - changed_index) as f32;
            (new_height, slope, changed_index, last_x)
        },
    };

    for i in 1..max-min {
        let y = slope * i as f32 + b;
        let iterpolated_height = y.clamp(-rect.h / 2.0, rect.h / 2.0);
        values[i + min].set_h(iterpolated_height);
    }

    (changed_index, new_height)
}

fn update_value_point(values: &mut Vec<FRect>, rect: FRect, center_line: f32, x: f32, y: f32) -> (usize, f32) {
    let changed_index = ((x - rect.x) as usize).clamp(0, rect.w.abs() as usize - 1);
    let new_height = (y - center_line).clamp(-rect.h / 2.0, rect.h / 2.0);
    values[changed_index].set_h(new_height);
    (changed_index, new_height)
}