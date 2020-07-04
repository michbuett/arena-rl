#[derive(Debug, Clone, Copy)]
pub struct WorldPos(pub f32, pub f32);

impl WorldPos {
    pub fn translate(&self, dx: f32, dy: f32) -> Self {
        WorldPos(self.0 + dx, self.1 + dy)
    }

    pub fn distance(Self(x1, y1): &Self, Self(x2, y2): &Self) -> f32 {
        let dx = x2 - x1;
        let dy = y2 - y1;
        f32::sqrt(dx * dx + dy * dy)
    }

    // pub fn diag_distance(Self(x1, y1): &Self, Self(x2, y2): &Self) -> f32 {
    //     let dx = x2 - x1;
    //     let dy = y2 - y1;
    //     f32::max(f32::abs(dx), f32::abs(dy))
    // }

    pub fn lerp(Self(x1, y1): &Self, Self(x2, y2): &Self, t: f32) -> Self {
        let xn = x1 + t * (x2 - x1);
        let yn = y1 + t * (y2 - y1);
        WorldPos(xn, yn)
        // WorldPos(lerp(*x1, *x2, t), lerp(*y1, *y2, t))
    }
}

use std::ops::Sub;
impl Sub for &WorldPos {
    type Output = (f32, f32);

    fn sub(self, other: &WorldPos) -> (f32, f32) {
        (self.0 - other.0, self.1 - other.1)
    }
}

// fn lerp(start: f32, end: f32, t: f32) -> f32 {
//     start + t * (end - start)
// }


// #[derive(Debug, Clone, Copy)]
// pub struct Tile(pub u32, pub u32, pub TileType);
//
// #[derive(Debug, Clone, Copy)]
// pub enum TileType{
//     Floor,
//     Void
// }

// pub enum TileDescriptors {
//     Empty(W
// }

