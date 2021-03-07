use std::time::Instant;

use specs::prelude::*;

use super::super::action::{act, Action, Change};
use super::super::actors::{generate_enemy_easy, generate_player, generate_player2, Actor, GameObject, Team};
// use super::super::ai::{action, actions_at, reaction, select_action};
use super::super::ai::{action, actions_at};
use super::types::*;
use crate::components::*;
use crate::core::WorldPos;
use crate::core::DisplayStr;

const TEAM_PLAYER: Team = Team("Player", 1, true);
const TEAM_CPU: Team = Team("Computer", 2, false);

pub fn init_combat_data<'a, 'b>(
    game_objects: Vec<GameObject>,
    world: World,
    dispatcher: Dispatcher<'a, 'b>,
) -> CombatData<'a, 'b> {
    CombatData {
        turn: 0,
        active_team_idx: 0,
        teams: vec![TEAM_PLAYER, TEAM_CPU],
        world,
        dispatcher,
        state: CombatState::Init(game_objects),
        log: vec!(),
    }
}

/// Steps the game one tick forward using the given user input
pub fn step<'a, 'b>(g: CombatData<'a, 'b>, i: &Option<UserInput>) -> CombatData<'a, 'b> {
    let CombatData {
        turn,
        active_team_idx,
        teams,
        state,
        mut dispatcher,
        mut world,
        mut log,
    } = g;

    let (next_turn, next_active_team, next_state, log_entry) =
        next_state(turn, &state, active_team_idx, &teams, i, &world);

    dispatcher.dispatch(&mut world);
    world.maintain();

    if let Some(log_entry) = log_entry {
        log.insert(0, log_entry);
        if log.len() > 10 {
            log.pop();
        }
    }

    return CombatData {
        turn: next_turn,
        active_team_idx: next_active_team,
        teams,
        state: next_state.unwrap_or(state),
        dispatcher,
        world,
        log,
    };
}

fn find_active_actor(world: &World) -> Option<(Entity, Actor)> {
    let (entities, actors): (Entities, ReadStorage<GameObjectCmp>) = world.system_data();

    for (e, o) in (&entities, &actors).join() {
        if let GameObject::Actor(actor) = &o.0 {
            if actor.active {
                return Some((e, actor.clone()));
            }
        }
    }

    None
}

fn next_ready_entity(world: &World, active_team: &Team) -> Option<(Entity, Actor)> {
    let (entities, actors): (Entities, ReadStorage<GameObjectCmp>) = world.system_data();

    for (e, o) in (&entities, &actors).join() {
        if let GameObject::Actor(actor) = o.0.clone() {
            if &actor.team == active_team && actor.can_activate() {
                return Some((e, actor));
            }
        }
    }

    None
}

fn handle_wait_until(t: &Instant, ol: &Vec<EntityAction>) -> Option<CombatState> {
    if *t > Instant::now() {
        // now is not the time!
        // => do nothing and wait some more
        return None;
    }

    if let Some((entity_action, tail)) = ol.split_first() {
        // wait time is up but there are more action queued up
        // => continue with next action in queue
        Some(CombatState::ResolveAction(
            entity_action.clone(),
            tail.to_vec(),
        ))
    } else {
        // wait time is up and no further reactions to handle
        // => continue with next actor
        Some(CombatState::FindActor())
    }
}

fn handle_wait_for_user_action(
    e: &(Entity, Actor),
    ctxt: &Option<InputContext>,
    i: &Option<UserInput>,
    w: &World,
) -> Option<CombatState> {
    match i {
        Some(UserInput::SelectWorldPos(pos)) => {
            // user tries to select a new world pos to get new options
            // => if if the current context allows to change the selected
            //    world pos (e.g. it is not allowed to switch when resolving
            //    a combat)

            let can_change_selected_area = match ctxt {
                None | Some(InputContext::SelectedArea(..)) => true,
                // _ => false,
            };

            if can_change_selected_area {
                // it is allowed
                // => determine the new possible actions and wait for the next
                //    user input
                let pos = WorldPos(pos.0.floor(), pos.1.floor());
                let objects = find_objects_at(&pos, &w);
                let actions = actions_at(e, pos, &w);
                let ui = InputContext::SelectedArea(pos, objects, actions);

                Some(CombatState::WaitForUserAction(e.clone(), Some(ui)))
            } else {
                // user tries to select a new area but is not allowed to
                // change it (e.g. when handling an reaction)
                // => ignore the input and wait some more
                None
            }
        }

        Some(UserInput::SelectAction((action, delay))) => {
            // user has selected an action
            // => resolve that action
            Some(CombatState::ResolveAction(
                (e.0.clone(), e.1.clone(), action.clone(), *delay),
                Vec::new(),
            ))
        }

        // no user input
        // => we wait some more
        _ => None,
    }
}

// fn find_winning_team(world: &World) -> Option<Team> {
//     let actors: ReadStorage<GameObjectCmp> = world.system_data();
//     let mut winning_team = None;

//     for GameObjectCmp(o) in (&actors).join() {
//         if let GameObject::Actor(a) = o {
//             if let Some(candidate) = winning_team {
//                 if candidate != a.team {
//                     // there are two actors of different team
//                     // => no winning team so far (exit)
//                     return None;
//                 }
//             }

//             winning_team = Some(a.team.clone());
//         }
//     }

//     winning_team
// }

fn find_objects_at(pos: &WorldPos, world: &World) -> Vec<GameObject> {
    let game_objects: ReadStorage<GameObjectCmp> = world.system_data();
    let mut result = Vec::new();

    for GameObjectCmp(o) in (&game_objects).join() {
        match o {
            GameObject::Actor(a) => {
                let WorldPos(x, y) = a.pos;

                if x.floor() == pos.0.floor() && y.floor() == pos.1.floor() {
                    result.push(o.clone());
                }
            }

            GameObject::Item(WorldPos(x, y), _) => {
                if x.floor() == pos.0.floor() && y.floor() == pos.1.floor() {
                    result.push(o.clone());
                }
            }
        }
    }

    result
}

fn next_state<'a, 'b>(
    round: u64,
    state: &CombatState,
    active_team_idx: usize,
    teams: &Vec<Team>,
    i: &Option<UserInput>,
    w: &World,
) -> (u64, usize, Option<CombatState>, Option<DisplayStr>) {
    match state {
        CombatState::Init(_game_objects) => {
            // TODO use configured characters
            // -> find way to inject team and pos

            let game_objects = vec![
                GameObject::Actor(generate_player2(WorldPos(7.0, 6.0), TEAM_PLAYER)),
                GameObject::Actor(generate_player(WorldPos(8.0, 6.0), TEAM_PLAYER)),
                GameObject::Actor(generate_player(WorldPos(7.0, 7.0), TEAM_PLAYER)),
                GameObject::Actor(generate_player(WorldPos(8.0, 7.0), TEAM_PLAYER)),
            ];

            for o in game_objects {
                insert_game_object_components(o.clone(), w);
            }

            spawn_enemies(round, w);

            (round, active_team_idx, Some(CombatState::StartTurn()), None)
        }

        CombatState::StartTurn() => {
            let mut entity_actions = Vec::new();
            let (entities, objects): (Entities, ReadStorage<GameObjectCmp>) = w.system_data();

            let active_team: &Team = teams.get(active_team_idx).unwrap();
            for (e, GameObjectCmp(o)) in (&entities, &objects).join() {
                if let GameObject::Actor(a) = o {
                    if &a.team == active_team {
                        entity_actions.push((e, a.clone(), Action::StartTurn(), 0));
                    }
                }
            }

            if let Some((entity_action, tail)) = entity_actions.split_first() {
                // wait time is up but there are more action queued up
                // => continue with next action in queue
                (
                    round,
                    active_team_idx,
                    Some(CombatState::ResolveAction(
                        entity_action.clone(),
                        tail.to_vec(),
                    )),
                    None,
                )
            } else {
                // wait time is up and no further reactions to handle
                // => continue with next actor
                (round, active_team_idx, Some(CombatState::FindActor()), None)
            }
        }

        CombatState::FindActor() => {
            // TODO handle WIN/LOSE condition
            // if let Some(team) = find_winning_team(w) {
            //     if team == TEAM_CPU {
            //         return (round, Some(CombatState::Win(team)));
            //     }
            // }

            if let Some(ea) = find_active_actor(w) {
                // there is an active actor
                // -> check if it can do some action
                return (round, active_team_idx, Some(CombatState::SelectAction(ea)), None);
            }

            let active_team: &Team = teams.get(active_team_idx).unwrap();
            if let Some(ea) = next_ready_entity(w, active_team) {
                let next_state =
                    CombatState::ResolveAction((ea.0, ea.1, Action::Activate(ea.0), 0), vec![]);

                (round, active_team_idx, Some(next_state), None)
            } else {
                // there are no more entities with a turn left...
                if active_team_idx < teams.len() - 1 {
                    // ... then continue with next team
                    (round, active_team_idx + 1, Some(CombatState::StartTurn()), None)
                } else {
                    // ... or start a new round beginning with the first team
                    (round + 1, 0, Some(CombatState::StartTurn()), None)
                }
            }
        }

        CombatState::SelectAction(ea) => {
            if ea.1.is_pc() {
                // the next ready actor is a player controlled entity
                // => wait for user input;
                //    so far we have no context for the input (e.g. selected
                //    world position, ...) and no reactions
                (
                    round,
                    active_team_idx,
                    Some(CombatState::WaitForUserAction(ea.clone(), None)),
                    None,
                )
            } else {
                // the next ready actor is a player controlled entity
                // => let the AI compute an action and resolve it
                //    so far we have no reactions
                let (action, delay) = action(&ea, w);

                (
                    round,
                    active_team_idx,
                    Some(CombatState::ResolveAction(
                        (ea.0, ea.1.clone(), action, delay),
                        Vec::new(),
                    )),
                    None,
                )
            }
        }

        CombatState::ResolveAction(entity_action, remaining_actions) => {
            let (change, durr, log_entry) = act(entity_action.clone(), w);

            for c in change {
                match c {
                    Change::Update(e, o) => update_components(e, o, w),
                    Change::Insert(o) => insert_game_object_components(o, w),
                    Change::Remove(e) => remove_components(e, w),
                    Change::Fx(fx) => fx.run(w),
                }
            }

            (
                round,
                active_team_idx,
                Some(CombatState::WaitUntil(
                    Instant::now() + durr,
                    remaining_actions.to_vec(),
                )),
                log_entry,
            )
        }

        CombatState::WaitForUserAction(e, ctxt) => (
            round,
            active_team_idx,
            handle_wait_for_user_action(&e, &ctxt, i, w),
            None,
        ),

        CombatState::WaitUntil(t, ol) => (round, active_team_idx, handle_wait_until(t, ol), None),

        // CombatState::Win(_) => {
        //     // ignore
        //     (round, active_team_idx, None)
        // }
    }
}

fn spawn_enemies(_turn: u64, w: &World) {
    vec![
        GameObject::Actor(generate_enemy_easy(WorldPos(1.0, 6.0), TEAM_CPU)),
        GameObject::Actor(generate_enemy_easy(WorldPos(1.0, 5.0), TEAM_CPU)),
        GameObject::Actor(generate_enemy_easy(WorldPos(6.0, 0.0), TEAM_CPU)),
        GameObject::Actor(generate_enemy_easy(WorldPos(7.0, 0.0), TEAM_CPU)),
    ]
    .drain(..)
    .for_each(move |enemy| insert_game_object_components(enemy, w));
}
