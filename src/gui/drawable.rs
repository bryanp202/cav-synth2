use std::{cmp::Ordering, sync::{mpsc, Arc}};

use realfft::{num_complex::Complex, num_traits::Zero, ComplexToReal, RealFftPlanner, RealToComplex};
use sdl3::{pixels::FColor, render::{Canvas, FRect}, video::Window};

use crate::{audio::{AudioMessage, Wavetable, WAVETABLE_FRAME_LENGTH}, common::{point_in_frect, ComponentVec}};

const MAX_DRAWABLE_COUNT: usize = 1;

#[derive(Clone, Copy)]
pub enum OnReleaseBehavior {
    Osc2WavetableTimeDomain,
}

pub struct Drawables {
    active_drawable: Option<(usize, f32, FRect, usize, f32)>, // Index, center y, rect, last_x, last_y
    rect: ComponentVec<FRect, MAX_DRAWABLE_COUNT>,
    on_release: ComponentVec<OnReleaseBehavior, MAX_DRAWABLE_COUNT>,
    values: ComponentVec<Vec<FRect>, MAX_DRAWABLE_COUNT>,
    r2cfft: Arc<dyn RealToComplex<f32>>,
    c2rfft: Arc<dyn ComplexToReal<f32>>,
}

impl Drawables {
    pub fn new(fft_planner: &mut RealFftPlanner<f32>) -> Self {
        Self {
            active_drawable: None,
            rect: ComponentVec::new(),
            on_release: ComponentVec::new(),
            values: ComponentVec::new(),
            r2cfft: fft_planner.plan_fft_forward(WAVETABLE_FRAME_LENGTH),
            c2rfft: fft_planner.plan_fft_inverse(WAVETABLE_FRAME_LENGTH),
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
        on_release_behavior(&drawables.r2cfft, &drawables.c2rfft, audio_channel, on_release, values, rect.h);
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
    canvas.set_draw_color(FColor::RGBA(0.0, 1.0, 1.0, 0.6));
    for values in drawables.values.iter() {
        canvas.draw_rects(&values)?;
    }
    Ok(())
}


fn on_release_behavior(
    r2c: &Arc<dyn RealToComplex<f32>>,
    c2r: &Arc<dyn ComplexToReal<f32>>,
    audio_channel: &mut mpsc::Sender<AudioMessage>,
    on_release: OnReleaseBehavior,
    values: &Vec<FRect>,
    height: f32
) {
    match on_release {
        OnReleaseBehavior::Osc2WavetableTimeDomain => {
            const PARTIAL_COUNT: usize = WAVETABLE_FRAME_LENGTH / 2 + 1;

            let mut default_variation: [f32; WAVETABLE_FRAME_LENGTH] = std::array::from_fn(|i| {
                let index1 = i / 8;
                let index2 = ((i + 1) / 8) % 256;
                let ratio = (i % 8) as f32 / 8.0;
                2.0 * (values[index1].h + (values[index2].h - values[index1].h) * ratio) / height
            });

            let mut new_wavetable: Wavetable = [0.0; WAVETABLE_FRAME_LENGTH * 8];

            let mut freq_domain = [Complex::zero(); PARTIAL_COUNT];
            r2c.process(&mut default_variation, &mut freq_domain).unwrap();

            freq_domain[0] = Complex::zero();
            c2r.process(&mut freq_domain.clone(), &mut new_wavetable[0..WAVETABLE_FRAME_LENGTH]).unwrap();
            freq_domain[PARTIAL_COUNT / 2..].fill(Complex::zero());
            c2r.process(&mut freq_domain.clone(), &mut new_wavetable[WAVETABLE_FRAME_LENGTH..WAVETABLE_FRAME_LENGTH*2]).unwrap();
            freq_domain[PARTIAL_COUNT / 4 .. PARTIAL_COUNT / 2].fill(Complex::zero());
            c2r.process(&mut freq_domain.clone(), &mut new_wavetable[WAVETABLE_FRAME_LENGTH*2..WAVETABLE_FRAME_LENGTH*3]).unwrap();
            freq_domain[PARTIAL_COUNT / 8 .. PARTIAL_COUNT / 4].fill(Complex::zero());
            c2r.process(&mut freq_domain.clone(), &mut new_wavetable[WAVETABLE_FRAME_LENGTH*3..WAVETABLE_FRAME_LENGTH*4]).unwrap();
            freq_domain[PARTIAL_COUNT / 16 .. PARTIAL_COUNT / 8].fill(Complex::zero());
            c2r.process(&mut freq_domain.clone(), &mut new_wavetable[WAVETABLE_FRAME_LENGTH*4..WAVETABLE_FRAME_LENGTH*5]).unwrap();
            freq_domain[PARTIAL_COUNT / 32 .. PARTIAL_COUNT / 16].fill(Complex::zero());
            c2r.process(&mut freq_domain.clone(), &mut new_wavetable[WAVETABLE_FRAME_LENGTH*5..WAVETABLE_FRAME_LENGTH*6]).unwrap();
            freq_domain[PARTIAL_COUNT / 64 .. PARTIAL_COUNT / 32].fill(Complex::zero());
            c2r.process(&mut freq_domain.clone(), &mut new_wavetable[WAVETABLE_FRAME_LENGTH*6..WAVETABLE_FRAME_LENGTH*7]).unwrap();
            freq_domain[PARTIAL_COUNT / 128 .. PARTIAL_COUNT / 64].fill(Complex::zero());
            c2r.process(&mut freq_domain.clone(), &mut new_wavetable[WAVETABLE_FRAME_LENGTH*7..]).unwrap();

            let max = new_wavetable.into_iter().reduce(f32::max).unwrap_or(0.0).abs();
            new_wavetable.iter_mut().for_each(|x| *x /= max);
            
            audio_channel.send(AudioMessage::Osc2WavetableUpdate(Arc::new(new_wavetable))).unwrap();
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