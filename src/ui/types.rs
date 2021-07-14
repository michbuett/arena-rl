use serde::Deserialize;

use crate::core::{DisplayStr, UserInput, WorldPos, Sprite};

pub const TILE_WIDTH: u32 = 128;
pub const TILE_HEIGHT: u32 = 128;

#[derive(Clone, Copy, Debug)]
pub struct ScreenCoord(
    /// the x coordinate
    i32,
    /// the y coordinate
    i32,
    /// the z coordinate
    i32
);

impl ScreenCoord {
    pub fn new(x: i32, y: i32) -> Self {
        Self(x, y, 0)
    }

    pub fn from_world_pos(wp: WorldPos) -> Self {
        let (xw, yw) = wp.as_xy();
        let tw = TILE_WIDTH as f32;
        let th = TILE_HEIGHT as f32;
        let x = tw * (xw - yw) / 2.0;
        let y = th * (xw + yw) / 2.0;
        let z = th * wp.z();

        Self(x.round() as i32, y.round() as i32, z.round() as i32)
    }

    pub fn to_world_pos(&self) -> WorldPos {
        let (xs, ys) = (self.0 as f32, self.1 as f32);
        let tw = TILE_WIDTH as f32;
        let th = TILE_HEIGHT as f32;
        let x = xs / tw + ys / th;
        let y = ys / th - xs / tw;
        let z = self.2 as f32 / th;

        WorldPos::new(x, y, z)
    }

    pub fn to_screen_pos(&self, scroll_offset: (i32, i32)) -> ScreenPos {
        ScreenPos(self.0 + scroll_offset.0, self.1 + scroll_offset.1 + self.2)
    }

    pub fn translate(self, dx: i32, dy: i32, dz: i32) -> ScreenCoord {
        Self(self.0 + dx, self.1 + dy, self.2 + dz)
    }

    pub fn translate_world_pos(wp: WorldPos, dx: i32, dy: i32) -> WorldPos {
        Self::from_world_pos(wp).translate(dx, dy, 0).to_world_pos()
    }

    pub fn euclidian_distance(self, other: Self) -> f32 {
        let dx = (other.0 - self.0) as f32;
        let dy = (other.1 - self.1) as f32;

        f32::sqrt(dx * dx + dy * dy)
    }

    pub fn z_layer(self) -> i32 {
        self.1
    }
}


#[derive(Clone, Copy, Debug)]
pub struct ScreenPos(pub i32, pub i32);

#[test]
fn mapping_between_world_and_screen_coordinates_is_isomorphic() {
    let wp = WorldPos::new(5.0, 10.0, 0.0);
    let sc = ScreenCoord::from_world_pos(wp);

    assert_eq!(wp.as_xy(), sc.to_world_pos().as_xy());
}

pub struct ClickArea {
    pub clipping_area: (i32, i32, u32, u32),
    pub action: Box<dyn Fn(ScreenPos) -> UserInput>,
}

pub type ClickAreas = Vec<ClickArea>;

pub struct UI {
    pub pixel_ratio: u8,
    pub viewport: (i32, i32, u32, u32),
    pub fps: u128,
    pub frames: u32,
    pub last_check: std::time::Instant,
    pub scrolling: Option<ScrollData>,
}

pub struct ScrollData {
    pub is_scrolling: bool,
    pub has_scrolled: bool,
    pub offset: (i32, i32),
}

#[derive(Debug, Copy, Clone)]
pub enum FontFace {
    Normal = 0,
    Big = 1,
    VeryBig = 2,
}

#[derive(Debug)]
pub struct ScreenText {
    pub font: FontFace,
    pub text: DisplayStr,
    pub pos: ScreenPos,
    pub color: (u8, u8, u8, u8),
    pub background: Option<(u8, u8, u8, u8)>,
    pub padding: u32,
    pub alpha: u8,

    /// e.g.
    /// Some(width, (red, green, blue, alpha))
    pub border: Option<(u32, (u8, u8, u8, u8))>,

    pub min_width: u32,
    pub max_width: u32,
}

impl ScreenText {
    pub fn new(text: DisplayStr, pos: ScreenPos) -> Self {
        Self {
            font: FontFace::Normal,
            text,
            pos,
            alpha: 255,
            color: (0, 0, 0, 255),
            background: None,
            padding: 0,
            border: None,
            min_width: 0,
            max_width: u32::max_value(),
        }
    }

    pub fn font(self: Self, font: FontFace) -> Self {
        Self { font, ..self }
    }

    pub fn color(self: Self, color: (u8, u8, u8, u8)) -> Self {
        Self { color, ..self }
    }

    pub fn background(self: Self, color: (u8, u8, u8, u8)) -> Self {
        Self {
            background: Some(color),
            ..self
        }
    }

    pub fn padding(self: Self, padding: u32) -> Self {
        Self {
            padding: padding,
            ..self
        }
    }

    pub fn border(self: Self, padding: u32, color: (u8, u8, u8, u8)) -> Self {
        Self {
            border: Some((padding, color)),
            ..self
        }
    }

    // pub fn max_width(self: Self, max_width: u32) -> Self {
    //     Self {
    //         max_width: max_width,
    //         ..self
    //     }
    // }

    // pub fn min_width(self: Self, min_width: u32) -> Self {
    //     Self {
    //         min_width: min_width,
    //         ..self
    //     }
    // }

    pub fn width(self: Self, width: u32) -> Self {
        Self {
            min_width: width,
            max_width: width,
            ..self
        }
    }
}

#[derive(Debug)]
pub struct Scene {
    pub background: (u8, u8, u8),
    pub texts: Vec<ScreenText>,
    pub sprites: Vec<ScreenSprite>,
    pub images: Vec<(String, ScreenSprite)>,
}

impl Scene {
    pub fn empty() -> Self {
        Self {
            background: (252, 251, 250),
            texts: vec!(),
            sprites: Vec::with_capacity(500),
            images: vec!(),
        }
    }

    pub fn set_background(self, r: u8, g: u8, b: u8) -> Self {
        Self {
            background: (r, g, b),
            ..self
        }
    }
}

#[derive(Debug)]
pub struct ScreenSprite(pub ScreenPos, pub Sprite);

#[derive(Debug, Clone, Deserialize)]
pub struct ProtoSpriteConfig {
    pub files: Vec<String>,
    pub offset: (i32, i32),
    pub alpha: u8,
    pub frame_durration: Option<u32>,
}
