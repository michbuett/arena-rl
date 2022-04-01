use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct SpriteConfig {
    pub source: SpriteSource,
    pub offset: (i32, i32),
    pub dim: (u32, u32),
    pub alpha: u8,
    pub scale: f32,
}

#[derive(Debug, Clone)]
pub struct Sprite {
    /// To allow tweening an animated sprite is printed as two images one for each keyframe
    /// The in-between-effect is created by making each keyframe transparent to the degree
    /// how close the sample is to a keyframe switch
    /// So the first two values define the position of the keyframe image in the sprite sheet
    /// and the last value is the degree of transition (0 -> 1) which can directly interpeted
    /// as an alpha value
    pub source: ((i32, i32), Option<(f64, i32, i32)>, Option<(f64, i32, i32)>),
    pub offset: (i32, i32),
    pub dim: (u32, u32),
    pub alpha: u8,
    pub scale: f32,
}

impl SpriteConfig {
    pub fn sample(&self, runtime_ms: u128) -> Sprite {
        Sprite {
            source: self.source.get_frame_pos(runtime_ms),
            offset: self.offset,
            dim: self.dim,
            alpha: self.alpha,
            scale: self.scale,
        }
    }
}

#[derive(Debug, Clone)]
pub enum SpriteSource {
    Static(i32, i32),
    SimpleAnimation(u32, Vec<(i32, i32)>),
}

impl SpriteSource {
    fn get_frame_pos(&self, runtime_ms: u128) -> ((i32, i32), Option<(f64, i32, i32)>, Option<(f64, i32, i32)>) {
        match self {
            Self::Static(x, y) => ((*x, *y), None, None),

            Self::SimpleAnimation(durr_per_frame, frames) => {
                let total_animation_time = *durr_per_frame as usize * frames.len();
                let remaining = runtime_ms as f64 % total_animation_time as f64;
                let frame_idx = remaining as f64 / *durr_per_frame as f64;
                let idx = frame_idx.floor() as usize;
                let idx_prev= if idx > 0 { idx - 1 } else { frames.len() - 1 };
                let idx_next = if idx < frames.len() - 1 { idx + 1 } else { 0 };
                let transition = frame_idx - frame_idx.floor();
                let (x_prev, y_prev) = frames[idx_prev];
                let (x_next, y_next) = frames[idx_next];

                (
                    frames[idx],
                    Some((0.6 * qube(1.0 - transition), x_prev, y_prev)),
                    Some((0.6 * qube(transition), x_next, y_next)),
                )
            }
        }
    }
}

fn qube(x: f64) -> f64 {
    x * x * x
}

pub type TextureMap = HashMap<String, SpriteConfig>;
