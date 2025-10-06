use sdl3::{render::{Canvas, FRect, Texture}, video::Window};

use crate::gui::METER_MASTER_ANIMATION;

pub struct Meters {
    master_dst: (FRect, FRect),
    master_level: (f32, f32),
}

impl Meters {
    pub fn new() -> Self {
        Self {
            master_dst: (FRect::new(0.0, 0.0, 0.0, 0.0), FRect::new(0.0, 0.0, 0.0, 0.0)),
            master_level: (0.0, 0.0),
        }
    }

    pub fn init(&mut self, left_master: FRect, right_master: FRect) {
        self.master_dst = (left_master, right_master);
    }

    pub fn set_level(&mut self, left: f32, right: f32) {
        self.master_level = (left, right);
    }

    pub fn render(&self, canvas: &mut Canvas<Window>, textures: &[Texture]) -> Result<(), sdl3::Error> {
        let frame = (METER_MASTER_ANIMATION.get_frame_count() - 1) as f32 * self.master_level.0;
        let (texture, src) = METER_MASTER_ANIMATION.get_frame(frame as usize, textures);
        canvas.copy(texture, src, self.master_dst.0)?;

        let frame = (METER_MASTER_ANIMATION.get_frame_count() - 1) as f32 * self.master_level.1;
        let (texture, src) = METER_MASTER_ANIMATION.get_frame(frame as usize, textures);
        canvas.copy(texture, src, self.master_dst.1)?;

        Ok(())
    } 
}