use crate::components::*;
use crate::core::*;

// pub fn fx_ouch(WorldPos(x, y): WorldPos) -> (Position, Sprite, MovementAnimation, EndOfLive) {
//     (
//         Position(WorldPos(0.0, 0.0)),
//         Sprite {
//             texture: String::from("eff_ouch"),
//             region: (0, 0, 130, 80),
//             offset: (0, 0),
//         },
//         MovementAnimation {
//             // start: Instant::now(),
//             start: Instant::now() + Duration::from_millis(200),
//             duration: Duration::from_millis(300),
//             loops: 1,
//             from: WorldPos(x, y),
//             to: WorldPos(x + 0.5, y - 1.0),
//         },
//         EndOfLive::after_ms(500),
//     )
// }

pub enum Fx {
    Text(Text, Position, MovementAnimation, EndOfLive),
}

impl Fx {
    pub fn text(txt: String, pos: &WorldPos, after_ms: u64) -> Self {
        Fx::Text(
            Text::new(txt, FontFace::VeryBig)
                .padding(5)
                .color(150, 21, 22, 255)
                .background(252, 251, 250, 155),
            Position(WorldPos(0.0, 0.0)),
            MovementAnimation {
                start: Instant::now() + Duration::from_millis(after_ms),
                duration: Duration::from_millis(250),
                loops: 1,
                from: *pos,
                to: animation_target_pos(pos),
            },
            EndOfLive::after_ms(500 + after_ms),
        )
    }

    pub fn run(self, world: &World) {
        let (entities, updater): (Entities, Read<LazyUpdate>) = world.system_data();

        match self {
            Fx::Text(txt, pos, anim, eol) => {
                updater
                    .create_entity(&entities)
                    .with(pos)
                    .with(txt)
                    .with(anim)
                    .with(eol)
                    .build();
            }
        }
    }
}

fn animation_target_pos(WorldPos(x, y): &WorldPos) -> WorldPos {
    extern crate rand;
    use rand::prelude::*;
    let range = rand::distributions::Uniform::from(-1.0..=1.0);
    let mut rng = rand::thread_rng();

    WorldPos(x + rng.sample(range), y + rng.sample(range))
}
