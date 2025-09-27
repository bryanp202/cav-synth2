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
use sdl3::pixels::{Color, PixelFormat};
use sdl3::sys::pixels::SDL_PIXELFORMAT_ABGR8888;
use sdl3::video::WindowContext;
use sdl3::{video::Window};
use sdl3::render::{Canvas, FPoint, FRect, Texture, TextureCreator};

use crate::audio::AudioMessage;
use crate::common::ComponentVec;
use crate::gui::animation::Animation;
use crate::gui::drawable::Drawables;
use crate::gui::jacks::JackData;
use crate::gui::toggleable::Toggleables;
use crate::gui::dragable::{DragType, Dragables, OnDragBehavior};

const JACK_INPUT_TEXTURE: usize = 4;
const JACK_OUTPUT_TEXTURE: usize = 5;
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
}

impl <'a> Gui <'a> {
    pub fn new(audio_channel: Sender<AudioMessage>, texture_creator: &'a TextureCreator<WindowContext>) -> Self {
        Self {
            audio_channel,
            mouse_pos: FPoint { x: 0.0, y: 0.0 },
            //text_msg: None,
            toggleables: Toggleables::init(),
            dragables: Dragables::init(),
            jacks: JackData::new(),
            drawables: Drawables::new(),
            textures: ComponentVec::new(),
            texture_creator,
        }
    }

    pub fn init(&mut self) {
        self.load_texture(include_bytes!("../assets/knob_basic128.png"));
        self.load_texture(include_bytes!("../assets/switch_two_state.png"));
        self.load_texture(include_bytes!("../assets/switch_three_state.png"));
        self.load_texture(include_bytes!("../assets/slider_detailed.png"));
        self.load_texture(include_bytes!("../assets/jack_input.png"));
        self.load_texture(include_bytes!("../assets/jack_output.png"));
        self.load_texture(include_bytes!("../assets/knob_basic4.png"));


        for x in 0..10 {
            let x = x as f32;
            for y in 0..5 {
                let y = y as f32;
                self.dragables.spawn(
                    FRect::new(x * 64.0, y * 64.0, 64.0, 64.0), 
                    0.5, 
                    (DragType::HORIZONTAL, OnDragBehavior::Osc1Freq),
                    dragable::OnDoubleClickBehavior::SetTo(0.5),
                    Animation::new(0, 128, 64.0, 64.0)
                ).unwrap();
                let switch_w = self.textures[1].width() as f32;
                let switch_h = switch_w;
                self.toggleables.spawn(
                    FRect::new(640.0 + x * switch_w, y * switch_h, switch_w, switch_h), 
                    toggleable::OnToggleBehavior::None, 
                    0,
                    Animation::new(1, 2, switch_w, switch_h)
                ).unwrap();
            }
            for y in 5..10 {
                let y = y as f32;
                self.dragables.spawn(
                    FRect::new(x * 64.0, y * 64.0, 64.0, 64.0), 
                    0.5, 
                    (DragType::VERTICAL, OnDragBehavior::Osc1Freq),
                    dragable::OnDoubleClickBehavior::SetTo(0.9),
                    Animation::new(0, 128, 64.0, 64.0)
                ).unwrap();
                let switch_w = self.textures[2].width() as f32;
                let switch_h = switch_w;
                self.toggleables.spawn(
                    FRect::new(640.0 + x * switch_w, y * switch_h, switch_w, switch_h), 
                    toggleable::OnToggleBehavior::None, 
                    0,
                    Animation::new(2, 3, switch_w, switch_h)
                ).unwrap();
                let slider_w = self.textures[3].width() as f32;
                let slider_h = self.textures[3].height() as f32 / 128.0;
                self.dragables.spawn(
                    FRect::new(1000.0 + x * slider_w, (y-5.0) * slider_h, slider_w, slider_h), 
                    0.5,
                    (DragType::VERTICAL, OnDragBehavior::DelayTime),
                    dragable::OnDoubleClickBehavior::SetTo(0.5),
                    Animation::new(3, 128, slider_w, slider_h)
                ).unwrap();
            }
        }

        for x in 0..5 {
            let x = x as f32 * 32.0;
            for y in 0..3 {
                let y = y as f32 * 32.0;
                self.jacks.spawn_input(
                    FRect::new(800.0 + x, 800.0 + x + y, 32.0, 32.0),
                    jacks::InputJack::Osc1Freq,
                ).unwrap();

                self.jacks.spawn_output(
                    FRect::new(200.0 + x, 800.0 + x + y, 32.0, 32.0),
                    jacks::OutputJack::Osc1Value,
                ).unwrap();
            }
        }

        self.dragables.spawn(
            FRect { x: 1200.0, y: 700.0, w: 64.0, h: 64.0 },
            0.0,
            (DragType::VERTICAL, OnDragBehavior::Osc1Shape),
            dragable::OnDoubleClickBehavior::SetTo(0.0),
            Animation::new(self.textures.len() - 1, 4, 64.0, 64.0)
        ).unwrap();
        self.drawables.spawn(FRect { x: 1300.0, y: 500.0, w: 256.0, h: 200.0 }, drawable::OnReleaseBehavior::Osc2WavetableTimeDomain).unwrap();
        self.drawables.spawn(FRect { x: 1300.0, y: 100.0, w: 256.0, h: 200.0 }, drawable::OnReleaseBehavior::Osc2WavetableTimeDomain).unwrap();
    }

    pub fn render(&mut self, canvas: &mut Canvas<Window>) -> Result<(), sdl3::Error> {
        canvas.set_draw_color(Color::RGBA(0, 0, 0, 0));
        canvas.clear();
        toggleable::render_system(canvas, &self.textures, &self.toggleables)?;
        dragable::render_system(canvas, &self.textures, &self.dragables)?;
        drawable::render_system(canvas, &self.drawables)?;
        canvas.set_blend_mode(sdl3::render::BlendMode::Blend);
        jacks::render_system(
            canvas,
            &self.textures[JACK_OUTPUT_TEXTURE],
            &self.textures[JACK_INPUT_TEXTURE],
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
        jacks::on_left_release_system(&mut self.jacks, self.mouse_pos);
    }

    pub fn right_mouse_down(&mut self, x: f32, y: f32, clicks: u8) {
        jacks::on_right_down_system(&mut self.jacks, x, y);
    }

    pub fn mouse_move(&mut self, x: f32, y: f32, xrel: f32, yrel: f32) {
        self.mouse_pos = FPoint::new(x, y);
        dragable::on_mouse_move_system(&mut self.audio_channel, &mut self.dragables, xrel, yrel);
        drawable::on_mouse_move_system(&mut self.drawables, x, y);
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