use sdl2::rect::{Point, Rect};
// use std::collections::HashMap;

use serde::Deserialize;

use crate::core::{DisplayStr, UserInput, WorldPos, Sprite};

#[derive(Clone, Copy, Debug)]
pub struct ScreenPos(pub i32, pub i32);

pub const TILE_WIDTH: u32 = 128;
pub const TILE_HEIGHT: u32 = 128;

impl ScreenPos {
    pub fn to_point(&self) -> Point {
        Point::new(self.0, self.1)
    }

    pub fn to_xy(&self) -> (i32, i32) {
        (self.0, self.1)
    }

    pub fn to_world_pos(&self, scroll_offset: (i32, i32)) -> WorldPos {
        let xs = (self.0 - scroll_offset.0) as f32;
        let ys = (self.1 - scroll_offset.1) as f32;
        let tw = TILE_WIDTH as f32;
        let th = TILE_HEIGHT as f32;
        let x = xs / tw + ys / th;
        let y = ys / th - xs / tw;

        WorldPos(x, y)
    }

    pub fn from_world_pos(wp: WorldPos, scroll_offset: (i32, i32)) -> Self {
        let WorldPos(xw, yw) = wp;
        let tw = TILE_WIDTH as f32;
        let th = TILE_HEIGHT as f32;
        let x = tw * (xw - yw) / 2.0;
        let y = th * (xw + yw) / 2.0;

        Self(
            x.round() as i32 + scroll_offset.0,
            y.round() as i32 + scroll_offset.1,
        )
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
    pub scrolling: Option<ScrollData>,
    // pub textures: (&'static str, HashMap<String, SpriteConfig>),
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
}

impl Scene {
    pub fn empty() -> Self {
        Self {
            background: (252, 251, 250),
            texts: vec![],
            sprites: Vec::with_capacity(500),
        }
    }
}

#[derive(Debug)]
pub struct ScreenSprite(pub ScreenPos, pub Sprite);
// pub struct ScreenSprite {
//     pub source: (String, i32, i32, u32, u32),
//     pub pos: ScreenPos,
//     pub offset: (i32, i32),
//     pub alpha: u8,
//     pub target_size: (u32, u32),
// }

// #[derive(Debug)]
// pub struct ScreenSprite(Sprite, ScreenPos);
// pub struct Visual(Vec<String>);

#[derive(Debug, Clone, Deserialize)]
pub struct ProtoSpriteConfig {
    pub files: Vec<String>,
    pub offset: (i32, i32),
    pub alpha: u8,
    pub frame_durration: Option<u32>,
}

// #[derive(Debug, Clone)]
// pub struct SpriteConfig {
//     pub source: SpriteSource,
//     pub offset: (i32, i32),
//     pub dim: (u32, u32),
//     pub alpha: u8,
// }

// impl SpriteConfig {
//     pub fn into_screen_sprite(&self, pos: ScreenPos, runtime_ms: u128) -> ScreenSprite {
//         let (w, h) = self.dim;
//         let (x, y) = self.source.get_frame_pos(runtime_ms);
//         let source = ("".to_string(), x, y, w, h);

//         ScreenSprite {
//             pos,
//             source,
//             target_size: self.dim,
//             offset: self.offset,
//             alpha: self.alpha,
//         }
//     }
// }

// #[derive(Debug, Clone)]
// pub enum SpriteSource {
//     Static(i32, i32),
//     SimpleAnimation(u32, Vec<(i32, i32)>),
// }

// impl SpriteSource {
//     fn get_frame_pos (&self, runtime_ms: u128) -> (i32, i32) {
//         match self {
//             Self::Static(x, y) => (*x, *y),

//             Self::SimpleAnimation(durr_per_frame, frames) => {
//                 let total_animation_time = *durr_per_frame as usize * frames.len();
//                 // let loops = runtime_ms / total_animation_time as u128;
//                 let remaining = runtime_ms as usize % total_animation_time;
//                 let frame_idx = remaining / *durr_per_frame as usize;

//                 frames[frame_idx]
//             }
//         }
//     }
// }

// pub type TextureMap = HashMap<String, SpriteConfig>;
