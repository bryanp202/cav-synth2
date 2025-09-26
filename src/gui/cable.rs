use core::f32;

use sdl3::{pixels::FColor, render::{Canvas, FPoint, Vertex, VertexIndices}, video::Window};

use crate::common::ComponentVec;

const CONNECT_CABLE_ALPHA: f32 = 0.4;
const MAX_CABLE_COUNT: usize = 128;
const CABLE_WIDTH: f32 = 12.0;

pub type Cables = ComponentVec<Cable, MAX_CABLE_COUNT>;

pub struct Cable (FPoint, FPoint, FPoint, FPoint);

impl Cable {
    pub fn new(start: FPoint, end: FPoint) -> Self {
        let angle = (end.y - start.y).atan2(end.x - start.x);
        let perp_ratio_cos = (angle - f32::consts::FRAC_PI_2).cos();
        let perp_ratio_sin = (angle - f32::consts::FRAC_PI_2).sin();
        let x_perp = CABLE_WIDTH / 2.0 * perp_ratio_cos;
        let y_perp = CABLE_WIDTH / 2.0 * perp_ratio_sin;
        Self (
            FPoint::new(start.x - x_perp , start.y - y_perp),
            FPoint::new(start.x + x_perp, start.y + y_perp),
            FPoint::new(end.x + x_perp, end.y + y_perp),
            FPoint::new(end.x - x_perp, end.y - y_perp),
        )
    }

    /// Requires that points in self are oriented in counter-clockwise order
    pub fn is_touching(&self, target: FPoint) -> bool {
        Self::is_left(self.0, self.1, target) &&
        Self::is_left(self.1, self.2, target) &&
        Self::is_left(self.2, self.3, target) &&
        Self::is_left(self.3, self.0, target)
    }

    fn is_left(p1: FPoint, p2: FPoint, target: FPoint) -> bool {
        let d = (p2.x - p1.x) * (target.y - p1.y) - (target.x - p1.x) * (p2.y - p1.y);
        d >= 0.0
    }
}

pub fn render_system(canvas: &mut Canvas<Window>, cables: &[Cable]) -> Result<(), sdl3::Error> {
    for cable in cables {
        let vertices = [
            new_vertex(cable.0, FColor::RGBA(0.0, 1.0, 1.0, CONNECT_CABLE_ALPHA), FPoint::new(1.0, 1.0)),
            new_vertex(cable.2, FColor::RGBA(0.0, 1.0, 1.0, CONNECT_CABLE_ALPHA), FPoint::new(1.0, 1.0)),
            new_vertex(cable.1, FColor::RGBA(0.0, 0.4, 0.4, CONNECT_CABLE_ALPHA), FPoint::new(1.0, 1.0)),
            new_vertex(cable.0, FColor::RGBA(0.0, 1.0, 1.0, CONNECT_CABLE_ALPHA), FPoint::new(1.0, 1.0)),
            new_vertex(cable.2, FColor::RGBA(0.0, 1.0, 1.0, CONNECT_CABLE_ALPHA), FPoint::new(1.0, 1.0)),
            new_vertex(cable.3, FColor::RGBA(0.0, 0.4, 0.4, CONNECT_CABLE_ALPHA), FPoint::new(1.0, 1.0)),
        ];
        let indices = VertexIndices::Sequential;
        canvas.render_geometry(&vertices, None, indices)?;
    }
    Ok(())
}

pub fn draw_cable(canvas: &mut Canvas<Window>, start: FPoint, end: FPoint) {
    let angle = (end.y - start.y).atan2(end.x - start.x);
    let perp_ratio_cos = (angle - f32::consts::FRAC_PI_2).cos();
    let perp_ratio_sin = (angle - f32::consts::FRAC_PI_2).sin();
    let x_perp = CABLE_WIDTH / 2.0 * perp_ratio_cos;
    let y_perp = CABLE_WIDTH / 2.0 * perp_ratio_sin;
    let vertices = [
        new_vertex(FPoint::new(start.x - x_perp , start.y - y_perp), FColor::CYAN, FPoint::new(1.0, 1.0)),
        new_vertex(FPoint::new(end.x + x_perp, end.y + y_perp), FColor::CYAN, FPoint::new(1.0, 1.0)),
        new_vertex(FPoint::new(end.x - x_perp, end.y - y_perp), FColor::RGB(0.0, 0.4, 0.4), FPoint::new(1.0, 1.0)),
        new_vertex(FPoint::new(start.x - x_perp, start.y - y_perp), FColor::CYAN, FPoint::new(1.0, 1.0)),
        new_vertex(FPoint::new(end.x + x_perp, end.y + y_perp), FColor::CYAN, FPoint::new(1.0, 1.0)),
        new_vertex(FPoint::new(start.x + x_perp, start.y + y_perp), FColor::RGB(0.0, 0.4, 0.4), FPoint::new(1.0, 1.0)),
    ];
    let indices = VertexIndices::Sequential;
    canvas.render_geometry(&vertices, None, indices).unwrap();
}

fn new_vertex(position: FPoint, color: FColor, tex_coord: FPoint) -> Vertex {
    Vertex { position, color, tex_coord }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn cable_collision() {
        let cable = Cable(FPoint::new(0.0, 0.0), FPoint::new(10.0, 0.0), FPoint::new(10.0, 10.0), FPoint::new(0.0, 10.0));
        assert!(cable.is_touching(FPoint::new(0.0, 0.0)));
        assert!(cable.is_touching(FPoint::new(4.0, 6.7)));
        assert!(!cable.is_touching(FPoint::new(-0.01, 6.7)));
        assert!(cable.is_touching(FPoint::new(10.0, 0.0)));
        assert!(!cable.is_touching(FPoint::new(10.1, 0.0)));
        assert!(!cable.is_touching(FPoint::new(10.0, -0.1)));
    }
}