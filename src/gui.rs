// pub struct GuiState {
//     clickables: _, // Send message and set mouse_move_message and text_input_message to the message provided by the (both to None if no collision)
//     selected_draggable: Option<fn(f32, f32, f32, f32, u8) -> GuiMessage>, // Resets to None on MouseLeftRelease
//     selected_text: Option<fn(String) -> GuiMessage>, // Only resets through clickables/collision testing
//     // EVERY MOUSE MOVE, all hoverables are checked for mouse collision
//          // WORKS AS FOLLOWS: all hoverables are set to not hovering, then collision checking is done and collisions are set to hovered
//          // This is because hovering causes visual changes, not behavior changes, so a message for each hoverable object seems like a waste
//          // There is also no corrisponding message for "releasing a hover" like a click, so hovering has to behave differently
//          // If hovering causing external state changes was desireable, then messages could be added, but some form of message would have to be
//          // Sent on hover release
// }
mod animation;
mod cable;
mod dragable;
mod drawable;
mod jacks;
mod toggleable;

use core::f32;
use std::sync::mpsc::Sender;
use realfft::RealFftPlanner;
use sdl3::pixels::{Color, PixelFormat};
use sdl3::sys::pixels::SDL_PIXELFORMAT_ABGR8888;
use sdl3::video::WindowContext;
use sdl3::{video::Window};
use sdl3::render::{Canvas, FPoint, FRect, Texture, TextureCreator};

use crate::audio::{AudioMessage, InputJack, OutputJack};
use crate::common::ComponentVec;
use crate::gui::animation::Animation;
use crate::gui::drawable::{Drawables, OnReleaseBehavior};
use crate::gui::jacks::JackData;
use crate::gui::toggleable::Toggleables;
use crate::gui::dragable::{DragType, Dragables, OnDragBehavior};

const FACEPLATE_TEXTURE: usize = 0;
const JACK_INPUT_TEXTURE: usize = 1;
const JACK_OUTPUT_TEXTURE: usize = 2;
const KNOB_128_TEXTURE: usize = 3;
const KNOB_4_TEXTURE: usize = 4;
const METER_MASTER_TEXTURE: usize = 5;
const CABLE_SLIDER_TEXTURE: usize = 6;
const TEXTURE_COUNT: usize = 7;

pub struct Gui<'a> {
    audio_channel: Sender<AudioMessage>,

    mouse_pos: FPoint,
    //text_msg: Option<fn (String) -> GuiMessage>,

    toggleables: Toggleables,
    dragables: Dragables,
    jacks: JackData,
    drawables: Drawables,
    // text_boxes: TextBoxes,

    // Textures
    textures: ComponentVec<Texture<'a>, TEXTURE_COUNT>,
    texture_creator: &'a TextureCreator<WindowContext>,
    // FFT
    fft_planner: RealFftPlanner<f32>,
}

impl <'a> Gui <'a> {
    pub fn new(audio_channel: Sender<AudioMessage>, texture_creator: &'a TextureCreator<WindowContext>) -> Self {
        let mut fft_planner = RealFftPlanner::new();
        Self {
            audio_channel,
            mouse_pos: FPoint { x: 0.0, y: 0.0 },
            //text_msg: None,
            toggleables: Toggleables::init(),
            dragables: Dragables::init(),
            jacks: JackData::new(),
            drawables: Drawables::new(&mut fft_planner),
            textures: ComponentVec::new(),
            texture_creator,
            fft_planner
        }
    }

    pub fn init(&mut self) {
        self.load_texture(include_bytes!("../assets/faceplate.png"));
        self.load_texture(include_bytes!("../assets/jack_input.png"));
        self.load_texture(include_bytes!("../assets/jack_output.png"));
        self.load_texture(include_bytes!("../assets/knob_basic128.png"));
        self.load_texture(include_bytes!("../assets/knob_basic4.png"));
        self.load_texture(include_bytes!("../assets/slider_128_35x90.png"));
        self.load_texture(include_bytes!("../assets/meter_master31_35x120.png"));

        // OSC 1
        self.dragables.spawn(
            FRect { x: 1200.0, y: 700.0, w: 64.0, h: 64.0 },
            0.0,
            (DragType::VERTICAL, OnDragBehavior::Osc1Shape),
            dragable::OnDoubleClickBehavior::SetTo(0.0),
            Animation::new(self.textures.len() - 2, 4, 64.0, 64.0)
        ).unwrap();

        // OSC 2
        self.drawables.spawn(FRect::new(502.0, 16.0, 256.0, 256.0), OnReleaseBehavior::Osc2WavetableTimeDomain).unwrap();
    }

    pub fn render(&mut self, canvas: &mut Canvas<Window>) -> Result<(), sdl3::Error> {
        canvas.copy(&self.textures[FACEPLATE_TEXTURE], None, None)?;
        toggleable::render_system(canvas, &self.textures, &self.toggleables)?;
        dragable::render_system(canvas, &self.textures, &self.dragables)?;
        drawable::render_system(canvas, &self.drawables)?;
        canvas.set_blend_mode(sdl3::render::BlendMode::Blend);
        jacks::render_system(
            canvas,
            &self.textures,
            &mut self.jacks,
            self.mouse_pos
        )?;
        canvas.set_blend_mode(sdl3::render::BlendMode::None);
        canvas.present();
        Ok(())
    }

    pub fn left_mouse_down(&mut self, x: f32, y: f32, clicks: u8) {
        dragable::on_left_down_system(&mut self.audio_channel, &mut self.dragables, x, y, clicks);
        toggleable::on_left_down_system(&mut self.audio_channel, &mut self.toggleables, x, y, clicks);
        drawable::on_left_down_system(&mut self.drawables, x, y);
        jacks::on_left_down_system(&mut self.jacks, x, y);
    }

    pub fn left_mouse_up(&mut self, clicks: u8)  {
        dragable::on_left_release_system(&mut self.dragables);
        drawable::on_left_release_system(&mut self.audio_channel, &mut self.drawables);
        jacks::on_left_release_system(&mut self.audio_channel, &mut self.jacks, self.mouse_pos);
    }

    pub fn right_mouse_down(&mut self, x: f32, y: f32, clicks: u8) {
        jacks::on_right_down_system(&mut self.audio_channel, &mut self.jacks, x, y, clicks);
    }

    pub fn right_mouse_up(&mut self, clicks: u8) {
        jacks::on_right_release_system(&mut self.jacks);
    }

    pub fn mouse_move(&mut self, x: f32, y: f32, xrel: f32, yrel: f32) {
        self.mouse_pos = FPoint::new(x, y);
        dragable::on_mouse_move_system(&mut self.audio_channel, &mut self.dragables, xrel, yrel);
        drawable::on_mouse_move_system(&mut self.drawables, x, y);
        jacks::on_mouse_move_system(&mut self.audio_channel, &mut self.jacks, xrel, yrel);
    }

    pub fn text_input(&mut self, text: String) {

    }

    fn load_texture(&mut self, img_bytes: &'static [u8]) {
        let img = image::ImageReader::new(std::io::Cursor::new(img_bytes))
            .with_guessed_format().unwrap().decode().unwrap();
        let mut texture = self.texture_creator.create_texture_streaming(
            unsafe {PixelFormat::from_ll(SDL_PIXELFORMAT_ABGR8888)},
            img.width(),
            img.height()
        ).unwrap();
        let rgba = img.to_rgba8();
        texture.update(None, &rgba, 4 * img.width() as usize).unwrap();
        self.textures.push(texture).unwrap();
    }
}