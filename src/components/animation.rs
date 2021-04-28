use std::time::Duration;
use std::time::Instant;

use specs::prelude::*;

use crate::components::*;
use crate::core::*;

/// The current map position (tile) of a given entity
#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct MovementAnimation {
    pub start: Instant,
    pub duration: Duration,
    pub loops: u8,
    // pub timing: AnimationTiming,
    pub from: WorldPos,
    pub to: WorldPos,
}

// impl MovementAnimation {
//     pub fn new(from: WorldPos, to: WorldPos, t: Duration) -> Self {
//         debug_assert!(t.as_nanos() > 0); // a zero length animation makes no sense
//         let start = Instant::now();
//         Self { start, duration: t, loops: 1, from, to, }
//     }

//     pub fn start(self, start: Instant) -> Self {
//         Self { start, ..self }
//     }

//     pub fn loops(self, loops: u8) -> Self {
//         Self { loops, ..self }
//     }
// }

// #[derive(Debug)]
// pub enum Easing {
//     Linear,
//     BounceBack,
// }

pub struct Animation;

impl<'a> System<'a> for Animation {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, Position>,
        ReadStorage<'a, MovementAnimation>,
        Read<'a, LazyUpdate>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut positions, animations, updater) = data;
        let now = Instant::now();

        for (anim, e, pos) in (&animations, &entities, &mut positions).join() {
            if anim.start > now {
                // animation not started yet -> skip it
                continue;
            }

            let delta: Duration = now - anim.start;
            if anim.loops > 0 && delta > anim.duration * anim.loops as u32 {
                // animation completed -> remove component
                pos.0 = anim.to;
                updater.remove::<MovementAnimation>(e);
                continue;
            }

            pos.0 = animate(delta, anim);
        }
    }
}

fn animate(delta: Duration, anim: &MovementAnimation) -> WorldPos {
    let MovementAnimation {
        duration,
        /*timing,*/ from,
        to,
        ..
    } = anim;
    let dt = (delta.as_nanos() % duration.as_nanos()) as f32 / duration.as_nanos() as f32;

    // match timing {
    // AnimationTiming::Linear => {
    let dx = dt * (to.0 - from.0);
    let dy = dt * (to.1 - from.1);

    WorldPos(from.0 + dx, from.1 + dy)
    // }
    // }
}

// #[derive(Component, Debug)]
// #[storage(VecStorage)]
// pub struct VisualCmp(Vec<Visual>);

// #[derive(Debug)]
// pub enum Visual {
//     Static(String, u32),
// }

// pub struct Visual2SpriteSystem;

// impl<'a> System<'a> for Visual2SpriteSystem {
//     type SystemData = (
//         Entities<'a>,
//         WriteStorage<'a, Sprites>,
//         ReadStorage<'a, VisualCmp>,
//         Read<'a, LazyUpdate>,
//     );

//     fn run(&mut self, data: Self::SystemData) {
//         let (entities, mut sprites, visuals, updater) = data;

//         for (vs, e) in (&visuals, &entities).join() {
//             for v in &vs.0 {
//                 match v {
//                     Visual::Static(tex_name, idx) => {
//                         if let Some(mut sprite) = sprites.get_mut(e) {
//                         }
//                     }
//                 }
//             }
//         }
//     }
// }

// #[derive(Component, Debug)]
// #[storage(VecStorage)]
// pub struct SpriteSheetAnimation {
//     pub start: Instant,
//     pub duration: Duration,
//     pub loops: u8,
//     pub sprites: Vec<Sprite>,
//     pub index: usize,
//     pub active_sprite: usize,
// }

// pub struct SpriteSheetAnimationSystem;

// impl<'a> System<'a> for SpriteSheetAnimationSystem {
//     type SystemData = (
//         Entities<'a>,
//         WriteStorage<'a, Sprites>,
//         ReadStorage<'a, SpriteSheetAnimation>,
//         Read<'a, LazyUpdate>,
//     );

//     fn run(&mut self, data: Self::SystemData) {
//         let (entities, mut positions, animations, updater) = data;
//         let now = Instant::now();

//         for (anim, e, pos) in (&animations, &entities, &mut positions).join() {
//             if anim.start > now {
//                 // animation not started yet -> skip it
//                 continue;
//             }

//             let delta: Duration = now - anim.start;
//             if anim.loops > 0 && delta > anim.duration * anim.loops as u32 {
//                 // animation completed -> remove component
//                 updater.remove::<MovementAnimation>(e);
//                 continue;
//             }

//             // pos.0 = animate(delta, anim);
//         }
//     }
// }

// fn from_tiles(num: u16) -> Sprite {
//     let x = (num as i32 % 64) * 32;
//     let y = (num as i32 / 64) * 32;

//     Sprite {
//         texture: "tiles".to_string(),
//         region: (x, y, 32, 32),
//         offset: (0, -32),
//     }
// }

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct Visuals(pub Vec<Visual>);

#[derive(Debug)]
pub enum Visual {
    Static {
        texture: String,
        index: u32,
        offset: (i32, i32),
    },

    Animation {
        texture: String,
        sprites: Vec<u32>,
        offset: (i32, i32),
        ms_per_frame: u32,
    },
}

impl Visual {
    pub fn new_static(texture: impl ToString, index: u32, offset_x: i32, offset_y: i32) -> Self {
        Self::Static {
            texture: texture.to_string(),
            index,
            offset: (offset_x, offset_y),
        }
    }
}

pub struct Visuals2SpriteIter<'a> {
    index: usize,
    visuals: &'a Visuals,
}

impl<'a> Iterator for Visuals2SpriteIter<'a> {
    type Item = Sprite;

    fn next(&mut self) -> Option<Sprite> {
        let idx = self.index;

        if idx >= self.visuals.0.len() {
            return None;
        }

        self.index = idx + 1;

        match &self.visuals.0[idx] {
            Visual::Static {
                texture,
                index,
                offset,
            } => {
                let x = (*index as i32 % 64) * 32;
                let y = (*index as i32 / 64) * 32;

                Some(Sprite {
                    texture: texture.clone(),
                    region: (x, y, 32, 32),
                    offset: *offset,
                })
            }

            Visual::Animation { .. } => None,
        }
    }
}
