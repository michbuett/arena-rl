use std::time::Duration;
use std::time::Instant;
use std::cmp::{max, min};

use specs::prelude::*;

use crate::ui::ScreenCoord;
use crate::components::*;
use crate::core::*;

/// The current map position (tile) of a given entity
#[derive(Component, Debug, Clone)]
#[storage(VecStorage)]
pub struct MovementAnimation {
    start: Instant,
    duration: Duration,
    steps: Vec<WorldPos>,
    modification: MovementModification,
    // pub loops: u8,
    // pub timing: AnimationTiming,
    // pub start_pos: WorldPos,
    // pub from: WorldPos,
    // pub to: WorldPos,
}

impl MovementAnimation {
    pub fn new(duration: Duration, steps: Vec<WorldPos>) -> Self {
        Self {
            start: Instant::now(),
            duration,
            steps,
            modification: MovementModification::None,
        }
    }

    // pub fn set_start(self, new_start: Instant) -> Self {
    //     Self { start: new_start, ..self }
    // }

    pub fn set_modification(self, modification: MovementModification) -> Self {
        Self { modification, ..self }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum MovementModification {
    None,
    ParabolaJump(u32),
}

// #[derive(Debug)]
// pub enum Easing {
//     Linear,
//     BounceBack,
// }

pub struct MovementAnimationSystem;

impl<'a> System<'a> for MovementAnimationSystem {
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
            // println!("animate pos {:?}", pos.0);
        }
    }
}

fn animate(delta: Duration, anim: &MovementAnimation) -> WorldPos {
    let MovementAnimation {
        duration,
        steps,
        modification,
        ..
    } = anim;
    let step_dur: u128 = duration.as_nanos();
    let step_idx: usize = (delta.as_nanos() / step_dur) as usize;

    if step_idx >= steps.len() - 1 {
        return *steps.last().unwrap();
    }

    let step_delta = delta.as_nanos() - step_idx as u128 * step_dur;
    let dt = (step_delta % step_dur) as f32 / step_dur as f32;
    let from: WorldPos = steps[step_idx];
    let to: WorldPos = steps[step_idx + 1];
    let dx = dt * (to.x() - from.x());
    let dy = dt * (to.y() - from.y());
    let target_pos = from.translate_xy(dx, dy);

    match modification {
        MovementModification::None => target_pos,
        MovementModification::ParabolaJump(max_height) => parabola_jump(
            ScreenCoord::from_world_pos(target_pos),
            ScreenCoord::from_world_pos(from),
            ScreenCoord::from_world_pos(to),
            *max_height as f32,
        )
        .to_world_pos(),
    }
}

fn parabola_jump(target: ScreenCoord, start: ScreenCoord, end: ScreenCoord, max_height: f32) -> ScreenCoord {
    let l = start.euclidian_distance(end); // the total distance
    let li = start.euclidian_distance(target); // the actual distance for the current animation step
    let hl = l / 2.0; // the half of the total distance; this is where the dy is maxed
    let damper = max_height / (hl * hl); // a dampening factor which ensures that dy <= max_height
    let dz = -damper * (hl * hl - (hl - li) * (hl - li));

    target.translate(0, 0, dz.round() as i32)
}

#[derive(Component, Debug, Clone)]
#[storage(VecStorage)]
pub struct FadeAnimation {
    start: Instant,
    duration: Duration,
    start_alpha: u8,
    end_alpha: u8,
}

impl FadeAnimation {
    pub fn fadeout_after(duration: Duration) -> Self {
        Self {
            start: Instant::now(),
            duration,
            start_alpha: 255,
            end_alpha: 0,
        }
    }
}
pub struct FadeAnimationSystem;

impl<'a> System<'a> for FadeAnimationSystem {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, FadeAnimation>,
        WriteStorage<'a, Text>,
        Read<'a, LazyUpdate>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, animations, mut texts, updater) = data;
        let now = Instant::now();

        for (anim, text, e) in (&animations, &mut texts, &entities).join() {
            if anim.start > now {
                // animation not started yet -> skip it
                continue;
            }

            let delta: Duration = now - anim.start;
            if delta > anim.duration {
                // animation completed -> remove component
                updater.remove::<FadeAnimation>(e);
                continue;
            }

            let dt = delta.as_millis() as f32 / anim.duration.as_millis() as f32;
            let da = dt * (anim.end_alpha as f32 - anim.start_alpha as f32);
            let new_alpha = anim.start_alpha as i32 + da.round() as i32;

            text.alpha = max(0, min( 255, new_alpha)) as u8;
        }
    }
}
