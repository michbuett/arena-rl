mod combat;
mod types;

use crate::core::*;

use combat::init_combat_data;

pub use types::*;

const TEAM_PLAYER: u8 = 1;
const TEAM_CPU: u8 = 2;

pub fn step<'a, 'b>(g: Game<'a, 'b>, i: &Option<UserInput>) -> Game<'a, 'b> {
    match g {
        Game::Start(gen, tex_map) => start_step(gen, tex_map, i),

        Game::TeamSelection(gen, tex_map, team) => teams_step(gen, tex_map, team, i),

        Game::Combat(combat_data) => Game::Combat(combat::step(combat_data, i)),
    }
}

fn start_step<'a, 'b>(g: ObjectGenerator, tm: TextureMap, i: &Option<UserInput>) -> Game<'a, 'b> {
    let team_id_player = TeamId::new(TEAM_PLAYER);

    match i {
        Some(UserInput::NewGame) => {
            let player_chars = vec![
                (g.generate_player(
                    WorldPos::new(7.0, 6.0, 0.0),
                    team_id_player,
                    ActorTemplateName::new("actor#tank"),
                )),
                (g.generate_player(
                    WorldPos::new(8.0, 6.0, 0.0),
                    team_id_player,
                    ActorTemplateName::new("actor#saw"),
                )),
                (g.generate_player(
                    WorldPos::new(7.0, 7.0, 0.0),
                    team_id_player,
                    ActorTemplateName::new("actor#spear"),
                )),
                (g.generate_player(
                    WorldPos::new(8.0, 7.0, 0.0),
                    team_id_player,
                    ActorTemplateName::new("actor#gunner"),
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
    t: Vec<Actor>,
    i: &Option<UserInput>,
) -> Game<'a, 'b> {
    match i {
        Some(UserInput::SelectTeam(..)) => Game::Combat(init_combat_data(
            t,
            vec![create_team_player(), create_team_cpu()],
            g,
            tm,
        )),

        _ => Game::TeamSelection(g, tm, t),
    }
}

fn create_team_player() -> Team {
    Team {
        name: "Player",
        id: TeamId::new(TEAM_PLAYER),
        is_pc: true,
        reinforcements: None,
    }
}

fn create_team_cpu() -> Team {
    Team {
        name: "Computer",
        id: TeamId::new(TEAM_CPU),
        is_pc: false,
        reinforcements: Some(vec![
            // initial (1st) wave
            (1, MapPos(1, 6), ActorTemplateName::new("enemy#sucker")),
            (1, MapPos(1, 7), ActorTemplateName::new("enemy#sucker")),
            (1, MapPos(6, 0), ActorTemplateName::new("enemy#sucker")),
            (1, MapPos(7, 0), ActorTemplateName::new("enemy#sucker")),
            // 2nd wave
            (5, MapPos(1, 6), ActorTemplateName::new("enemy#worm")),
            (5, MapPos(1, 7), ActorTemplateName::new("enemy#worm")),
            // 3rd wave
            (10, MapPos(6, 0), ActorTemplateName::new("enemy#zombi")),
            (10, MapPos(7, 0), ActorTemplateName::new("enemy#zombi")),
        ]),
    }
}
