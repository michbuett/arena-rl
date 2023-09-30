mod combat;
mod types;

use crate::core::*;

use combat::init_combat_data;

pub use types::*;

// const TEAM_PLAYER: Team = Team("Player", 1, true);
// const TEAM_CPU: Team = Team("Computer", 2, false);
const TEAM_PLAYER: Team = Team {
    name: "Player",
    id: 1,
    is_pc: true,
};
const TEAM_CPU: Team = Team {
    name: "Computer",
    id: 2,
    is_pc: false,
};

pub fn step<'a, 'b>(g: Game<'a, 'b>, i: &Option<UserInput>) -> Game<'a, 'b> {
    match g {
        Game::Start(gen, tex_map) => start_step(gen, tex_map, i),

        Game::TeamSelection(gen, tex_map, team) => teams_step(gen, tex_map, team, i),

        Game::Combat(combat_data) => Game::Combat(combat::step(combat_data, i)),
    }
}

fn start_step<'a, 'b>(g: ObjectGenerator, tm: TextureMap, i: &Option<UserInput>) -> Game<'a, 'b> {
    match i {
        Some(UserInput::NewGame) => {
            let player_chars = vec![
                GameObject::Actor(g.generate_player_by_type(
                    WorldPos::new(7.0, 6.0, 0.0),
                    TEAM_PLAYER,
                    ActorType::Tank,
                )),
                GameObject::Actor(g.generate_player_by_type(
                    WorldPos::new(8.0, 6.0, 0.0),
                    TEAM_PLAYER,
                    ActorType::Saw,
                )),
                GameObject::Actor(g.generate_player_by_type(
                    WorldPos::new(7.0, 7.0, 0.0),
                    TEAM_PLAYER,
                    ActorType::Spear,
                )),
                GameObject::Actor(g.generate_player_by_type(
                    WorldPos::new(8.0, 7.0, 0.0),
                    TEAM_PLAYER,
                    ActorType::Gunner,
                )),
            ];

            Game::TeamSelection(g, tm, player_chars)
        }

        _ => Game::Start(g, tm),
    }
}

fn teams_step<'a, 'b>(
    g: ObjectGenerator,
    tm: TextureMap,
    t: Vec<GameObject>,
    i: &Option<UserInput>,
) -> Game<'a, 'b> {
    match i {
        Some(UserInput::SelectTeam(..)) => {
            Game::Combat(init_combat_data(t, vec![TEAM_PLAYER, TEAM_CPU], g, tm))
            // Game::Combat(init_combat_data(game_objects.clone(), world, dispatcher))
        }

        _ => Game::TeamSelection(g, tm, t),
    }
}

// fn create_sprite_sheets() -> CombatAssets {}
