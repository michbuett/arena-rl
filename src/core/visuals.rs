use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct SpriteConfig {
    pub source: SpriteSource,
    pub offset: (i32, i32),
    pub dim: (u32, u32),
    pub alpha: u8,
}

#[derive(Debug, Clone)]
pub struct Sprite {
    pub source: (i32, i32),
    pub offset: (i32, i32),
    pub dim: (u32, u32),
    pub alpha: u8,
}

impl SpriteConfig {
    pub fn sample(&self, runtime_ms: u128) -> Sprite {
        Sprite {
            source: self.source.get_frame_pos(runtime_ms),
            offset: self.offset,
            dim: self.dim,
            alpha: self.alpha,
        }
    }
}

#[derive(Debug, Clone)]
pub enum SpriteSource {
    Static(i32, i32),
    SimpleAnimation(u32, Vec<(i32, i32)>),
}

impl SpriteSource {
    fn get_frame_pos (&self, runtime_ms: u128) -> (i32, i32) {
        match self {
            Self::Static(x, y) => (*x, *y),

            Self::SimpleAnimation(durr_per_frame, frames) => {
                let total_animation_time = *durr_per_frame as usize * frames.len();
                // let loops = runtime_ms / total_animation_time as u128;
                let remaining = runtime_ms as usize % total_animation_time;
                let frame_idx = remaining / *durr_per_frame as usize;

                frames[frame_idx]
            }
        }
    }
}

pub type TextureMap = HashMap<String, SpriteConfig>;
