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
    /// - Duration: the duration of the complete action
    /// - MovementModification: modification of the movement (e.g. add jump effect)
    MoveTo(Entity, Vec<WorldPos>, Duration, MovementModification),

    Sprite(String, WorldPos, u64),
}

impl Fx {
    pub fn move_to(
        e: Entity,
        p: Vec<WorldPos>,
        delay: u64,
        dur_ms: u64,
        m: MovementModification,
    ) -> Self {
        let d = Duration::from_millis(dur_ms);
        Fx(start_after(delay), FxEffect::MoveTo(e, p, d, m))
    }

    pub fn sprite(s: String, p: WorldPos, delay: u64, dur_ms: u64) -> Self {
        Fx(start_after(delay), FxEffect::Sprite(s, p, dur_ms))
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
        Read<'a, TextureMap>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, fx, updater, texture_map) = data;
        let now = Instant::now();

        for (e, Fx(start_time, fx_eff)) in (&entities, &fx).join() {
            if now < *start_time {
                continue;
            }

            match fx_eff {
                FxEffect::Text(txt, pos, dur) => {
                    updater
                        .create_entity(&entities)
                        .with(
                            Text::new(txt.to_string(), FontFace::VeryBig)
                                .padding(5)
                                // .background(252, 251, 250, 155)
                                .color(210, 31, 42, 255),
                        )
                        .with(Position(*pos))
                        .with(MovementAnimation::new(
                            Duration::from_millis(250),
                            vec![*pos, animation_target_pos(pos)],
                        ))
                        .with(FadeAnimation::fadeout_after_ms(*dur))
                        .with(EndOfLive::after_ms(*dur))
                        .build();
                }

                FxEffect::Sprite(sprite, pos, duration) => {
                    handle_sprite(sprite, *pos, *duration, &entities, &updater, &texture_map);
                }

                FxEffect::MoveTo(entity, path, duration, modification) => {
                    handle_move_to(*entity, path.to_vec(), *duration, *modification, &updater);
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
    modification: MovementModification,
    updater: &Read<LazyUpdate>,
) {
    updater.insert(
        target_entity,
        MovementAnimation::new(duration, path).set_modification(modification),
    );
}

fn animation_target_pos(wp: &WorldPos) -> WorldPos {
    extern crate rand;
    use rand::prelude::*;

    let mut rng = rand::thread_rng();
    let range_x = rand::distributions::Uniform::from(-100..=100);
    let range_y = rand::distributions::Uniform::from(-150..=-100);
    let ScreenPos(sx, sy) = ScreenPos::from_world_pos(*wp, (0, 0));
    let (dx, dy) = (rng.sample(range_x), rng.sample(range_y));

    ScreenPos(sx + dx, sy + dy).to_world_pos((0, 0))
}

fn handle_sprite(
    sprite_name: &str,
    pos: WorldPos,
    duration: u64,
    entities: &Entities,
    updater: &Read<LazyUpdate>,
    texture_map: &Read<TextureMap>,
) {
    if let Some(sprite) = texture_map.get(sprite_name) {
        updater
            .create_entity(&entities)
            .with(Sprites::new(vec![sprite.clone()]))
            .with(Position(pos))
            .with(EndOfLive::after_ms(duration))
            .with(ZLayerFX)
            .build();
    }
}

fn start_after(ms: u64) -> Instant {
    Instant::now() + Duration::from_millis(ms)
}
