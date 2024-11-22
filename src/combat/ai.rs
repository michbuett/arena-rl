use bevy::prelude::*;

use super::{
    actor::{Action, BeginActivationCommand, PreparedAction, Team},
    map::{HexMap, MapPos, Path},
};

pub fn combat_ai_plugin(app: &mut App) {
    app.observe(choose_ai_action);
}

fn choose_ai_action(
    trigger: Trigger<BeginActivationCommand>,
    activated_actor_q: Query<(&Team, &MapPos)>,
    other_actors_q: Query<(Entity, &Team, &MapPos)>,
    map: Res<HexMap>,
    mut commands: Commands,
) {
    let e = trigger.entity();
    let Ok((team, pos)) = activated_actor_q.get(e) else {
        warn!("Cannot find entity to determine ai action.");
        return;
    };

    // 1. check if actor can attack an enemy
    // TODO implement

    // 2. if unable to attack, then try to move closer
    let mut nearest_enemy: Option<(Entity, Path)> = None;

    for (e, other_team, other_pos) in other_actors_q.iter() {
        if team == other_team {
            // ignore actors from the same team
            continue;
        }
        if let Some(path) = map.find_path(*pos, *other_pos) {
            if let Some((_, path_so_far)) = &nearest_enemy {
                if path.len() < path_so_far.len() {
                    nearest_enemy = Some((e, path));
                }
            } else {
                nearest_enemy = Some((e, path));
            }
        }
    }

    if let Some((_, mut p)) = nearest_enemy {
        let path = p.drain(0..p.len() - 1).collect();
        commands
            .entity(e)
            .insert(PreparedAction(Action::MoveAlong { path }));
        return;
    }

    // 3. if unable to move closer, then skip action
    commands.entity(e).insert(PreparedAction(Action::NoOp));
}
