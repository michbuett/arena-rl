// use crate::ui::{ScreenCoord, ScreenPos};

#[derive(Debug, Clone, Copy)]
pub struct WorldPos(f32, f32, f32);

impl WorldPos {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self(x, y, z)
    }

    pub fn translate_xy(self, dx: f32, dy: f32) -> Self {
        Self(self.0 + dx, self.1 + dy, self.2)
    }

    pub fn as_xy(&self) -> (f32, f32) {
        (self.0, self.1)
    }

    pub fn x(&self) -> f32 {
        self.0
    }

    pub fn y(&self) -> f32 {
        self.1
    }

    pub fn z(&self) -> f32 {
        self.2
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Direction(f32);

impl Direction {
    // pub fn from_point(from: WorldPos, to: WorldPos) -> Self {
    //     let ScreenPos(x0, y0) = ScreenCoord::from_world_pos(from).to_screen_pos((0, 0));
    //     let ScreenPos(x1, y1) = ScreenCoord::from_world_pos(to).to_screen_pos((0, 0));
    //     let dx = (x1 - x0) as f32;
    //     let dy = (y1 - y0) as f32;
    //     let alpha = dy.atan2(dx);

    //     Self(alpha)
    // }

    pub fn as_degree(&self) -> f64 {
        self.0.to_degrees() as f64
    }

    // pub fn as_radian(&self) -> f64 {
    //     self.0 as f64
    // }
}
