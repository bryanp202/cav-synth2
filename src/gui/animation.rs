use sdl3::render::{FRect, Texture};

/// Animation frames are stored vertically
pub struct Animation {
    texture_index: usize,
    frames: usize,
    frame_w: f32,
    frame_h: f32,
}

impl Animation {
    pub const fn new_comptime(texture_index: usize, frames: usize, frame_w: f32, frame_h: f32) -> Animation {
        Self { texture_index, frames, frame_w, frame_h }
    }

    #[allow(dead_code)]
    pub fn new(texture_index: usize, frames: usize, frame_w: f32, frame_h: f32) -> Animation {
        Self { texture_index, frames, frame_w, frame_h }
    }

    pub fn width(&self) -> f32 {
        self.frame_w
    }
    
    pub fn height(&self) -> f32 {
        self.frame_h
    }

    pub fn get_frame_count(&self) -> usize {
        self.frames
    }

    pub fn get_frame<'r>(&self, frame: usize, textures: &'r [Texture]) -> (&'r Texture<'r>, FRect) {
        let src_rect = FRect::new(0.0, frame as f32 * self.frame_h, self.frame_w, self.frame_h);
        (&textures[self.texture_index], src_rect)
    }
}
