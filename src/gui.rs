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
use sdl3::pixels::PixelFormat;
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
const SLIDER_CABLE_TEXTURE: usize = 5;
const SLIDER_128_TEXTURE: usize = 6;
const METER_MASTER_TEXTURE: usize = 7;
const TEXTURE_COUNT: usize = 8;

const JACK_WIDTH: f32 = 32.0;
const JACK_HEIGHT: f32 = 32.0;

const KNOB_128_ANIMATION: Animation = Animation::new_comptime(KNOB_128_TEXTURE, 128, 64.0, 64.0);
const KNOB_4_ANIMATION: Animation = Animation::new_comptime(KNOB_4_TEXTURE, 4, 64.0, 64.0);
const SLIDER_CABLE_ANIMATION: Animation = Animation::new_comptime(SLIDER_CABLE_TEXTURE, 201, 64.0, 32.0);
const METER_MASTER_ANIMATION: Animation = Animation::new_comptime(METER_MASTER_TEXTURE, 31, 35.0, 120.0);
const SLIDER_128_ANIMATION: Animation = Animation::new_comptime(SLIDER_128_TEXTURE, 128, 35.0, 90.0);

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
    _fft_planner: RealFftPlanner<f32>,
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
            _fft_planner: fft_planner,
        }
    }

    pub fn init(&mut self) {
        self.load_texture(include_bytes!("../assets/faceplate.png"));
        self.load_texture(include_bytes!("../assets/jack_input.png"));
        self.load_texture(include_bytes!("../assets/jack_output.png"));
        self.load_texture(include_bytes!("../assets/knob_basic128.png"));
        self.load_texture(include_bytes!("../assets/knob_basic4.png"));
        self.load_texture(include_bytes!("../assets/slider_cable201_64x32.png"));
        self.load_texture(include_bytes!("../assets/slider_128_35x90.png"));
        self.load_texture(include_bytes!("../assets/meter_master31_35x120.png"));

        self.init_osc1();
        self.init_osc2();
        self.init_midi();
        self.init_lfos();
        self.init_envs();
        self.init_filters();
        self.init_effects();

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

    pub fn left_mouse_up(&mut self, _clicks: u8)  {
        dragable::on_left_release_system(&mut self.dragables);
        drawable::on_left_release_system(&mut self.audio_channel, &mut self.drawables);
        jacks::on_left_release_system(&mut self.audio_channel, &mut self.jacks, self.mouse_pos);
    }

    pub fn right_mouse_down(&mut self, x: f32, y: f32, clicks: u8) {
        jacks::on_right_down_system(&mut self.audio_channel, &mut self.jacks, x, y, clicks);
    }

    pub fn right_mouse_up(&mut self, _clicks: u8) {
        jacks::on_right_release_system(&mut self.jacks);
    }

    pub fn mouse_move(&mut self, x: f32, y: f32, xrel: f32, yrel: f32) {
        self.mouse_pos = FPoint::new(x, y);
        dragable::on_mouse_move_system(&mut self.audio_channel, &mut self.dragables, xrel, yrel);
        drawable::on_mouse_move_system(&mut self.drawables, x, y);
        jacks::on_mouse_move_system(&mut self.audio_channel, &mut self.jacks, xrel, yrel);
    }

    pub fn text_input(&mut self, _text: String) {

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

impl <'a> Gui <'a> {
    fn init_osc1(&mut self) {
        // Knobs
        self.dragables.spawn(
            FRect::new(32.0, 48.0, KNOB_128_ANIMATION.width(), KNOB_128_ANIMATION.height()),
            0.5,
            (DragType::VERTICAL, OnDragBehavior::Osc1Level),
            dragable::OnDoubleClickBehavior::SetTo(0.5),
            KNOB_128_ANIMATION,
        ).unwrap();
        self.dragables.spawn(
            FRect::new(140.0, 48.0, KNOB_4_ANIMATION.width(), KNOB_4_ANIMATION.height()),
            0.0,
            (DragType::VERTICAL, OnDragBehavior::Osc1Shape),
            dragable::OnDoubleClickBehavior::SetTo(0.0),
            KNOB_4_ANIMATION,
        ).unwrap();
        self.dragables.spawn(
            FRect::new(32.0, 160.0, KNOB_128_ANIMATION.width(), KNOB_128_ANIMATION.height()),
            0.0,
            (DragType::VERTICAL, OnDragBehavior::Osc1Phase),
            dragable::OnDoubleClickBehavior::SetTo(0.0),
            KNOB_128_ANIMATION,
        ).unwrap();
        self.dragables.spawn(
            FRect::new(142.0, 160.0, KNOB_128_ANIMATION.width(), KNOB_128_ANIMATION.height()),
            0.5,
            (DragType::VERTICAL, OnDragBehavior::Osc1Freq),
            dragable::OnDoubleClickBehavior::SetTo(0.5),
            KNOB_128_ANIMATION,
        ).unwrap();

        // Inputs
        self.jacks.spawn_input(
            FRect::new(280.0, 86.0, JACK_WIDTH, JACK_HEIGHT),
            InputJack::Osc1Freq,
        ).unwrap();
        self.jacks.spawn_input(
            FRect::new(280.0, 190.0, JACK_WIDTH, JACK_HEIGHT),
            InputJack::Osc1Phase,
        ).unwrap();
        self.jacks.spawn_input(
            FRect::new(358.0, 148.0, JACK_WIDTH, JACK_HEIGHT),
            InputJack::Osc1Amp,
        ).unwrap();
        self.jacks.spawn_input(
            FRect::new(436.0, 86.0, JACK_WIDTH, JACK_HEIGHT),
            InputJack::Osc1Level,
        ).unwrap();

        // Output
        self.jacks.spawn_output(
            FRect::new(436.0, 190.0, JACK_WIDTH, JACK_HEIGHT),
            OutputJack::Osc1Value,
        ).unwrap();
    }

    fn init_osc2(&mut self) {
        // Knobs
        self.dragables.spawn(
            FRect::new(1173.0, 50.0, KNOB_128_ANIMATION.width(), KNOB_128_ANIMATION.height()),
            0.5,
            (DragType::VERTICAL, OnDragBehavior::Osc2Level),
            dragable::OnDoubleClickBehavior::SetTo(0.5),
            KNOB_128_ANIMATION,
        ).unwrap();
        self.dragables.spawn(
            FRect::new(1026.0, 50.0, KNOB_128_ANIMATION.width(), KNOB_128_ANIMATION.height()),
            0.5,
            (DragType::VERTICAL, OnDragBehavior::Osc2Freq),
            dragable::OnDoubleClickBehavior::SetTo(0.5),
            KNOB_128_ANIMATION,
        ).unwrap();
        self.dragables.spawn(
            FRect::new(1100.0, 158.0, KNOB_128_ANIMATION.width(), KNOB_128_ANIMATION.height()),
            0.0,
            (DragType::VERTICAL, OnDragBehavior::Osc2Level),
            dragable::OnDoubleClickBehavior::SetTo(0.0),
            KNOB_128_ANIMATION,
        ).unwrap();

        // Inputs
        self.jacks.spawn_input(
            FRect::new(920.0, 86.0, JACK_WIDTH, JACK_HEIGHT),
            InputJack::Osc2Freq,
        ).unwrap();
        self.jacks.spawn_input(
            FRect::new(920.0, 192.0, JACK_WIDTH, JACK_HEIGHT),
            InputJack::Osc2Phase,
        ).unwrap();
        self.jacks.spawn_input(
            FRect::new(856.0, 140.0, JACK_WIDTH, JACK_HEIGHT),
            InputJack::Osc2Amp,
        ).unwrap();
        self.jacks.spawn_input(
            FRect::new(790.0, 86.0, JACK_WIDTH, JACK_HEIGHT),
            InputJack::Osc2Level,
        ).unwrap();

        // Output
        self.jacks.spawn_output(
            FRect::new(790.0, 192.0, JACK_WIDTH, JACK_HEIGHT),
            OutputJack::Osc2Value,
        ).unwrap();
    }

    fn init_midi(&mut self) {
        self.jacks.spawn_output(
            FRect::new(491.0, 379.0, JACK_WIDTH, JACK_HEIGHT),
            OutputJack::MidiGate,
        ).unwrap();
        self.jacks.spawn_output(
            FRect::new(614.0, 379.0, JACK_WIDTH, JACK_HEIGHT),
            OutputJack::MidiVelocity,
        ).unwrap();
        self.jacks.spawn_output(
            FRect::new(754.0, 379.0, JACK_WIDTH, JACK_HEIGHT),
            OutputJack::MidiNote,
        ).unwrap();
    }

    fn init_lfos(&mut self) {
        // Knobs
        self.dragables.spawn(
            FRect::new(64.0, 312.0, KNOB_4_ANIMATION.width(), KNOB_4_ANIMATION.height()),
            0.0,
            (DragType::VERTICAL, OnDragBehavior::Lfo1Shape),
            dragable::OnDoubleClickBehavior::SetTo(0.0),
            KNOB_4_ANIMATION,
        ).unwrap();
        self.dragables.spawn(
            FRect::new(62.0, 409.0, KNOB_4_ANIMATION.width(), KNOB_4_ANIMATION.height()),
            0.0,
            (DragType::VERTICAL, OnDragBehavior::Lfo2Shape),
            dragable::OnDoubleClickBehavior::SetTo(0.0),
            KNOB_4_ANIMATION,
        ).unwrap();
        self.dragables.spawn(
            FRect::new(170.0, 312.0, KNOB_128_ANIMATION.width(), KNOB_128_ANIMATION.height()),
            0.5,
            (DragType::VERTICAL, OnDragBehavior::Lfo1Freq),
            dragable::OnDoubleClickBehavior::SetTo(0.5),
            KNOB_128_ANIMATION,
        ).unwrap();
        self.dragables.spawn(
            FRect::new(170.0, 409.0, KNOB_128_ANIMATION.width(), KNOB_128_ANIMATION.height()),
            0.5,
            (DragType::VERTICAL, OnDragBehavior::Lfo2Freq),
            dragable::OnDoubleClickBehavior::SetTo(0.5),
            KNOB_128_ANIMATION,
        ).unwrap();

        // // Outputs
        // self.jacks.spawn_output(
        //     FRect::new(284.0, 328.0, JACK_WIDTH, JACK_HEIGHT),
        //     OutputJack::Lfo1Value,
        // ).unwrap();
        // self.jacks.spawn_output(
        //     FRect::new(364.0, 426.0, JACK_WIDTH, JACK_HEIGHT),
        //     OutputJack::Lfo2Value,
        // ).unwrap();
    }

    fn init_envs(&mut self) {
        // Env1 ------
        // Inputs
        self.jacks.spawn_input(
            FRect::new(103.0, 546.0, JACK_WIDTH, JACK_HEIGHT),
            InputJack::Env1Gate,
        ).unwrap();
        self.jacks.spawn_input(
            FRect::new(103.0, 592.0, JACK_WIDTH, JACK_HEIGHT),
            InputJack::Env1Vel,
        ).unwrap();
        self.jacks.spawn_input(
            FRect::new(9.0, 634.0, JACK_WIDTH, JACK_HEIGHT),
            InputJack::Env1Attack,
        ).unwrap();
        self.jacks.spawn_input(
            FRect::new(48.0, 635.0, JACK_WIDTH, JACK_HEIGHT),
            InputJack::Env1Decay,
        ).unwrap();
        self.jacks.spawn_input(
            FRect::new(86.0, 635.0, JACK_WIDTH, JACK_HEIGHT),
            InputJack::Env1Sustain,
        ).unwrap();
        self.jacks.spawn_input(
            FRect::new(123.0, 635.0, JACK_WIDTH, JACK_HEIGHT),
            InputJack::Env1Release,
        ).unwrap();
        // Sliders
        self.dragables.spawn(
            FRect::new(10.0, 666.0, SLIDER_128_ANIMATION.width(), SLIDER_128_ANIMATION.height()),
            0.0,
            (DragType::VERTICAL, OnDragBehavior::Env1Attack),
            dragable::OnDoubleClickBehavior::SetTo(0.0),
            SLIDER_128_ANIMATION,
        ).unwrap();
        self.dragables.spawn(
            FRect::new(49.0, 666.0, SLIDER_128_ANIMATION.width(), SLIDER_128_ANIMATION.height()),
            0.0,
            (DragType::VERTICAL, OnDragBehavior::Env1Decay),
            dragable::OnDoubleClickBehavior::SetTo(0.0),
            SLIDER_128_ANIMATION,
        ).unwrap();
        self.dragables.spawn(
            FRect::new(87.0, 666.0, SLIDER_128_ANIMATION.width(), SLIDER_128_ANIMATION.height()),
            0.0,
            (DragType::VERTICAL, OnDragBehavior::Env1Sustain),
            dragable::OnDoubleClickBehavior::SetTo(0.0),
            SLIDER_128_ANIMATION,
        ).unwrap();
        self.dragables.spawn(
            FRect::new(124.0, 666.0, SLIDER_128_ANIMATION.width(), SLIDER_128_ANIMATION.height()),
            0.0,
            (DragType::VERTICAL, OnDragBehavior::Env1Release),
            dragable::OnDoubleClickBehavior::SetTo(0.0),
            SLIDER_128_ANIMATION,
        ).unwrap();

        // ENV2 ------
        // Inputs
        self.jacks.spawn_input(
            FRect::new(278.0, 547.0, JACK_WIDTH, JACK_HEIGHT),
            InputJack::Env2Gate,
        ).unwrap();
        self.jacks.spawn_input(
            FRect::new(278.0, 593.0, JACK_WIDTH, JACK_HEIGHT),
            InputJack::Env2Vel,
        ).unwrap();
        self.jacks.spawn_input(
            FRect::new(184.0, 635.0, JACK_WIDTH, JACK_HEIGHT),
            InputJack::Env2Attack,
        ).unwrap();
        self.jacks.spawn_input(
            FRect::new(223.0, 636.0, JACK_WIDTH, JACK_HEIGHT),
            InputJack::Env2Decay,
        ).unwrap();
        self.jacks.spawn_input(
            FRect::new(261.0, 636.0, JACK_WIDTH, JACK_HEIGHT),
            InputJack::Env2Sustain,
        ).unwrap();
        self.jacks.spawn_input(
            FRect::new(298.0, 636.0, JACK_WIDTH, JACK_HEIGHT),
            InputJack::Env2Release,
        ).unwrap();
        // Sliders
        self.dragables.spawn(
            FRect::new(185.0, 667.0, SLIDER_128_ANIMATION.width(), SLIDER_128_ANIMATION.height()),
            0.0,
            (DragType::VERTICAL, OnDragBehavior::Env2Attack),
            dragable::OnDoubleClickBehavior::SetTo(0.0),
            SLIDER_128_ANIMATION,
        ).unwrap();
        self.dragables.spawn(
            FRect::new(224.0, 667.0, SLIDER_128_ANIMATION.width(), SLIDER_128_ANIMATION.height()),
            0.0,
            (DragType::VERTICAL, OnDragBehavior::Env2Decay),
            dragable::OnDoubleClickBehavior::SetTo(0.0),
            SLIDER_128_ANIMATION,
        ).unwrap();
        self.dragables.spawn(
            FRect::new(262.0, 667.0, SLIDER_128_ANIMATION.width(), SLIDER_128_ANIMATION.height()),
            0.0,
            (DragType::VERTICAL, OnDragBehavior::Env2Sustain),
            dragable::OnDoubleClickBehavior::SetTo(0.0),
            SLIDER_128_ANIMATION,
        ).unwrap();
        self.dragables.spawn(
            FRect::new(299.0, 667.0, SLIDER_128_ANIMATION.width(), SLIDER_128_ANIMATION.height()),
            0.0,
            (DragType::VERTICAL, OnDragBehavior::Env2Release),
            dragable::OnDoubleClickBehavior::SetTo(0.0),
            SLIDER_128_ANIMATION,
        ).unwrap();

        // ENV 3 ------
        // Inputs
        self.jacks.spawn_input(
            FRect::new(450.0, 547.0, JACK_WIDTH, JACK_HEIGHT),
            InputJack::Env3Gate,
        ).unwrap();
        self.jacks.spawn_input(
            FRect::new(450.0, 593.0, JACK_WIDTH, JACK_HEIGHT),
            InputJack::Env3Vel,
        ).unwrap();
        self.jacks.spawn_input(
            FRect::new(356.0, 635.0, JACK_WIDTH, JACK_HEIGHT),
            InputJack::Env3Attack,
        ).unwrap();
        self.jacks.spawn_input(
            FRect::new(395.0, 636.0, JACK_WIDTH, JACK_HEIGHT),
            InputJack::Env3Decay,
        ).unwrap();
        self.jacks.spawn_input(
            FRect::new(433.0, 636.0, JACK_WIDTH, JACK_HEIGHT),
            InputJack::Env3Sustain,
        ).unwrap();
        self.jacks.spawn_input(
            FRect::new(470.0, 636.0, JACK_WIDTH, JACK_HEIGHT),
            InputJack::Env3Release,
        ).unwrap();
        // Sliders
        self.dragables.spawn(
            FRect::new(357.0, 667.0, SLIDER_128_ANIMATION.width(), SLIDER_128_ANIMATION.height()),
            0.0,
            (DragType::VERTICAL, OnDragBehavior::Env3Attack),
            dragable::OnDoubleClickBehavior::SetTo(0.0),
            SLIDER_128_ANIMATION,
        ).unwrap();
        self.dragables.spawn(
            FRect::new(396.0, 667.0, SLIDER_128_ANIMATION.width(), SLIDER_128_ANIMATION.height()),
            0.0,
            (DragType::VERTICAL, OnDragBehavior::Env3Decay),
            dragable::OnDoubleClickBehavior::SetTo(0.0),
            SLIDER_128_ANIMATION,
        ).unwrap();
        self.dragables.spawn(
            FRect::new(434.0, 667.0, SLIDER_128_ANIMATION.width(), SLIDER_128_ANIMATION.height()),
            0.0,
            (DragType::VERTICAL, OnDragBehavior::Env3Sustain),
            dragable::OnDoubleClickBehavior::SetTo(0.0),
            SLIDER_128_ANIMATION,
        ).unwrap();
        self.dragables.spawn(
            FRect::new(471.0, 667.0, SLIDER_128_ANIMATION.width(), SLIDER_128_ANIMATION.height()),
            0.0,
            (DragType::VERTICAL, OnDragBehavior::Env3Release),
            dragable::OnDoubleClickBehavior::SetTo(0.0),
            SLIDER_128_ANIMATION,
        ).unwrap();

        // Outputs
        self.jacks.spawn_output(
            FRect::new(552.0, 564.0, JACK_WIDTH, JACK_HEIGHT),
            OutputJack::Env1Value,
        ).unwrap();
        self.jacks.spawn_output(
            FRect::new(552.0, 634.0, JACK_WIDTH, JACK_HEIGHT),
            OutputJack::Env2Value,
        ).unwrap();
        self.jacks.spawn_output(
            FRect::new(552.0, 706.0, JACK_WIDTH, JACK_HEIGHT),
            OutputJack::Env3Value,
        ).unwrap();
    }

    fn init_filters(&mut self) {
        // Knobs
        self.dragables.spawn(
            FRect::new(1144.0, 326.0, KNOB_128_ANIMATION.width(), KNOB_128_ANIMATION.height()),
            0.7,
            (DragType::VERTICAL, OnDragBehavior::Filter1Freq),
            dragable::OnDoubleClickBehavior::SetTo(0.7),
            KNOB_128_ANIMATION,
        ).unwrap();
        self.dragables.spawn(
            FRect::new(1144.0, 414.0, KNOB_128_ANIMATION.width(), KNOB_128_ANIMATION.height()),
            0.7,
            (DragType::VERTICAL, OnDragBehavior::Filter2Freq),
            dragable::OnDoubleClickBehavior::SetTo(0.7),
            KNOB_128_ANIMATION,
        ).unwrap();

        // Inputs
        self.jacks.spawn_input(
            FRect::new(1010.0, 344.0, JACK_WIDTH, JACK_HEIGHT),
            InputJack::Filter1Cutoff,
        ).unwrap();
        self.jacks.spawn_input(
            FRect::new(1010.0, 428.0, JACK_WIDTH, JACK_HEIGHT),
            InputJack::Filter2Cutoff,
        ).unwrap();
        self.jacks.spawn_input(
            FRect::new(936.0, 344.0, JACK_WIDTH, JACK_HEIGHT),
            InputJack::Filter1Value,
        ).unwrap();
        self.jacks.spawn_input(
            FRect::new(936.0, 428.0, JACK_WIDTH, JACK_HEIGHT),
            InputJack::Filter2Value,
        ).unwrap();

        // Outputs
        self.jacks.spawn_output(
            FRect::new(864.0, 344.0, JACK_WIDTH, JACK_HEIGHT),
            OutputJack::Filter1Value,
        ).unwrap();
        self.jacks.spawn_output(
            FRect::new(864.0, 428.0, JACK_WIDTH, JACK_HEIGHT),
            OutputJack::Filter2Value,
        ).unwrap();
    }

    fn init_effects(&mut self) {
        // Input
        self.jacks.spawn_input(
            FRect::new(671.0, 705.0, JACK_WIDTH, JACK_HEIGHT),
            InputJack::EffectsChain,
        ).unwrap();

        // Dist
        self.dragables.spawn(
            FRect::new(746.0, 586.0, KNOB_128_ANIMATION.width(), KNOB_128_ANIMATION.height()),
            0.0,
            (DragType::VERTICAL, OnDragBehavior::EffectDistDrive),
            dragable::OnDoubleClickBehavior::SetTo(0.0),
            KNOB_128_ANIMATION,
        ).unwrap();
        self.dragables.spawn(
            FRect::new(746.0, 676.0, KNOB_128_ANIMATION.width(), KNOB_128_ANIMATION.height()),
            0.0,
            (DragType::VERTICAL, OnDragBehavior::EffectDistWet),
            dragable::OnDoubleClickBehavior::SetTo(0.0),
            KNOB_128_ANIMATION,
        ).unwrap();

        // Delay
        self.dragables.spawn(
            FRect::new(888.0, 559.0, KNOB_128_ANIMATION.width(), KNOB_128_ANIMATION.height()),
            0.0,
            (DragType::VERTICAL, OnDragBehavior::EffectDelayFeedback),
            dragable::OnDoubleClickBehavior::SetTo(0.0),
            KNOB_128_ANIMATION,
        ).unwrap();
        self.dragables.spawn(
            FRect::new(888.0, 628.0, KNOB_128_ANIMATION.width(), KNOB_128_ANIMATION.height()),
            0.5,
            (DragType::VERTICAL, OnDragBehavior::EffectDelayTime),
            dragable::OnDoubleClickBehavior::SetTo(0.5),
            KNOB_128_ANIMATION,
        ).unwrap();
        self.dragables.spawn(
            FRect::new(888.0, 700.0, KNOB_128_ANIMATION.width(), KNOB_128_ANIMATION.height()),
            0.0,
            (DragType::VERTICAL, OnDragBehavior::EffectDelayWet),
            dragable::OnDoubleClickBehavior::SetTo(0.0),
            KNOB_128_ANIMATION,
        ).unwrap();

        // Reverb
        self.dragables.spawn(
            FRect::new(1025.0, 586.0, KNOB_128_ANIMATION.width(), KNOB_128_ANIMATION.height()),
            0.0,
            (DragType::VERTICAL, OnDragBehavior::EffectReverbTime),
            dragable::OnDoubleClickBehavior::SetTo(0.0),
            KNOB_128_ANIMATION,
        ).unwrap();
        self.dragables.spawn(
            FRect::new(1025.0, 676.0, KNOB_128_ANIMATION.width(), KNOB_128_ANIMATION.height()),
            0.0,
            (DragType::VERTICAL, OnDragBehavior::EffectReverbWet),
            dragable::OnDoubleClickBehavior::SetTo(0.0),
            KNOB_128_ANIMATION,
        ).unwrap();

        // Master
        self.dragables.spawn(
            FRect::new(1160.0, 700.0, KNOB_128_ANIMATION.width(), KNOB_128_ANIMATION.height()),
            0.7,
            (DragType::VERTICAL, OnDragBehavior::MasterGain),
            dragable::OnDoubleClickBehavior::SetTo(0.7),
            KNOB_128_ANIMATION,
        ).unwrap();
    }
}