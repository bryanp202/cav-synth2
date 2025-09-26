use sdl3::{render::{Canvas, FPoint, FRect, Texture}, video::Window};

use crate::{common::{frect_center, point_in_frect, ComponentVec}, gui::cable::{self, Cable, Cables}};

const MAX_INPUT_JACKS: usize = 64;
const MAX_OUTPUT_JACKS: usize = 64;

pub enum InputJack {
    Osc1Freq,
}

pub enum OutputJack {
    Osc1Value,
}

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
}

impl JackData {
    pub fn new() -> Self {
        Self {
            inputs: InputJacks { rect: ComponentVec::new(), on_connect: ComponentVec::new() },
            outputs: OutputJacks { rect: ComponentVec::new(), on_connect: ComponentVec::new() },
            cables: Cables::new(),
            clicked_input: None,
            clicked_output: None,
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

pub fn render_system(canvas: &mut Canvas<Window>, output_texture: &Texture, input_texture: &Texture, jack_data: &JackData, mouse_pos: FPoint) -> Result<(), sdl3::Error> {
    for &rect in jack_data.outputs.rect.iter() {
        canvas.copy(output_texture, None, Some(rect))?;
    }
    for &rect in jack_data.inputs.rect.iter() {
        canvas.copy(input_texture, None, Some(rect))?;
    }
    cable::render_system(canvas, &jack_data.cables)?;
    match (jack_data.clicked_input, jack_data.clicked_output) {
        (Some((_, start)), None) => cable::draw_cable(canvas, start, mouse_pos),
        (None, Some((_, start))) => cable::draw_cable(canvas, start, mouse_pos),
        _ => {},
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

pub fn on_left_release_system(jack_data: &mut JackData, mouse_pos: FPoint) {
    if let Some((input_index, start)) = jack_data.clicked_input {
        for (output_index, end_rect) in jack_data.outputs.rect.iter().enumerate() {
            if point_in_frect(end_rect, mouse_pos.x, mouse_pos.y) {
                jack_data.cables.push(Cable::new(start, frect_center(end_rect))).unwrap();
                break;
            }
        }
        jack_data.clicked_input = None;
    } else if let Some((output_index, start)) = jack_data.clicked_output {
        for (input_index, end_rect) in jack_data.inputs.rect.iter().enumerate() {
            if point_in_frect(end_rect, mouse_pos.x, mouse_pos.y) {
                jack_data.cables.push(Cable::new(start, frect_center(end_rect))).unwrap();
                break;
            }
        }
        jack_data.clicked_output = None;
    }
}

pub fn on_right_down_system(jack_data: &mut JackData, x: f32, y: f32) {
    let mouse_pos = FPoint::new(x, y);
    let maybe_remove_index = jack_data.cables.iter().rev().position(|cable| cable.is_touching(mouse_pos));
    if let Some(remove_index) = maybe_remove_index {
        jack_data.cables.remove(jack_data.cables.len() - remove_index - 1);
    }
}