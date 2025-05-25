use std::time::Duration;

use bevy::prelude::*;
use rand::prelude::Distribution;

use crate::{
    animations::{FadeAnimation, MovementAnimation, MovementModification, ScaleAnimation},
    assets::Visual,
    style::TextStyle,
    EndOfLive, MarkedForDeath,
};

use super::{
    map::MapPos,
    ui::{UiState, UiStateTransitionedEvent, Z_LAYER_ACTOR, Z_LAYER_FLOOR, Z_LAYER_VFX},
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

    // pub fn then_insert(mut self, mut other: FxSequence) -> Self {
    //     for (wait, eff) in other.1.drain(..) {
    //         self.1.push((self.0 + wait, eff));
    //     }
    //     self
    // }

    // pub fn then_append(self, other: FxSequence) -> Self {
    //     let other_wait = other.0;
    //     let mut result = self.then_insert(other);
    //     result.0 += other_wait;
    //     result
    // }

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
                    commands.entity(*entity).insert(MarkedForDeath);
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

                FxEffect::CustomText {
                    pos,
                    duration,
                    text,
                    movement_anim,
                } => {
                    handle_custom_text(
                        &mut commands,
                        *pos,
                        *duration,
                        text,
                        movement_anim.as_ref(),
                    );
                }
            }

            // Effect is triggered
            // => entity can be removed
            commands.entity(entity).insert(MarkedForDeath);
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

    CustomText {
        pos: Vec3,
        duration: u64,
        // sprite: Option<String>,
        text: (String, TextStyle),
        // scale_anim: Option<(f32, f32)>,
        movement_anim: Option<Vec<Vec3>>,
        // fade_anim: bool,
    },
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

    pub fn say(txt: impl Into<String>, pos: MapPos) -> Self {
        let pos = pos.into_vec3().with_z(Z_LAYER_VFX) + Vec3::new(0.0, 50.0, 0.0);

        Self::CustomText {
            pos,
            duration: 2000,
            text: (txt.into(), TextStyle::InGameScream),
            movement_anim: None,
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

            FxEffect::CustomText { duration, .. } => *duration,

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
            Transform::from_translation(pos),
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
        Transform::from_translation(pos),
        visual,
        FadeAnimation::fade_out(duration),
        ScaleAnimation::new(1.0, 3.0, duration),
    ));
}

fn handle_custom_text(
    commands: &mut Commands,
    pos: Vec3,
    duration: u64,
    (text, text_style): &(String, TextStyle),
    movement_anim: Option<&Vec<Vec3>>,
) {
    let duration = Duration::from_millis(duration);
    let mut text_entity = commands.spawn((
        Text2d(text.clone()),
        text_style.as_text_font(),
        text_style.as_text_color(),
        Transform::from_translation(pos),
        EndOfLive::after(duration),
        FadeAnimation::fade_out(duration),
        ScaleAnimation::new(1.0, 3.0, duration),
    ));

    if let Some(movement_anim) = movement_anim {
        text_entity.insert(MovementAnimation::new(duration, movement_anim.clone()));
    }
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
}

fn between(num1: i32, num2: i32) -> i32 {
    let mut rng = rand::thread_rng();
    let distribution = rand::distributions::Uniform::from(num1..=num2);
    distribution.sample(&mut rng)
}
