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
    pub fn text(txt: String, WorldPos(x, y): &WorldPos, after_ms: u64) -> Self {
        // println!("HIT {} at ({}, {})", txt, x, y);
        Fx::Text(
            Text::new(txt, "big")
                .padding(5)
                .background(252, 251, 250, 255),
            Position(WorldPos(0.0, 0.0)),
            MovementAnimation {
                // start: Instant::now(),
                start: Instant::now() + Duration::from_millis(after_ms),
                duration: Duration::from_millis(250),
                loops: 1,
                from: WorldPos(*x + 0.2, *y + 0.5),
                to: WorldPos(x + 0.4, *y),
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
