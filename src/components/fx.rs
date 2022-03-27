extern crate rand;

use rand::prelude::*;

use specs::prelude::*;
use specs_derive::Component;

use crate::components::*;
use crate::core::*;
use crate::ui::ScreenCoord;

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

    pub fn into_fx_vec(mut self, start_time: Instant) -> Vec<Fx> {
        self.1
            .drain(..)
            .map(|(wait, eff)| Fx(start_time + wait, eff))
            .collect()
    }

    // pub fn debug(&self) {
    //     println!("[DEBUG FxSequence] (length: {})", self.1.len());
    //     for (d, fx) in self.1.iter() {
    //         println!("  - {:?}: {:?}", d, fx); 
    //     }
    // }
}

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct Fx(Instant, FxEffect);

impl Fx {
    pub fn ends_at(&self) -> Instant {
        self.0 + self.1.duration()
    }

    pub fn run(self, world: &World) {
        let (entities, updater): (Entities, Read<LazyUpdate>) = world.system_data();

        updater.create_entity(&entities).with(self).build();
    }
}

#[derive(Debug)]
pub enum FxEffect {
    /// - Entity: the target entity (the entity which should move)
    /// - Vec<WorldPos>: the path the entity should move along
    /// - MovementModification: modification of the movement (e.g. add jump effect)
    MoveTo(ID, Vec<WorldPos>, MovementModification, u64),

    BloodSplatter(WorldPos),

    Custom {
        pos: WorldPos,
        duration: u64,
        sprite: Option<String>,
        text: Option<Text>,
        scale_anim: Option<(f32, f32)>,
        movement_anim: Option<Vec<WorldPos>>,
        fade_anim: bool,
    },
}

impl FxEffect {
    pub fn say(txt: impl ToString, pos: WorldPos) -> FxEffect {
        let txt = Text::new(txt.to_string(), FontFace::Big)
            .padding(5)
            .align(Align::MidCenter)
            .color(21, 22, 23, 255);

        FxBuilder::new(pos, 1000)
            .text(txt)
            .scale(1.0, 2.0)
            .move_to(animation_target_pos(&pos, (-150, 150), (-250, -200)))
            .fade_out()
            .build()
    }

    pub fn scream(txt: impl ToString, pos: WorldPos) -> FxEffect {
        let txt = Text::new(txt.to_string(), FontFace::VeryBig)
            .align(Align::MidCenter)
            .padding(5)
            .color(195, 31, 42, 255);

        FxBuilder::new(pos, 1000)
            .text(txt)
            .scale(1.0, 3.0)
            .move_to(animation_target_pos(&pos, (-150, 150), (-250, -200)))
            .fade_out()
            .build()
    }

    pub fn jump(id: ID, p: Vec<WorldPos>) -> Self {
        let h = 48; // half tile height
        FxEffect::MoveTo(id, p, MovementModification::ParabolaJump(h), 200)
    }

    pub fn move_along(id: ID, p: Vec<WorldPos>) -> Self {
        FxEffect::MoveTo(id, p, MovementModification::None, 100)
    }

    pub fn sprite(s: impl ToString, p: WorldPos, d: u64) -> Self {
        FxEffect::custom(p, d).sprite(s).build()
    }

    pub fn dust(s: impl ToString, p: WorldPos, d: u64) -> Self {
        FxEffect::custom(p, d)
            .sprite(s)
            .scale(1.0, 1.5)
            .move_to(animation_target_pos(&p, (0, 0), (-100, -50)))
            .fade_out()
            .build()
    }

    pub fn projectile(s: impl ToString, from: WorldPos, to: WorldPos) -> Self {
        let p1 = MapPos::from_world_pos(from);
        let p2 = MapPos::from_world_pos(to);
        let d = 50 * p1.distance(p2) as u64;

        FxBuilder::new(from, d)
            .sprite(s)
            .scale(1.0, 2.0)
            .move_to(to)
            .build()
    }

    pub fn blood_splatter(p: WorldPos) -> Self {
        FxEffect::BloodSplatter(p)
    }

    pub fn custom(p: WorldPos, d: u64) -> FxBuilder {
        FxBuilder::new(p, d)
    }

    pub fn duration(&self) -> Duration {
        let millis = match self {
            FxEffect::BloodSplatter(..) => 1000,

            FxEffect::MoveTo(_, p, _, dur) => p.len().checked_sub(1).unwrap_or(0) as u64 * dur,

            FxEffect::Custom { duration, .. } => *duration,
        };

        Duration::from_millis(millis)
    }
}

#[derive(Debug)]
pub struct FxBuilder {
    pos: WorldPos,
    duration: u64,
    sprite: Option<String>,
    text: Option<Text>,
    scale_anim: Option<(f32, f32)>,
    movement_anim: Option<Vec<WorldPos>>,
    fade_anim: bool,
}

impl FxBuilder {
    fn new(pos: WorldPos, ms: u64) -> Self {
        Self {
            pos,
            duration: ms,
            sprite: None,
            text: None,
            scale_anim: None,
            movement_anim: None,
            fade_anim: false,
        }
    }

    pub fn sprite(mut self, sprite: impl ToString) -> Self {
        self.sprite = Some(sprite.to_string());
        self
    }

    pub fn text(self, text: Text) -> Self {
        Self {
            text: Some(text),
            ..self
        }
    }

    pub fn scale(self, from: f32, to: f32) -> Self {
        Self {
            scale_anim: Some((from, to)),
            ..self
        }
    }

    pub fn move_to(mut self, target_pos: WorldPos) -> Self {
        self.movement_anim = Some(vec![self.pos, target_pos]);
        self
    }

    pub fn fade_out(mut self) -> Self {
        self.fade_anim = true;
        self
    }

    pub fn build(self) -> FxEffect {
        FxEffect::Custom {
            pos: self.pos,
            duration: self.duration,
            sprite: self.sprite,
            text: self.text,
            scale_anim: self.scale_anim,
            movement_anim: self.movement_anim,
            fade_anim: self.fade_anim,
        }
    }
}

pub struct FxSystem;

impl<'a> System<'a> for FxSystem {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, Fx>,
        ReadStorage<'a, GameObjectCmp>,
        Read<'a, LazyUpdate>,
        Read<'a, TextureMap>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, fx, game_objects, updater, texture_map) = data;
        let now = Instant::now();

        for (e, Fx(start_time, fx_eff)) in (&entities, &fx).join() {
            if now < *start_time {
                continue;
            }

            let duration = fx_eff.duration();

            match fx_eff {
                FxEffect::BloodSplatter(pos) => {
                    handle_blood_splatter(*pos, duration, &entities, &updater, &texture_map)
                }

                FxEffect::MoveTo(id, path, modification, _) => {
                    if let Some(e) = find_entity_by_id(*id, &entities, &game_objects) {
                        handle_move_to(e, path.to_vec(), duration, *modification, &updater);
                    }
                }

                FxEffect::Custom {
                    pos,
                    duration,
                    sprite,
                    text,
                    scale_anim,
                    movement_anim,
                    fade_anim,
                } => handle_custom(
                    *pos,
                    *duration,
                    &sprite,
                    &text,
                    &scale_anim,
                    &movement_anim,
                    *fade_anim,
                    &entities,
                    &updater,
                    &texture_map,
                ),
            }

            let _ = entities.delete(e);
        }
    }
}

fn find_entity_by_id(id: ID, entities: &Entities, game_objects: &ReadStorage<GameObjectCmp>) -> Option<Entity> {
    for (e, GameObjectCmp(go)) in (entities, game_objects).join() {
        if go.id() == id {
            return Some(e)
        }
    }

    None
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

fn animation_target_pos(wp: &WorldPos, dx: (i32, i32), dy: (i32, i32)) -> WorldPos {
    let mut rng = rand::thread_rng();
    let range_x = rand::distributions::Uniform::from(dx.0..=dx.1);
    let range_y = rand::distributions::Uniform::from(dy.0..=dy.1);
    let (dx, dy) = (rng.sample(range_x), rng.sample(range_y));

    ScreenCoord::translate_world_pos(*wp, dx, dy)
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
            .with(ScaleAnimation::new(1.0, 3.0, duration))
            .with(FadeAnimation::fadeout_after(duration))
            .with(EndOfLive::after(duration))
            .with(ZLayerFX)
            .with(MovementAnimation::new(
                duration,
                vec![pos, animation_target_pos(&pos, (0, 0), (-200, -100))],
            ))
            .build();

        updater
            .create_entity(&entities)
            .with(Sprites::new(vec![sprite.clone()]))
            .with(Position(pos))
            .with(
                MovementAnimation::new(duration, vec![pos, to])
                    .set_modification(MovementModification::ParabolaJump(100)),
            )
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

fn one_of<'a, T>(v: &'a Vec<T>) -> &'a T {
    v.choose(&mut rand::thread_rng()).unwrap()
}

fn handle_custom(
    pos: WorldPos,
    duration: u64,
    sprite_name: &Option<String>,
    text: &Option<Text>,
    scale_anim: &Option<(f32, f32)>,
    movement_anim: &Option<Vec<WorldPos>>,
    fade_anim: bool,
    entities: &Entities,
    updater: &Read<LazyUpdate>,
    texture_map: &Read<TextureMap>,
) {
    let duration = Duration::from_millis(duration);
    let mut new_entity = updater
        .create_entity(&entities)
        .with(Position(pos))
        .with(EndOfLive::after(duration))
        .with(ZLayerFX);

    if let Some(sprite_name) = sprite_name {
        if let Some(sprite) = texture_map.get(sprite_name) {
            new_entity = new_entity.with(Sprites::new(vec![sprite.clone()]))
        }
    }

    if let Some(txt) = text {
        new_entity = new_entity.with(txt.clone());
    }

    if let Some((scale_from, scale_to)) = scale_anim {
        new_entity = new_entity.with(ScaleAnimation::new(*scale_from, *scale_to, duration))
    }

    if let Some(path) = movement_anim {
        new_entity = new_entity.with(MovementAnimation::new(duration, path.to_vec()));
    }

    if fade_anim {
        new_entity = new_entity.with(FadeAnimation::fadeout_after(duration));
    }

    new_entity.build();
}
