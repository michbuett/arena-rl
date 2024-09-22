use bevy::prelude::*;
use std::time::Duration;

pub fn animation_plugin(app: &mut App) {
    app.add_systems(Update, (update_sprite_animation, update_movement_animation));
}

#[derive(Debug, Component)]
pub struct SpriteAnimation {
    pub indices: Vec<usize>,
    pub current_idx: usize,
    pub timer: Timer,
}

fn update_sprite_animation(
    mut animations: Query<(&mut TextureAtlas, &mut SpriteAnimation)>,
    time: Res<Time>,
) {
    for (mut atlas, mut anim) in animations.iter_mut() {
        anim.timer.tick(time.delta());

        if anim.timer.finished() {
            if anim.current_idx < anim.indices.len() - 1 {
                anim.current_idx += 1;
            } else {
                anim.current_idx = 0;
            }

            atlas.index = anim.indices[anim.current_idx];
        }
    }
}

#[derive(Component, Debug, Clone)]
pub struct MovementAnimation {
    timer: Timer,
    steps: Vec<Vec3>,
    step_duration: Duration,
    modification: MovementModification,
}

impl MovementAnimation {
    pub fn new(duration: Duration, steps: Vec<Vec3>) -> Self {
        Self {
            timer: Timer::new(duration * steps.len() as u32, TimerMode::Once),
            step_duration: duration,
            steps,
            modification: MovementModification::None,
        }
    }

    pub fn set_modification(self, modification: MovementModification) -> Self {
        Self {
            modification,
            ..self
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum MovementModification {
    None,
    ParabolaJump(u32),
}

fn update_movement_animation(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, Mut<MovementAnimation>, Mut<Transform>)>,
) {
    for (e, mut anim, mut transform) in query.iter_mut() {
        anim.timer.tick(time.delta());

        if anim.timer.finished() {
            // animation completed -> remove component
            transform.translation = *anim.steps.last().unwrap();
            commands.entity(e).remove::<MovementAnimation>();
            continue;
        }

        let delta = anim.timer.elapsed();
        transform.translation = animate_move(delta, &anim);
    }
}

fn animate_move(delta: Duration, anim: &MovementAnimation) -> Vec3 {
    let MovementAnimation {
        step_duration,
        steps,
        modification,
        ..
    } = anim;

    let step_dur: u128 = step_duration.as_nanos();
    let step_idx: usize = (delta.as_nanos() / step_dur) as usize;

    if step_idx >= steps.len() - 1 {
        return *steps.last().unwrap();
    }

    let step_delta = delta.as_nanos() - step_idx as u128 * step_dur;
    let dt = (step_delta % step_dur) as f32 / step_dur as f32;
    let from = steps[step_idx];
    let to = steps[step_idx + 1];
    let dx = dt * (to.x - from.x);
    let dy = dt * (to.y - from.y);
    let target_pos = from + Vec3::new(dx, dy, 0.0);

    match modification {
        MovementModification::None => target_pos,
        MovementModification::ParabolaJump(max_height) => {
            parabola_jump(target_pos, from, to, *max_height as f32)
        }
    }
}

fn parabola_jump(target: Vec3, start: Vec3, end: Vec3, max_height: f32) -> Vec3 {
    let l = (end - start).length(); // the total distance
    let li = (target - start).length(); // the actual distance for the current animation step
    let hl = l / 2.0; // the half of the total distance; this is where the dy is maxed
    let damper = max_height / (hl * hl); // a dampening factor which ensures that dy <= max_height
    let dy = damper * (hl * hl - (hl - li) * (hl - li));

    target + Vec3::new(0.0, dy, 0.0)
}
