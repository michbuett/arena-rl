extern crate rand;

use rand::prelude::*;

use specs::prelude::*;
use specs_derive::Component;

use crate::components::*;
use crate::core::*;
use crate::ui::ScreenCoord;

#[derive(Debug)]
pub struct FxSequence(Instant, Vec<Fx>);

impl FxSequence {
    pub fn new() -> Self {
        Self(Instant::now(), vec![])
    }

    pub fn wait_until_finished(mut self) -> Self {
        if let Some(Fx(_, dur, _)) = self.1.last() {
            self.0 += *dur
        }

        self
    }

    pub fn wait(mut self, ms: u64) -> Self {
        self.0 += Duration::from_millis(ms);
        self
    }

    pub fn then(mut self, fx: FxEffect) -> Self {
        self.1.push(Fx(self.0, duration(&fx), fx));
        self
    }

    pub fn into_vec(self) -> Vec<Fx> {
        self.1
    }
}


#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct Fx(Instant, Duration, FxEffect);

impl Fx {
    pub fn ends_at(&self) -> Instant {
        self.0 + self.1
    }

    pub fn run(self, world: &World) {
        let (entities, updater): (Entities, Read<LazyUpdate>) = world.system_data();

        updater.create_entity(&entities).with(self).build();
    }
}

#[derive(Debug)]
pub enum FxEffect {
    Text(Text, WorldPos),

    /// - Entity: the target entity (the entity which should move)
    /// - Vec<WorldPos>: the path the entity should move along
    /// - MovementModification: modification of the movement (e.g. add jump effect)
    MoveTo(Entity, Vec<WorldPos>, MovementModification),

    Sprite(String, WorldPos),

    BloodSplatter(WorldPos),

    Projectile(String, WorldPos, WorldPos),
}

impl FxEffect {
    pub fn say(txt: impl ToString, pos: WorldPos) -> FxEffect {
        let txt = Text::new(txt.to_string(), FontFace::Big).padding(5).color(21, 22, 23, 255);
        FxEffect::Text(txt, pos)
    }

    pub fn scream(txt: impl ToString, pos: WorldPos) -> FxEffect {
        let txt = Text::new(txt.to_string(), FontFace::VeryBig).padding(5).color(195, 31, 42, 255);
        FxEffect::Text(txt, pos)
    }

    pub fn jump(e: Entity, p: Vec<WorldPos>) -> Self {
        FxEffect::MoveTo(e, p, MovementModification::ParabolaJump(96))
    }

    pub fn sprite(s: impl ToString, p: WorldPos, _d: u64) -> Self {
        FxEffect::Sprite(s.to_string(), p)
    }
                            
    pub fn projectile(s: impl ToString, from: WorldPos, to: WorldPos) -> Self {
        FxEffect::Projectile(s.to_string(), from, to)
    }

    pub fn blood_splatter(p: WorldPos) -> Self {
        FxEffect::BloodSplatter(p)
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

                FxEffect::BloodSplatter(pos) => 
                    handle_blood_splatter(*pos, *duration, &entities, &updater, &texture_map),

                FxEffect::Projectile(sprite, from, to) => 
                    handle_projectile(sprite, *from, *to, *duration, &entities, &updater, &texture_map),

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
        .with(ScaleAnimation::new(1.0, 2.0, duration))
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

fn handle_blood_splatter(
    pos: WorldPos,
    duration: Duration,
    entities: &Entities,
    updater: &Read<LazyUpdate>,
    texture_map: &Read<TextureMap>,
) {
    for i in 1..=3 {
        let sprite = texture_map.get(&format!("blood-splatter-{}", i)).unwrap();
        let to = random_neighbor_pos(&pos);

        updater
            .create_entity(&entities)
            .with(Sprites::new(vec![sprite.clone()]))
            .with(Position(pos))
            .with(MovementAnimation::new(duration, vec![pos, to]).set_modification(MovementModification::ParabolaJump(100)))
            .with(ScaleAnimation::new(0.0, 1.0, duration))
            .with(EndOfLive::after(duration))
            .with(ZLayerGameObject)
            .with(DelayedSpawn {
                start_at: Instant::now() + duration,
                pos: to,
                z_layer: ZLayer::Floor,
                sprites: vec![sprite.clone()],
            })
            .build();
    }
}

fn random_neighbor_pos(from_pos: &WorldPos) -> WorldPos {
    let (x, y) = from_pos.as_xy();
    let choises = vec![-1.0, -0.5, 0.0, 0.5, 1.0];
    let dx = one_of(&choises);
    let dy = one_of(&choises);

    WorldPos::new(x + dx, y + dy, 0.0)
}

fn handle_projectile(
    sprite_name: &str,
    from: WorldPos,
    to: WorldPos,
    duration: Duration,
    entities: &Entities,
    updater: &Read<LazyUpdate>,
    texture_map: &Read<TextureMap>,
) {
    if let Some(sprite) = texture_map.get(sprite_name) {
        updater
            .create_entity(&entities)
            .with(Sprites::new(vec![sprite.clone()]))
            .with(Position(from))
            .with(MovementAnimation::new(duration, vec![from, to]))
            .with(ScaleAnimation::new(1.0, 2.0, duration))
            .with(EndOfLive::after(duration))
            .with(ZLayerFX)
            .build();
    }
}

fn one_of<'a, T>(v: &'a Vec<T>) -> &'a T {
    v.choose(&mut rand::thread_rng()).unwrap()
}

fn duration(fx: &FxEffect) -> Duration {
    let millis = match fx {
        FxEffect::Text(..) | FxEffect::BloodSplatter(..) => 1000,

        FxEffect::MoveTo(_, p, _) => p.len().checked_sub(1).unwrap_or(0) as u64 * 200,

        FxEffect::Projectile(_, from, to) => {
            let p1 = MapPos::from_world_pos(*from);
            let p2 = MapPos::from_world_pos(*to);

            50 * p1.distance(p2) as u64
        }

        FxEffect::Sprite(..) => 300,
    };

    Duration::from_millis(millis)
}
