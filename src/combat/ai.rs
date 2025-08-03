use bevy::prelude::*;

use crate::core::Attacks;

use super::{
    actor::{
        Action, ActionTriggeredEvent, Active, ActorActivatedEvent, AttackData, Controller, Team,
    },
    map::{HexMap, MapPos, Path},
};

pub fn combat_ai_plugin(app: &mut App) {
    app.add_observer(choose_ai_action);
}

fn choose_ai_action(
    trigger: Trigger<ActorActivatedEvent>,
    activated_actor_q: Query<(&Team, &MapPos, &Attacks, &Controller, &Active)>,
    other_actors_q: Query<(Entity, &Team, &MapPos)>,
    map: Res<HexMap>,
    mut commands: Commands,
) -> Result<(), BevyError> {
    // info!("[ai::choose_ai_action] entity={:?}", trigger.target());

    let ActorActivatedEvent(actor) = trigger.event();
    let (team, pos, attacks, controller, Active(activation)) = activated_actor_q.get(*actor)?;

    if controller.is_pc() {
        // ignore player controlled actors
        return Ok(());
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
        for attack_template in attacks.0.iter() {
            if attack_template.can_attack(p.len() as i32 - 1) {
                commands.trigger(ActionTriggeredEvent(Action::Attack(AttackData::new(
                    *actor,
                    target,
                    attack_template,
                    *activation,
                ))));
                return Ok(());
            }
        }

        // 2. if unable to attack, then try to move closer
        let path = p.drain(0..p.len() - 1).collect();
        commands.trigger_targets(
            ActionTriggeredEvent(Action::MoveAlong {
                actor: *actor,
                path,
            }),
            *actor,
        );
        return Ok(());
    }

    // 3. if unable to move closer, then skip action
    commands.trigger_targets(ActionTriggeredEvent(Action::NoOp(*actor)), *actor);
    Ok(())
}
