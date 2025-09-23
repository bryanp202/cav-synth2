// pub enum GuiMessage {
//     Osc1(f32, f32, f32, f32, u8),
//     Thing(String),
// }

// pub struct GuiState {
//     clickables: _, // Send message and set mouse_move_message and text_input_message to the message provided by the (both to None if no collision)
//     selected_draggable: Option<fn(f32, f32, f32, f32, u8) -> GuiMessage>, // Resets to None on MouseLeftRelease
//     selected_text: Option<fn(String) -> GuiMessage>, // Only resets through clickables/collision testing    
// }

// impl GuiState {
//     pub fn new() -> Self {
//         Self {
//             selected_draggable: Some(GuiMessage::Osc1),
//             selected_text: Some(GuiMessage::Thing),
//         }
//     }

//     pub fn render(&self) {
//         // Hard coded to
//     }

//     pub fn update(&mut self, event: Event) -> GuiMessage {

//     }
// }

use sdl3::{video::Window, Error};
use sdl3::render::{Canvas, FRect, Texture};

pub fn button(canvas: &mut Canvas<Window>, texture: &Texture, dest: FRect) -> Result<(), Error> {
    canvas.copy(texture, None, dest)?;
    Ok(())
}

pub fn knob(canvas: &mut Canvas<Window>, texture: &Texture, dest: FRect, angle: f64) -> Result<(), Error> {
    canvas.copy_ex(texture, None, dest, angle, None, false, false)
}