mod actors;
mod animation;
mod fx;
mod sprites;

use crate::FontFace;
use std::time::{Duration, Instant};

use specs::prelude::*;
use specs_derive::Component;

use crate::core::{GameObject, SpriteConfig, Team, WorldPos};

pub use crate::components::actors::*;
pub use crate::components::animation::*;
pub use crate::components::fx::*;
pub use crate::components::sprites::*;

pub fn register(world: &mut World) {
    world.register::<Text>();
    world.register::<EndOfLive>();
    world.register::<DelayedSpawn>();
    world.register::<GameObjectCmp>();
    world.register::<ObstacleCmp>();
    world.register::<Position>();

    // from animation module
    world.register::<MovementAnimation>();
    world.register::<FadeAnimation>();
    world.register::<ScaleAnimation>();

    // from sprites module
    world.register::<Sprites>();
    world.register::<ZLayerFloor>();
    world.register::<ZLayerGameObject>();
    world.register::<ZLayerFX>();

    // from fx module
    world.register::<Fx>();
}

/// The current position of a given entity
#[derive(Component, Debug, Clone)]
#[storage(VecStorage)]
pub struct Position(pub WorldPos);

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct GameObjectCmp(pub GameObject);

#[derive(Debug, Clone)]
pub enum Restriction {
    AllowAll,
    AllowTeam(Team),
    AllowNone,
}

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct ObstacleCmp {
    pub restrict_movement: Restriction,
    pub restrict_melee_attack: Restriction,
    pub restrict_ranged_attack: Restriction,
}

#[derive(Component, Debug, Clone)]
#[storage(DenseVecStorage)]
pub struct Text {
    /// The text to display
    pub txt: String,
    /// A reference to the used font
    pub font: FontFace,
    /// (dx, dy); An optional screen offset which is relativ to its postions (e.g. Position/ScreenPosition)
    pub offset: Option<(i32, i32)>, // (dx, dy)
    pub padding: u32,
    /// (red, green, blue, alpha); Defaults to opaque black (0, 0, 0, 255)
    pub color: (u8, u8, u8, u8), // (r, g, b, a)
    /// (red, green, blue, alpha); Optional background color, None for transparent background
    pub background: Option<(u8, u8, u8, u8)>, // (r, g, b, a)
    pub border: Option<(u32, (u8, u8, u8, u8))>,
    pub alpha: u8,
    pub scale: f32,
}

impl Text {
    pub fn new(txt: String, font: FontFace) -> Self {
        Self {
            txt,
            font,
            offset: None,
            padding: 0,
            color: (0, 0, 0, 255),
            background: None,
            border: None,
            alpha: 255,
            scale: 1.0,
        }
    }

    pub fn offset(self, dx: i32, dy: i32) -> Self {
        Self {
            offset: Some((dx, dy)),
            ..self
        }
    }

    pub fn padding(self, padding: u32) -> Self {
        Self { padding, ..self }
    }

    pub fn color(self, r: u8, g: u8, b: u8, a: u8) -> Self {
        Self {
            color: (r, g, b, a),
            ..self
        }
    }

    // pub fn background(self, r: u8, g: u8, b: u8, a: u8) -> Self {
    //     Self {
    //         background: Some((r, g, b, a)),
    //         ..self
    //     }
    // }
}

#[derive(Component, Debug, Clone)]
#[storage(DenseVecStorage)]
pub struct EndOfLive(pub Instant);

impl EndOfLive {
    pub fn after(d: Duration) -> Self {
        EndOfLive(Instant::now() + d)
    }
}

pub struct EndOfLiveSystem;

impl<'a> System<'a> for EndOfLiveSystem {
    type SystemData = (Entities<'a>, ReadStorage<'a, EndOfLive>);

    fn run(&mut self, data: Self::SystemData) {
        let (entities, eol) = data;
        let now = Instant::now();

        for (e, EndOfLive(moment)) in (&entities, &eol).join() {
            if now >= *moment {
                let _ = entities.delete(e);
            }
        }
    }
}

#[derive(Component)]
#[storage(VecStorage)]
pub struct DelayedSpawn {
    pub start_at: Instant,
    pub pos: WorldPos,
    pub z_layer: ZLayer,
    pub sprites: Vec<SpriteConfig>,
}

pub struct DelayedSpawnSystem;

impl<'a> System<'a> for DelayedSpawnSystem {
    type SystemData = (
        ReadStorage<'a, DelayedSpawn>,
        Entities<'a>,
        Read<'a, LazyUpdate>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (delayed_runs, entities, updater) = data;
        let now = Instant::now();

        for (spawn, e) in (&delayed_runs, &entities).join() {
            if now >= spawn.start_at {
                let mut builder = updater
                    .create_entity(&entities)
                    .with(Sprites::new(spawn.sprites.clone()))
                    .with(Position(spawn.pos));


                builder = match spawn.z_layer {
                    ZLayer::Floor => builder.with(ZLayerFloor),
                    // ZLayer::GameObject => builder.with(ZLayerGameObject),
                    // ZLayer::Fx => builder.with(ZLayerFX),
                };

                builder.build();

                let _ = updater.remove::<DelayedSpawn>(e);
            }
        }
    }
}
