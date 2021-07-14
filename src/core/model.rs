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
