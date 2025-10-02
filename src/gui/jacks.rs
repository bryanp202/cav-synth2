use std::{collections::HashSet, sync::mpsc};

use sdl3::{render::{Canvas, FPoint, FRect, Texture}, video::Window};

use crate::{audio::{AudioMessage, InputJack, OutputJack}, common::{frect_center, point_in_frect, ComponentVec}, gui::{animation::Animation, cable::{self, Cable, Cables}}, SCREEN_HEIGHT, SCREEN_WIDTH};

const MAX_INPUT_JACKS: usize = 64;
const MAX_OUTPUT_JACKS: usize = 64;

struct InputJacks {
    rect: ComponentVec<FRect, MAX_INPUT_JACKS>,
    on_connect: ComponentVec<InputJack, MAX_INPUT_JACKS>,
}

struct OutputJacks {
    rect: ComponentVec<FRect, MAX_OUTPUT_JACKS>,
    on_connect: ComponentVec<OutputJack, MAX_OUTPUT_JACKS>,
}

pub struct JackData {
    inputs: InputJacks,
    outputs: OutputJacks,
    clicked_input: Option<(usize, FPoint)>,
    clicked_output: Option<(usize, FPoint)>,
    
    cables: Cables,
    cable_combos: HashSet<(InputJack, OutputJack)>,
    right_clicked_cable: Option<(usize, FRect)>,
    cable_slider_animation: Animation,
}

impl JackData {
    pub fn new() -> Self {
        Self {
            inputs: InputJacks { rect: ComponentVec::new(), on_connect: ComponentVec::new() },
            outputs: OutputJacks { rect: ComponentVec::new(), on_connect: ComponentVec::new() },
            cables: Cables::new(),
            clicked_input: None,
            clicked_output: None,
            right_clicked_cable: None,
            cable_slider_animation: Animation::new(super::CABLE_SLIDER_TEXTURE, 201, 64.0, 32.0),
            cable_combos: HashSet::new(),
        }
    }

    pub fn spawn_input(&mut self, rect: FRect, on_connect: InputJack) -> Result<(), ()> {
        self.inputs.on_connect.push(on_connect)?;
        self.inputs.rect.push(rect)?;
        Ok(())
    }

    pub fn spawn_output(&mut self, rect: FRect, on_connect: OutputJack) -> Result<(), ()> {
        self.outputs.on_connect.push(on_connect)?;
        self.outputs.rect.push(rect)?;
        Ok(())
    }
}

pub fn render_system(canvas: &mut Canvas<Window>, textures: &[Texture], jack_data: &JackData, mouse_pos: FPoint) -> Result<(), sdl3::Error> {
    for &rect in jack_data.outputs.rect.iter() {
        canvas.copy(&textures[super::JACK_OUTPUT_TEXTURE], None, Some(rect))?;
    }
    for &rect in jack_data.inputs.rect.iter() {
        canvas.copy(&textures[super::JACK_INPUT_TEXTURE], None, Some(rect))?;
    }
    cable::render_system(canvas, &jack_data.cables)?;
    match (jack_data.clicked_input, jack_data.clicked_output) {
        (Some((_, start)), None) => cable::draw_cable(canvas, start, mouse_pos),
        (None, Some((_, start))) => cable::draw_cable(canvas, start, mouse_pos),
        _ => {},
    }
    if let Some((cable_index, dst)) = jack_data.right_clicked_cable {
        let cable = &jack_data.cables[cable_index];
        let animation_frame = ((jack_data.cable_slider_animation.get_frame_count() - 1) as f32 * (cable.value() / 2.0 + 0.5)) as usize;
        let (texture, src) = jack_data.cable_slider_animation.get_frame(animation_frame, textures);
        canvas.copy(texture, Some(src), Some(dst))?;
    }
    Ok(())
}

pub fn on_left_down_system(jack_data: &mut JackData, x: f32, y: f32) {
    for (i, rect) in jack_data.outputs.rect.iter().enumerate() {
        if point_in_frect(rect, x, y) {
            jack_data.clicked_output = Some((i, frect_center(rect)));
            return;
        }
    }
    for (i, rect) in jack_data.inputs.rect.iter().enumerate() {
        if point_in_frect(rect, x, y) {
            jack_data.clicked_input = Some((i, frect_center(rect)));
            return;
        }
    }
}

pub fn on_left_release_system(audio_channel: &mut mpsc::Sender<AudioMessage>, jack_data: &mut JackData, mouse_pos: FPoint) {
    if let Some((input_index, start)) = jack_data.clicked_input {
        for (output_index, end_rect) in jack_data.outputs.rect.iter().enumerate() {
            if point_in_frect(end_rect, mouse_pos.x, mouse_pos.y) {
                let combo = (jack_data.inputs.on_connect[input_index], jack_data.outputs.on_connect[output_index]);
                if !jack_data.cable_combos.contains(&combo) {
                    jack_data.cables.push(Cable::new(start, frect_center(end_rect), combo)).unwrap();
                    jack_data.cable_combos.insert(combo);
                    audio_channel.send(AudioMessage::CableConnection(combo.0, combo.1)).unwrap();
                }
                break;
            }
        }
        jack_data.clicked_input = None;
    } else if let Some((output_index, start)) = jack_data.clicked_output {
        for (input_index, end_rect) in jack_data.inputs.rect.iter().enumerate() {
            if point_in_frect(end_rect, mouse_pos.x, mouse_pos.y) {
                let combo = (jack_data.inputs.on_connect[input_index], jack_data.outputs.on_connect[output_index]);
                if !jack_data.cable_combos.contains(&combo) {
                    jack_data.cables.push(Cable::new(start, frect_center(end_rect), combo)).unwrap();
                    jack_data.cable_combos.insert(combo);
                    audio_channel.send(AudioMessage::CableConnection(combo.0, combo.1)).unwrap();
                }
                break;
            }
        }
        jack_data.clicked_output = None;
    }
}

pub fn on_right_down_system(audio_channel: &mut mpsc::Sender<AudioMessage>, jack_data: &mut JackData, x: f32, y: f32, clicks: u8) {
    let mouse_pos = FPoint::new(x, y);
    let maybe_remove_index = jack_data.cables.iter().rev().position(|cable| cable.is_touching(mouse_pos));

    match (clicks, maybe_remove_index) {
        (1, Some(clicked_index)) => {
            let x_pos = if x < SCREEN_WIDTH as f32 / 2.0 {
                x + 10.0 // Left side of screen
            } else {
                x - 64.0 - 10.0// Right side of screen
            };
            let y_pos = if y < SCREEN_HEIGHT as f32 / 2.0 {
                y + 10.0 // Top side of screen
            } else {
                y - 32.0 - 10.0// Bottom side of screen
            };
            let rect = FRect::new(x_pos, y_pos, 64.0, 32.0);
            let cable_index = jack_data.cables.len() - clicked_index - 1;
            jack_data.right_clicked_cable = Some((cable_index, rect));
        },
        (_, Some(remove_index)) => {
            let cable_index = jack_data.cables.len() - remove_index - 1;
            let cable_combo = jack_data.cables[cable_index].combo();
            jack_data.cable_combos.remove(&cable_combo);
            jack_data.cables.remove(cable_index);
            audio_channel.send(AudioMessage::CableRemove(cable_index)).unwrap();
        },
        _ => {},
    }
}

pub fn on_right_release_system(jack_data: &mut JackData) {
    jack_data.right_clicked_cable = None;
}

pub fn on_mouse_move_system(audio_channel: &mut mpsc::Sender<AudioMessage>, jack_data: &mut JackData, _xrel: f32, yrel: f32) {
    if let Some((cable_index, _)) = jack_data.right_clicked_cable {
        let drag_amt = -yrel/200.0;
        let cable = &mut jack_data.cables[cable_index];
        let new_value = (cable.value() + drag_amt).clamp(-1.0, 1.0);
        let animation_frames = jack_data.cable_slider_animation.get_frame_count();
        
        let old_frame = ((animation_frames - 1) as f32 * (cable.value() / 2.0 + 0.5)) as usize;
        let new_frame = (animation_frames - 1) as f32 * (new_value / 2.0 + 0.5);

        println!("val: {}, old: {}, new: {}", new_value, old_frame, new_frame);
        cable.set_value(new_value);
        if old_frame != new_frame as usize {
            audio_channel.send(AudioMessage::CableAttenuation(cable_index, new_value)).unwrap();
        }
    }
}