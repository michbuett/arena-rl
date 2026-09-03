use bevy::prelude::*;

use crate::combat::commands::ManeuverTemplate;

use super::{
    actor::{Actor, Controller, Team},
    commands::{
        ActorManeuvers, InputStepCompletedEvent, InputValue, SelectActorCommand,
        SelectManeuverCommand, SelectPathCommand,
    },
    map::{HexMap, MapPos, Path},
};

pub fn combat_ai_plugin(app: &mut App) {
    app.add_observer(handle_select_maneuver_command)
        .add_observer(handle_select_path_command)
        .add_observer(handle_select_actor_command);
}

fn handle_select_maneuver_command(
    trigger: On<SelectManeuverCommand>,
    controller_q: Query<(&Controller, &ActorManeuvers)>,
    mut commands: Commands,
) -> Result<(), BevyError> {
    let SelectManeuverCommand {
        actor,
        input_id,
        filter,
        is_reaction,
        ..
    } = trigger.event();

    let (controller, ActorManeuvers(maneuvers)) = controller_q.get(*actor)?;
    if controller.is_pc() {
        // ignore player controlled actors
        return Ok(());
    }

    let mut options = maneuvers
        .iter()
        .filter(|m| {
            *is_reaction == matches!(m, ManeuverTemplate::Reactive(..))
                && m.keywords().contains(*filter)
        })
        .cloned()
        .collect::<Vec<_>>();

    // println!(
    //     "AI choose maneuver - available options: {}",
    //     options
    //         .iter()
    //         .map(|t| t.name.clone())
    //         .collect::<Vec<_>>()
    //         .join(", "),
    // );

    if options.is_empty() {
        // No suitable maneuver possible
        // => skip this maneuver
        commands.trigger(InputStepCompletedEvent {
            input_id: input_id.clone(),
            input_value: InputValue::None,
        });
    } else {
        // TODO implement more sofisticated decision making
        let selected_maneuver = options.remove(0);

        commands.trigger(InputStepCompletedEvent {
            input_id: input_id.clone(),
            input_value: InputValue::Maneuver {
                actor: *actor,
                template: selected_maneuver,
            },
        });
    }

    Ok(())
}

fn handle_select_path_command(
    trigger: On<SelectPathCommand>,
    controller_q: Query<(&Controller, &Team, &MapPos)>,
    other_actors_q: Query<(Entity, &Team, &MapPos)>,
    map: Res<HexMap>,
    mut commands: Commands,
) -> Result<(), BevyError> {
    let SelectPathCommand {
        actor,
        input_id,
        length,
        ..
    } = trigger.event();

    let (controller, team, start) = controller_q.get(*actor)?;
    if controller.is_pc() {
        // ignore player controlled actors
        return Ok(());
    }

    // println!("AI select path");
    let nearest_enemy = find_nearest_enemy(*start, team, map.as_ref(), other_actors_q.iter());

    if let Some((_, p)) = nearest_enemy {
        let path = p.iter().take((*length).into()).copied().collect();

        commands.trigger(InputStepCompletedEvent {
            input_id: input_id.clone(),
            input_value: InputValue::Path(path),
        });
    } else {
        commands.trigger(InputStepCompletedEvent {
            input_id: input_id.clone(),
            input_value: InputValue::None,
        });
    }

    Ok(())
}

fn handle_select_actor_command(
    trigger: On<SelectActorCommand>,
    controller_q: Query<(&Controller, &Team, &MapPos)>,
    possible_targets_q: Query<(Entity, &Team, &MapPos), With<Actor>>,
    mut commands: Commands,
) -> Result<(), BevyError> {
    let SelectActorCommand {
        actor,
        input_id,
        filter,
        ..
    } = trigger.event();

    let (controller, actor_team, actor_pos) = controller_q.get(*actor)?;
    if controller.is_pc() {
        return Ok(());
    }

    // println!("AI select target actor");
    let candidates = possible_targets_q
        .iter()
        .filter_map(|c| {
            if filter.compare(c, (*actor, actor_team, actor_pos)) {
                Some(c.0)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    if candidates.len() > 0 {
        // println!("> possible target: {candidates:?}");
        commands.trigger(InputStepCompletedEvent {
            input_id: input_id.clone(),
            input_value: InputValue::Actor(candidates.first().unwrap().to_owned()),
        });
    } else {
        // println!("> no targets found");
        commands.trigger(InputStepCompletedEvent {
            input_id: input_id.clone(),
            input_value: InputValue::None,
        });
    }

    Ok(())
}

fn find_nearest_enemy<'a>(
    pos: MapPos,
    team: &Team,
    map: &HexMap,
    other_actors_q: impl Iterator<Item = (Entity, &'a Team, &'a MapPos)>,
) -> Option<(Entity, Path)> {
    let mut nearest_enemy: Option<(Entity, Path)> = None;
    for (e, other_team, other_pos) in other_actors_q {
        if team == other_team {
            // ignore actors from the same team
            continue;
        }
        for (neighbor_pos, _, _) in map.neighbors(*other_pos, 1) {
            if let Some(path) = map.find_path(pos, neighbor_pos) {
                if let Some((_, path_so_far)) = &nearest_enemy {
                    if path.len() < path_so_far.len() {
                        nearest_enemy = Some((e, path));
                    }
                } else {
                    nearest_enemy = Some((e, path));
                }
            }
        }
    }

    nearest_enemy
}
