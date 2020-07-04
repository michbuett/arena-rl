use sdl2::rect::{Rect, Point};
use crate::core::{UserInput};

pub struct ScreenPos(pub i32, pub i32);

impl ScreenPos {
    // pub fn to_xy(self: &Self) -> (i32, i32) {
    //     (self.0, self.1)
    // }

    pub fn to_point(&self) -> Point {
        Point::new(self.0, self.1)
    }
}

pub struct ClickArea {
    pub clipping_area: Rect,
    pub action: Box<dyn Fn(ScreenPos) -> UserInput>,
}

pub type ClickAreas = Vec<ClickArea>;

pub struct UI {
    pub pixel_ratio: u8,
    pub viewport: Rect,
    pub fps: u128,
    pub frames: u32,
    pub last_check: std::time::Instant,
    pub is_scrolling: bool,
    pub has_scrolled: bool,
    pub scroll_offset: (i32, i32),
}

// struct Game<'a, 'b> {
//     pub ui: UI,
//     pub game: crate::core::Game<'a, 'b>,
// }

// pub enum Screen {
//     Combat{
//         is_scrolling: bool,
//         scroll_offset: (i32, i32),
//     }
// }
