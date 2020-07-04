use std::time::Instant;

use specs::prelude::*;

use super::super::actors::{GameObject, Actor, Team};
use super::super::action::{Action, Reaction, Change, run_action};
use super::super::ai::{action, actions_at, reaction, select_action};
use super::types::*;
use crate::components::*;
use crate::core::{WorldPos};


/// Steps the game one tick forward using the given user input
pub fn step<'a, 'b>(g: CombatData<'a, 'b>, i: &Option<UserInput>) -> CombatData<'a, 'b> {
    let CombatData {
        turn,
        state,
        mut dispatcher,
        mut world,
    } = g;

    let (next_turn, next_state) = next_state(turn, &state, i, &world);

    dispatcher.dispatch(&mut world.res);
    world.maintain();

    return CombatData {
        turn: next_turn,
        state: next_state.unwrap_or(state),
        dispatcher,
        world,
    };
}

fn next_ready_entity(world: &World, turn: u64) -> Option<(Entity, Actor)> {
    let (entities, actors): (Entities, ReadStorage<GameObjectCmp>) = world.system_data();
    let mut result = None;
    let mut max_initiative = 0;

    for (e, o) in (&entities, &actors).join() {
        if let GameObject::Actor(actor) = o.0.clone() {
            let initiative = actor.initiative();
            if actor.turn <= turn && initiative > max_initiative {
                result = Some((e, actor.clone()));
                max_initiative = initiative;
            }
        }
    }

    if let Some((e, mut a)) = result {
        // TODO there may be a nicer way to mark the active actor
        a.active = true;
        update_components(e, GameObject::Actor(a.clone()), world);

        return Some((e, a));
    }

    None
}

fn handle_wait_until(
    t: &Instant,
    ol: &Vec<Reaction>,
    w: &World,
) -> Option<CombatState> {
    if *t > Instant::now() {
        // now is not the time!
        // => do nothing and wait some more
        return None;
    }

    if let Some(((e, o), tail)) = ol.split_first() {
        let tail = tail.to_vec();
        let actions = reaction(e, o.clone());

        if e.1.is_pc() {
            let reaction = InputContext::Opportunity(o.clone(), actions.to_vec());

            Some(CombatState::WaitForUserAction(e.clone(), Some(reaction), tail))
        } else {
            let action = select_action(e.clone(), o, &actions, w);

            Some(CombatState::ResolveAction(e.clone(), action, tail))
        }
    } else {
        // wait time is up and no further reactions to handle
        // => continue with next actor
        Some(CombatState::FindActor())
    }
}


fn handle_wait_for_user_action(
    e: &(Entity, Actor),
    ctxt: &Option<InputContext>,
    rl: &Vec<Reaction>,
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
                _ => false,
            };

            if can_change_selected_area {
                // it is allowed
                // => determine the new possible actions and wait for the next
                //    user input
                let pos = WorldPos(pos.0.floor(), pos.1.floor());
                let objects = find_objects_at(&pos, &w);
                let actions = actions_at(e, pos, &w);
                let ui = InputContext::SelectedArea(pos, objects, actions);

                Some(CombatState::WaitForUserAction(e.clone(), Some(ui), rl.to_vec()))
            } else {
                // user tries to select a new area but is not allowed to
                // change it (e.g. when handling an reaction)
                // => ignore the input and wait some more
                None
            }
        }

        Some(UserInput::SelectAction(a)) => {
            // user has selected an action
            // => resolve that action
            Some(CombatState::ResolveAction(e.clone(), a.clone(), rl.to_vec()))
        }

        // no user input
        // => we wait some more
        _ => None,
    }
}

fn find_winning_team(world: &World) -> Option<Team> {
    let actors: ReadStorage<GameObjectCmp> = world.system_data();
    let mut winning_team = None;

    for GameObjectCmp(o) in (&actors).join() {
        if let GameObject::Actor(a) = o {
            if let Some(candidate) = winning_team {
                if candidate != a.team {
                    // there are two actors of different team
                    // => no winning team so far (exit)
                    return None;
                }
            }

            winning_team = Some(a.team.clone());
        }
    }

    winning_team
}

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

fn find_last_turn_entity(turn: u64, w: &World) -> Option<(Entity, Actor)> {
    let (entities, objects): (Entities, ReadStorage<GameObjectCmp>) = w.system_data();

    for (e, GameObjectCmp(o)) in (&entities, &objects).join() {
        match o {
            GameObject::Actor(a) => {
                if a.turn <= turn {
                    return Some((e, a.clone()));
                }
            }
            _ => {}
        }
    }

    None
}

fn next_state<'a, 'b>(
    turn: u64,
    state: &CombatState,
    i: &Option<UserInput>,
    w: &World,
) -> (u64, Option<CombatState>) {
    match state {
        CombatState::Init(game_objects) => {
            for o in game_objects {
                insert_game_object_components(o.clone(), w);
            }

            (turn, Some(CombatState::FindActor()))
        }

        CombatState::FindActor() => {
            if let Some(team) = find_winning_team(w) {
                return (turn, Some(CombatState::Win(team)));
            }

            if let Some(ea) = next_ready_entity(w, turn) {
                // println!("[next actor] {:?}", ea.0);
                if ea.1.is_pc() {
                    // the next ready actor is a player controlled entity
                    // => wait for user input;
                    //    so far we have no context for the input (e.g. selected
                    //    world position, ...) and no reactions
                    (turn, Some(CombatState::WaitForUserAction(ea, None, Vec::new())))
                } else {
                    // the next ready actor is a player controlled entity
                    // => let the AI compute an action and resolve it
                    //    so far we have no reactions
                    let action = action(&ea, w);

                    (turn, Some(CombatState::ResolveAction(ea, action, Vec::new())))
                }
            } else {
                // there are no more entities with a turn left
                // => end current turn
                (turn, Some(CombatState::EndTurn()))
            }
        }

        CombatState::EndTurn() => {
            if let Some(game_object) = find_last_turn_entity(turn, w) {
                (
                    turn,
                    Some(CombatState::ResolveAction(
                        game_object,
                        Action::EndTurn(turn + 1),
                        vec![],
                    )),
                )
            } else {
                // no further in the current turn
                // => step the game one turn forward
                (turn + 1, Some(CombatState::FindActor()))
            }
        }

        CombatState::ResolveAction(ea, action, remaining_reactions) => {
            // println!("[resolve action] {:?}", ea.0);
            let (change, durr, new_reactions) = run_action(ea.clone(), action.clone());
            let mut reactions = remaining_reactions.to_vec();

            reactions.extend(new_reactions);

            for c in change {
                match c {
                    Change::Update(e, o) => update_components(e, o, w),
                    Change::Insert(o) => insert_game_object_components(o, w),
                    Change::Remove(e) => remove_components(e, w),
                    Change::Fx(fx) => fx.run(w),
                }
            }

            (
                turn,
                Some(CombatState::WaitUntil(Instant::now() + durr, reactions)),
            )
        }

        CombatState::WaitForUserAction(e, ctxt, rl) => {
            (turn, handle_wait_for_user_action(&e, &ctxt, &rl, i, w))
        }

        CombatState::WaitUntil(t, ol) => {
            (turn, handle_wait_until(t, ol, w))
        }

        CombatState::Win(_) => {
            // ignore
            (turn, None)
        }
    }
}
