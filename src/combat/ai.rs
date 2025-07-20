use bevy::prelude::*;

use crate::core::Attacks;

use super::{
    actor::{
        Action, ActionTriggeredEvent, AttackData, BeginActivationCommand, PlayerControlled, Team,
    },
    map::{HexMap, MapPos, Path},
};

pub fn combat_ai_plugin(app: &mut App) {
    app.add_observer(choose_ai_action);
}

fn choose_ai_action(
    trigger: Trigger<BeginActivationCommand>,
    activated_actor_q: Query<(&Team, &MapPos, &Attacks, &PlayerControlled)>,
    other_actors_q: Query<(Entity, &Team, &MapPos)>,
    map: Res<HexMap>,
    mut commands: Commands,
) {
    // info!("[ai::choose_ai_action] entity={:?}", trigger.target());

    let e = trigger.target();
    let Ok((team, pos, attacks, PlayerControlled(is_pc))) = activated_actor_q.get(e) else {
        warn!("Cannot find entity to determine ai action.");
        return;
    };

    if *is_pc {
        // ignore player controlled actors
        return;
    }

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

    if let Some((target, mut p)) = nearest_enemy {
        // 1. check if actor can attack an enemy
        for ao in attacks.0.iter() {
            if ao.can_attack(p.len() as i32 - 1) {
                commands.trigger_targets(
                    ActionTriggeredEvent(Action::Attack(AttackData::new(target, ao))),
                    e,
                );
                return;
            }
        }

        // 2. if unable to attack, then try to move closer
        let path = p.drain(0..p.len() - 1).collect();
        commands.trigger_targets(ActionTriggeredEvent(Action::MoveAlong { path }), e);
        return;
    }

    // 3. if unable to move closer, then skip action
    commands.trigger_targets(ActionTriggeredEvent(Action::NoOp), e);
}
