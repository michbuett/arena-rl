use specs::prelude::*;
use specs_derive::Component;

use crate::components::*;
use crate::core::*;
use crate::ui::ScreenCoord;

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct Fx(Instant, Duration, FxEffect);

#[derive(Debug)]
pub enum FxEffect {
    Text(Text, WorldPos),

    /// - Entity: the target entity (the entity which should move)
    /// - Vec<WorldPos>: the path the entity should move along
    /// - MovementModification: modification of the movement (e.g. add jump effect)
    MoveTo(Entity, Vec<WorldPos>, MovementModification),

    Sprite(String, WorldPos),
}

impl Fx {
    pub fn from_combat_event(ev: CombatEventFx, delay: u64) -> Self {
        match ev {
            CombatEventFx::Text(txt, pos, duration) =>
                Self::say(txt, pos, delay, duration),

            CombatEventFx::Scream(txt, pos, duration) =>
                Self::scream(txt, pos, delay, duration),
                        
            CombatEventFx::Sprite(s, pos, duration) =>
                Self::sprite(s, pos, delay, duration),
        }
    }

    pub fn move_to(
        e: Entity,
        p: Vec<WorldPos>,
        delay: u64,
        dur_ms: u64,
        m: MovementModification,
    ) -> Self {
        Fx(start_after(delay), Duration::from_millis(dur_ms), FxEffect::MoveTo(e, p, m))
    }

    pub fn sprite(s: String, p: WorldPos, delay: u64, dur_ms: u64) -> Self {
        Fx(start_after(delay), Duration::from_millis(dur_ms), FxEffect::Sprite(s, p))
    }

    pub fn say(txt: DisplayStr, pos: WorldPos, delay: u64, dur_ms: u64) -> Self {
        let txt = Text::new(txt.to_string(), FontFace::Big).padding(5).color(21, 22, 23, 255);
        Fx(start_after(delay), Duration::from_millis(dur_ms), FxEffect::Text(txt, pos))
    }

    pub fn scream(txt: DisplayStr, pos: WorldPos, delay: u64, dur_ms: u64) -> Self {
        let txt = Text::new(txt.to_string(), FontFace::VeryBig).padding(5).color(195, 31, 42, 255);
        Fx(start_after(delay), Duration::from_millis(dur_ms), FxEffect::Text(txt, pos))
    }

    pub fn duration_ms(&self) -> u64 {
        self.1.as_millis() as u64
    }

    pub fn ends_at(&self) -> Instant {
        self.0 + self.1
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

        for (e, Fx(start_time, duration, fx_eff)) in (&entities, &fx).join() {
            if now < *start_time {
                continue;
            }

            match fx_eff {
                FxEffect::Text(txt, pos) => 
                    handle_text(&entities, txt.clone(), *pos, *duration, &updater),

                FxEffect::Sprite(sprite, pos) => 
                    handle_sprite(sprite, *pos, *duration, &entities, &updater, &texture_map),

                FxEffect::MoveTo(entity, path, modification) =>
                    handle_move_to(*entity, path.to_vec(), *duration, *modification, &updater),
            }

            let _ = entities.delete(e);
        }
    }
}

fn handle_text(
    entities: &Entities,
    txt: Text,
    pos: WorldPos,
    duration: Duration,
    updater: &Read<LazyUpdate>,
) {
    updater
        .create_entity(entities)
        .with(txt)
        .with(Position(pos))
        .with(MovementAnimation::new(duration, vec![pos, animation_target_pos(&pos)]))
        .with(FadeAnimation::fadeout_after(duration))
        .with(EndOfLive::after(duration))
        .build();
}


fn handle_move_to(
    target_entity: Entity,
    path: Vec<WorldPos>,
    duration: Duration,
    modification: MovementModification,
    updater: &Read<LazyUpdate>,
) {
    assert!(path.len() > 1);
    
    let num_steps = path.len() as u64 - 1;
    let time_per_step = Duration::from_millis(duration.as_millis() as u64 / num_steps);
        
    updater.insert(
        target_entity,
        MovementAnimation::new(time_per_step, path).set_modification(modification),
    );
}

fn animation_target_pos(wp: &WorldPos) -> WorldPos {
    extern crate rand;
    use rand::prelude::*;

    let mut rng = rand::thread_rng();
    let range_x = rand::distributions::Uniform::from(-100..=100);
    let range_y = rand::distributions::Uniform::from(-150..=-100);
    let (dx, dy) = (rng.sample(range_x), rng.sample(range_y));

    ScreenCoord::translate_world_pos(*wp, dx, dy)
}

fn handle_sprite(
    sprite_name: &str,
    pos: WorldPos,
    duration: Duration,
    entities: &Entities,
    updater: &Read<LazyUpdate>,
    texture_map: &Read<TextureMap>,
) {
    if let Some(sprite) = texture_map.get(sprite_name) {
        updater
            .create_entity(&entities)
            .with(Sprites::new(vec![sprite.clone()]))
            .with(Position(pos))
            .with(EndOfLive::after(duration))
            .with(ZLayerFX)
            .build();
    }
}

fn start_after(ms: u64) -> Instant {
    Instant::now() + Duration::from_millis(ms)
}
