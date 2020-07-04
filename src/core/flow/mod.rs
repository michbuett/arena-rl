mod combat;
mod types;

use specs::prelude::*;

use crate::components::{register, Animation, EndOfLiveSystem};
use crate::core::map::dummy;

pub use types::*;

pub fn step<'a, 'b>(g: Game<'a, 'b>, i: &Option<UserInput>) -> Game<'a, 'b> {
    match g {
        Game::Start => start_step(g, i),

        Game::TeamSelection(..) => teams_step(i).unwrap_or(g),

        Game::Combat(combat_data) => Game::Combat(combat::step(combat_data, i)),
    }
}

fn start_step<'a, 'b>(g: Game<'a, 'b>, i: &Option<UserInput>) -> Game<'a, 'b> {
    match i {
        Some(UserInput::NewGame) => Game::TeamSelection(vec![]),

        _ => g,
    }
}

fn teams_step<'a, 'b>(i: &Option<UserInput>) -> Option<Game<'a, 'b>> {
    match i {
        Some(UserInput::SelectTeam(game_objects)) => {
            let dispatcher = DispatcherBuilder::new()
                .with(Animation, "Animaton", &[])
                .with(EndOfLiveSystem, "EOL", &[])
                .build();

            let mut world = World::new();
            let map = dummy();

            register(&mut world);

            world.add_resource(map);

            Some(Game::Combat(CombatData {
                turn: 0,
                world,
                dispatcher,
                state: CombatState::Init(game_objects.clone()),
            }))
        }
        _ => None,
    }
}
