use std::time::Duration;
use std::time::Instant;

use specs::prelude::*;

// use crate::ui::ScreenPos;
use crate::components::*;
use crate::core::*;

/// The current map position (tile) of a given entity
#[derive(Component, Debug, Clone)]
#[storage(VecStorage)]
pub struct MovementAnimation {
    pub start: Instant,
    pub duration: Duration,
    // pub loops: u8,
    // pub timing: AnimationTiming,
    pub steps: Vec<WorldPos>,
    // pub start_pos: WorldPos,
    // pub from: WorldPos,
    // pub to: WorldPos,
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
            if delta > (anim.duration) * anim.steps.len() as u32 {
            // if anim.loops > 0 && delta > anim.duration * anim.loops as u32 {
                // animation completed -> remove component
                pos.0 = *anim.steps.last().unwrap();
                // pos.0 = anim.to;
                updater.remove::<MovementAnimation>(e);
                continue;
            }

            pos.0 = animate(delta, anim);
        }
    }
}

fn animate(delta: Duration, anim: &MovementAnimation) -> WorldPos {
    // let MovementAnimation { duration, from, to, ..} = anim;
    let MovementAnimation { duration, steps, ..} = anim;
    let step_dur: u128 = duration.as_nanos();
    let step_idx: usize = (delta.as_nanos() / step_dur) as usize;

    if step_idx >= steps.len() - 1 {
        return *steps.last().unwrap();
    }

    let step_delta = delta.as_nanos() - step_idx as u128 * step_dur;
    let dt = (step_delta % step_dur) as f32 / step_dur as f32;
    // println!("{:?}", anim);
    // println!("step={}, durration per step={}, dt={}, delta={:?}", step_idx, step_dur, dt, delta);
    
    // let dt = (delta.as_nanos() % duration.as_nanos()) as f32 / duration.as_nanos() as f32;
    let from: WorldPos = steps[step_idx];
    let to: WorldPos = steps[step_idx + 1];

    // match timing {
    // AnimationTiming::Linear => {
    let dx = dt * (to.0 - from.0);
    let dy = dt * (to.1 - from.1);

    WorldPos(from.0 + dx, from.1 + dy)
    // }
    // }
}
