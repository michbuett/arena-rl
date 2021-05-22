use crate::components::*;
use crate::core::*;
use crate::ui::ScreenPos;
    

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
            Position(*pos),
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
