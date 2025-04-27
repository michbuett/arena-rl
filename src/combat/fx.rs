use std::time::Duration;

use bevy::prelude::*;
use rand::prelude::Distribution;

use crate::{
    animations::{FadeAnimation, MovementAnimation, MovementModification},
    assets::Visual,
    EndOfLive,
};

use super::{
    map::MapPos,
    ui::{UiState, UiStateTransitionedEvent, Z_LAYER_ACTOR, Z_LAYER_FLOOR},
};

const MOVE_STEP_DURATION: u64 = 200;
const BLOOD_SPLATTER_DURATION: u64 = 1000;

#[derive(Debug)]
pub struct FxSequence(Duration, Vec<(Duration, FxEffect)>);

impl FxSequence {
    pub fn new() -> Self {
        Self(Duration::from_millis(0), vec![])
    }

    pub fn wait_until_finished(mut self) -> Self {
        if let Some(d) = self
            .1
            .iter()
            .map(|(wait, eff)| *wait + eff.duration())
            .max()
        {
            self.0 = d;
        }

        self
    }

    pub fn wait(mut self, ms: u64) -> Self {
        self.0 += Duration::from_millis(ms);
        self
    }

    pub fn then(mut self, fx: FxEffect) -> Self {
        self.1.push((self.0, fx));
        self
    }

    pub fn then_insert(mut self, mut other: FxSequence) -> Self {
        for (wait, eff) in other.1.drain(..) {
            self.1.push((self.0 + wait, eff));
        }
        self
    }

    pub fn then_append(self, other: FxSequence) -> Self {
        let other_wait = other.0;
        let mut result = self.then_insert(other);
        result.0 += other_wait;
        result
    }

    pub fn run(mut self, commands: &mut Commands) {
        for (duration, effect) in self.1.drain(..) {
            commands.spawn(Fx {
                timer: Timer::new(duration, TimerMode::Once),
                effect,
            });
        }

        commands.trigger(UiStateTransitionedEvent(UiState::wait(
            self.0.as_millis() as u64
        )));
    }
}

#[derive(Debug, Component)]
pub struct Fx {
    timer: Timer,
    effect: FxEffect,
}

pub fn update_check_fx_ready(
    time: Res<Time>,
    mut fx_q: Query<(Entity, &mut Fx)>,
    mut commands: Commands,
) {
    for (entity, mut fx) in fx_q.iter_mut() {
        fx.timer.tick(time.delta());

        if fx.timer.finished() {
            // Timed effect is ready
            // => trigger the effect by adding/updating the neccesary components
            match &fx.effect {
                FxEffect::Remove(entity) => {
                    commands.entity(*entity).despawn_recursive();
                }

                FxEffect::BloodSplatter(pos) => {
                    handle_blood_splatter(&mut commands, *pos);
                }

                FxEffect::Stains {
                    pos,
                    visual,
                    fade_out_after,
                } => {
                    handle_stains(&mut commands, *pos, visual.clone(), *fade_out_after);
                }

                FxEffect::MoveTo {
                    entity,
                    path,
                    movement_modification,
                    step_durration,
                } => {
                    commands.entity(*entity).insert(
                        MovementAnimation::new(
                            Duration::from_millis(*step_durration),
                            path.clone(),
                        )
                        .set_modification(*movement_modification),
                    );
                }
            }

            // Effect is triggered
            // => entity can be removed
            commands.entity(entity).despawn_recursive();
        }
    }
}

#[derive(Debug)]
pub enum FxEffect {
    // Update(Actor),
    Remove(Entity),

    // /// - ID: the target game object
    // /// - Vec<WorldPos>: the path the entity should move along
    // /// - MovementModification: modification of the movement (e.g. add jump effect)
    // MoveTo(ID, Vec<WorldPos>, MovementModification, u64),
    MoveTo {
        entity: Entity,
        path: Vec<Vec3>,
        movement_modification: MovementModification,
        step_durration: u64,
    },

    BloodSplatter(Vec3),

    Stains {
        pos: Vec3,
        visual: Visual,
        fade_out_after: u64,
    },
    // Custom {
    //     pos: WorldPos,
    //     duration: u64,
    //     sprite: Option<String>,
    //     text: Option<Text>,
    //     scale_anim: Option<(f32, f32)>,
    //     movement_anim: Option<Vec<WorldPos>>,
    //     fade_anim: bool,
    // },
}

impl FxEffect {
    pub fn walk_along(actor: Entity, path: &Vec<MapPos>) -> Self {
        let path = path
            .iter()
            .map(|mpos| mpos.into_vec3().with_z(Z_LAYER_ACTOR))
            .collect();

        Self::MoveTo {
            entity: actor,
            path,
            movement_modification: MovementModification::ParabolaJump(50),
            step_durration: MOVE_STEP_DURATION,
        }
    }

    pub fn duration(&self) -> Duration {
        let millis = match self {
            FxEffect::BloodSplatter(..) => BLOOD_SPLATTER_DURATION,

            FxEffect::Stains {
                fade_out_after: fade_out,
                ..
            } => *fade_out,

            FxEffect::MoveTo {
                path,
                step_durration,
                ..
            } => path.len().checked_sub(1).unwrap_or(0) as u64 * step_durration,
            //
            // FxEffect::Custom { duration, .. } => *duration,
            _ => 0,
        };

        Duration::from_millis(millis)
    }
}

fn handle_blood_splatter(commands: &mut Commands, pos: Vec3) {
    let duration = Duration::from_millis(BLOOD_SPLATTER_DURATION);
    let num_particals = 10;

    for i in 1..=num_particals {
        let visual = Visual::Single(format!("blood-splatter-{}", (i % 3) + 1));
        let move_steps = animation_path(pos, (50, 100));
        let stains_pos = move_steps.last().unwrap().with_z(Z_LAYER_FLOOR);
        let mv_dur =
            Duration::from_millis(between(400, (BLOOD_SPLATTER_DURATION - 100) as i32) as u64);
        let eol_dur = duration + Duration::from_millis(100); // a little bit of extra time to avoid stuttering

        commands.spawn((
            EndOfLive::after(eol_dur),
            Name::new("blood-splatter"),
            SpatialBundle {
                transform: Transform::from_translation(pos),
                ..Default::default()
            },
            visual.clone(),
            MovementAnimation::new(mv_dur, move_steps)
                .set_modification(MovementModification::ParabolaJump(100)),
        ));

        commands.spawn(Fx {
            timer: Timer::new(duration, TimerMode::Once),
            effect: FxEffect::Stains {
                pos: stains_pos,
                visual,
                fade_out_after: 60 * 1000,
            },
        });
    }
}

fn handle_stains(commands: &mut Commands, pos: Vec3, visual: Visual, fade_out_after: u64) {
    let duration = Duration::from_millis(fade_out_after);

    commands.spawn((
        Name::new("stains"),
        EndOfLive::after(duration),
        SpatialBundle {
            transform: Transform::from_translation(pos),
            ..Default::default()
        },
        visual,
        FadeAnimation::fade_out(duration),
    ));
}

fn animation_path(source_pos: Vec3, length: (u32, u32)) -> Vec<Vec3> {
    let mut rng = rand::thread_rng();
    let distribution_dir = rand::distributions::Uniform::from(-100..=100);
    let dir = Vec2::new(
        distribution_dir.sample(&mut rng) as f32,
        distribution_dir.sample(&mut rng) as f32,
    )
    .normalize();

    let distribution_length = rand::distributions::Uniform::from(length.0..=length.1);
    let length = distribution_length.sample(&mut rng) as f32;
    let target_pos = source_pos + (dir * length).extend(0.0);

    vec![source_pos, target_pos]

    // let dir = Vec2::new(1.0, 1.0);
    // let factor = 1000.0;
    // let dx_range = (
    //     (dir.x * factor * length.0 as f32).round() as u32,
    //     (dir.x * factor * length.1 as f32).round() as u32,
    // );
    // let dy_range = (
    //     (dir.y * factor * length.0 as f32).round() as u32,
    //     (dir.y * factor * length.1 as f32).round() as u32,
    // );

    // let distribution_x = rand::distributions::Uniform::from(dx_range.0..=dx_range.1);
    // let distribution_y = rand::distributions::Uniform::from(dy_range.0..=dy_range.1);
    // let dx = distribution_x.sample(&mut rng) as f32 / factor;
    // let dy = distribution_y.sample(&mut rng) as f32 / factor;

    // vec![source_pos, source_pos + Vec3::new(dx, dy, 0.0)]
}

fn between(num1: i32, num2: i32) -> i32 {
    let mut rng = rand::thread_rng();
    let distribution = rand::distributions::Uniform::from(num1..=num2);
    distribution.sample(&mut rng)
}
