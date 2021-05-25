use specs::prelude::*;
use specs_derive::Component;

use crate::components::*;
use crate::core::*;
use crate::ui::ScreenPos;

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct Fx(Instant, FxEffect);

#[derive(Debug)]
pub enum FxEffect {
    Text(String, WorldPos, u64),

    /// - Entity: the target entity (the entity which should move)
    /// - Vec<WorldPos>: the path the entity should move along
    /// - Duration: the duration of the complet action
    MoveTo(Entity, Vec<WorldPos>, Duration),
}

impl Fx {
    pub fn move_to(e: Entity, p: Vec<WorldPos>, delay: u64, dur_ms: u64) -> Self {
        let d = Duration::from_millis(dur_ms);
        
        Fx(start_after(delay), FxEffect::MoveTo(e, p, d))
    }

    pub fn text(txt: String, pos: &WorldPos, delay: u64) -> Self {
        Fx(start_after(delay), FxEffect::Text(txt, *pos, 500 + delay))
    }

    pub fn run(self, world: &World) {
        let (entities, updater): (Entities, Read<LazyUpdate>) = world.system_data();

        updater.create_entity(&entities).with(self).build();
    }
}

pub struct FxSystem;

impl<'a> System<'a> for FxSystem {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, Fx>,
        Read<'a, LazyUpdate>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, fx, updater) = data;
        let now = Instant::now();

        for (e, Fx(start_time, fx_eff)) in (&entities, &fx).join() {
            if now < *start_time {
                continue;
            }

            match fx_eff {
                FxEffect::Text(txt, pos, dur) => {
                    updater
                        .create_entity(&entities)
                        .with(Text::new(txt.to_string(), FontFace::VeryBig)
                              .padding(5)
                              .color(150, 21, 22, 255)
                              .background(252, 251, 250, 155))
                        .with(Position(*pos))
                        .with(MovementAnimation {
                            start: now,
                            duration: Duration::from_millis(250),
                            steps: vec![*pos, animation_target_pos(pos)],
                        })
                        .with(EndOfLive::after_ms(*dur))
                        .build();
                }

                FxEffect::MoveTo(entity, path, duration) => {
                    handle_move_to(*entity, path.to_vec(), *duration, &updater);
                }
            }

            let _ = entities.delete(e);
        }
    }
}

fn handle_move_to(
    target_entity: Entity,
    path: Vec<WorldPos>,
    duration: Duration,
    updater: &Read<LazyUpdate>,
) {
    updater.insert(
        target_entity,
        MovementAnimation {
            start: Instant::now(),
            duration: duration / (path.len() as u32 - 1),
            steps: path,
        },
    );
}

fn animation_target_pos(wp: &WorldPos) -> WorldPos {
    extern crate rand;
    use rand::prelude::*;

    let mut rng = rand::thread_rng();
    let range_x = rand::distributions::Uniform::from(-75..=75);
    let range_y = rand::distributions::Uniform::from(-100..=-25);
    let ScreenPos(sx, sy) = ScreenPos::from_world_pos(*wp, (0, 0));
    let (dx, dy) = (rng.sample(range_x), rng.sample(range_y));

    ScreenPos(sx + dx, sy + dy).to_world_pos((0, 0))
}

fn start_after(ms: u64) -> Instant {
    Instant::now() + Duration::from_millis(ms)
}
