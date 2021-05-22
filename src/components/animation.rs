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
