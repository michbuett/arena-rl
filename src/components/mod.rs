mod actors;
mod animation;
mod fx;
mod sprites;

use crate::FontFace;
use std::num::NonZeroU8;
use std::time::{Duration, Instant};

use specs::prelude::{
    Builder, Component, DenseVecStorage, Entities, Join, LazyUpdate, Read, ReadStorage, System,
    VecStorage, World, WorldExt,
};
use specs_derive::Component;

use crate::core::{DisplayStr, GameObject, Obstacle, SpriteConfig, WorldPos};
use crate::ui::{Align, ScreenPos, ScreenText};

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
    world.register::<HoverAnimation>();

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
pub struct HitArea {
    obstacle: Obstacle,
    offset: (f32, f32),
    dim: (f32, f32),
}

impl HitArea {
    pub fn intersect_line_at(&self, pos: (f32, f32), p0: (f32, f32), p1: (f32, f32)) -> bool {
        let (x0, y0) = p0;
        let (x1, y1) = p1;
        let dx = x1 - x0;
        let dy = y1 - y0;
        let (x, y) = (pos.0 + self.offset.0, pos.1 + self.offset.1);
        let (w, h) = self.dim;

        if dx == 0.0 {
            // vertical line
            return x <= x0 && x0 <= x + w;
        }

        let m = dy / dx;
        let n = y0 - m * x0;

        if m > 0.0 {
            if m * x + n > y + h {
                return false;
            }

            if m * (x + w) + n < y {
                return false;
            }
        } else {
            if m * x + n < y {
                return false;
            }

            if m * (x + w) + n > y + h {
                return false;
            }
        }

        true
    }
}

#[derive(Debug, Clone)]
pub struct Hitbox {
    inner: Option<HitArea>,
    outer: HitArea,
}

impl Hitbox {
    pub fn new_pillar() -> Self {
        Self {
            inner: Some(HitArea {
                obstacle: Obstacle::Blocker,
                offset: (-0.2, -0.2),
                dim: (0.4, 0.4),
            }),
            outer: HitArea {
                obstacle: Obstacle::Impediment(NonZeroU8::new(80).unwrap(), 2),
                offset: (-0.5, -0.5),
                dim: (1.0, 1.0),
            },
        }
    }

    pub fn new_normal_actor() -> Self {
        Self {
            inner: None,
            outer: HitArea {
                obstacle: Obstacle::Impediment(NonZeroU8::new(50).unwrap(), 1),
                offset: (-0.4, -0.4),
                dim: (0.8, 0.8),
            },
        }
    }

    pub fn obstacle_at(&self, pos: (f32, f32), p0: (f32, f32), p1: (f32, f32)) -> Option<Obstacle> {
        if let Some(inner_hit_area) = self.inner.as_ref() {
            if inner_hit_area.intersect_line_at(pos, p0, p1) {
                return Some(inner_hit_area.obstacle);
            }
        }

        if self.outer.intersect_line_at(pos, p0, p1) {
            return Some(self.outer.obstacle);
        }

        None
    }
}

#[derive(Component, Debug, Clone)]
#[storage(VecStorage)]
pub struct ObstacleCmp {
    /// The handicaps for moving by foot, flying and underground
    pub movement: (Option<Obstacle>, Option<Obstacle>, Option<Obstacle>),
    /// The handicap for physically reaching sth (e.g. for an attack)
    pub reach: Option<Hitbox>,
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
    pub align: Align,
    pub text_align: Align,
    pub width: Option<u32>,
}

impl Default for Text {
    fn default() -> Self {
        Self {
            txt: "(empty)".to_string(),
            font: FontFace::Normal,
            offset: None,
            padding: 0,
            color: (0, 0, 0, 255),
            background: None,
            border: None,
            alpha: 255,
            scale: 1.0,
            align: Align::TopLeft,
            text_align: Align::TopLeft,
            width: None,
        }
    }
}

impl Text {
    pub fn into_screen_text(&self, pos: ScreenPos) -> ScreenText {
        let mut t = ScreenText::new(DisplayStr::new(self.txt.clone()), pos)
            .font(self.font)
            .color(self.color)
            .padding(self.padding)
            .alpha(self.alpha)
            .scale(self.scale)
            .align(self.align)
            .text_align(self.text_align);

        if let Some(bg) = self.background {
            t = t.background(bg);
        }

        if let Some((b_size, b_color)) = self.border {
            t = t.border(b_size, b_color);
        }

        if let Some(w) = self.width {
            t = t.width(w);
        }

        t
    }
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
    pub fade_out: Option<Duration>,
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

                if let Some(durration) = spawn.fade_out {
                    builder = builder
                        .with(FadeAnimation::fadeout_after(durration))
                        .with(EndOfLive::after(durration));
                }

                builder.build();

                let _ = updater.remove::<DelayedSpawn>(e);
            }
        }
    }
}
