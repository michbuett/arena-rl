use std::collections::HashMap;
use std::time::Instant;

use specs::prelude::*;

use super::types::*;
use crate::components::*;
use crate::core::ai::determine_actor_action;
use crate::core::ai::possible_player_actions;
use crate::core::*;

const ENEMY_SPAWN_POS: [(u8, u8); 12] = [
    (1, 5),
    (1, 6),
    (1, 7),
    (6, 0),
    (7, 0),
    (8, 0),
    (6, 12),
    (7, 12),
    (8, 12),
    (13, 5),
    (13, 6),
    (13, 7),
];

pub fn init_combat_data<'a, 'b>(
    game_objects: Vec<GameObject>,
    teams: Vec<Team>,
    generator: ObjectGenerator,
    texture_map: TextureMap,
) -> CombatData<'a, 'b> {
    let dispatcher = DispatcherBuilder::new()
        .with(FxSystem, "FxSystem", &[])
        .with(MovementAnimationSystem, "MovementAnimationSystem", &[])
        .with(FadeAnimationSystem, "FadeAnimatonSystem", &[])
        .with(ScaleAnimationSystem, "ScaleAnimationSystem", &[])
        .with(HoverAnimationSystem, "HoverAnimationSystem", &[])
        .with(EndOfLiveSystem, "EOL", &[])
        .with(DelayedSpawnSystem, "DelayedSpawnSystem", &[])
        .build();

    let mut world = World::new();
    let map = dummy();

    register(&mut world);

    world.insert(map);
    world.insert(generator);
    world.insert(texture_map);

    CombatData::new(CombatState::Init(game_objects), world, dispatcher, teams)
}

/// Steps the game one tick forward using the given user input
pub fn step<'a, 'b>(g: CombatData<'a, 'b>, i: &Option<UserInput>) -> CombatData<'a, 'b> {
    let step_result = perform_step(g.get_turn(), g.get_state(), g.get_world(), i);
    g.step(step_result)
}

fn find_active_actor(world: &CoreWorld) -> Option<ID> {
    world.find_actor(|a| a.active).map(|a| a.id)
}

fn find_actor_ready_for_activation(turn: &TurnState, world: &CoreWorld) -> Vec<(ID, MapPos, bool)> {
    // let mut result = vec![];
    // let mut min_so_far = 15;

    // for go in world.game_objects() {
    //     if let GameObject::Actor(a) = go {
    //         if !a.activations.is_empty() {}
    //     }
    // }

    let candidates = world
        .game_objects()
        .filter_map(|go| {
            if let GameObject::Actor(a) = go {
                if !a.activations.is_empty() {
                    return Some((a.id, a.pos, a.is_pc(), a.team.id, a.activations[0].value));
                }
            }
            None
        })
        .collect::<Vec<_>>();

    if candidates.is_empty() {
        return vec![];
    }

    let min_card_value = candidates.iter().map(|(_, _, _, _, cv)| *cv).min().unwrap();
    let candidates = candidates
        .iter()
        .filter_map(|(actor_id, pos, is_pc, team_id, cv)| {
            if *cv > min_card_value {
                None
            } else {
                Some((*actor_id, *pos, *is_pc, *team_id))
            }
        })
        .collect::<Vec<_>>();

    let priority_team = candidates
        .iter()
        .map(|(_, _, _, team_id)| *team_id)
        .min_by(|team_a, team_b| turn.cmp_team_by_priority(*team_a, *team_b))
        // unwrap is save because the list of candidates cannot be empty at this point
        .unwrap();

    // final filter: remove all actors which are not part of the priority team
    candidates
        .iter()
        .filter(|(_, _, _, team_id)| *team_id == priority_team)
        .map(|(actor_id, pos, is_pc, _)| (*actor_id, MapPos::from_world_pos(*pos), *is_pc))
        .collect::<Vec<_>>()
}

// fn next_ready_entity(world: &World, active_team: &Team) -> Option<(Entity, Actor)> {
//     let (entities, actors): (Entities, ReadStorage<GameObjectCmp>) = world.system_data();

//     for (e, o) in (&entities, &actors).join() {
//         if let GameObject::Actor(actor) = o.0.clone() {
//             if &actor.team == active_team && actor.can_activate() {
//                 return Some((e, actor));
//             }
//         }
//     }

//     None
// }

fn handle_wait_until(t: &Instant, remaining_actions: &Vec<PlayerAction>) -> StepResult {
    // fn handle_wait_until(t: &Instant, remaining_actions: &Vec<(ID, Act)>) -> StepResult {
    if *t > Instant::now() {
        // now is not the time!
        // => do nothing and wait some more
        return StepResult::new();
    }

    StepResult::new().switch_state(CombatState::ResolveAction(remaining_actions.to_vec()))

    // if let Some((entity_action, tail)) = ol.split_first() {
    //     // wait time is up but there are more action queued up
    //     // => continue with next action in queue
    //     StepResult::new().switch_state(CombatState::ResolveAction(
    //         entity_action.clone(),
    //         tail.to_vec(),
    //     ))
    // } else {
    //     // wait time is up and no further reactions to handle
    //     // => continue with next actor
    //     StepResult::new().switch_state(CombatState::FindActor())
    // }
}

fn handle_wait_for_user_input(
    ctxt: &InputContext,
    selected_pos: &Option<SelectedPos>,
    i: &Option<UserInput>,
    td: &TurnState,
    w: &CoreWorld,
) -> StepResult {
    if let Some(i) = i {
        println!("\n[DEBUG] handle user input {:?}", i);
    }
    match i {
        Some(UserInput::SelectWorldPos(pos)) => {
            // User did just select another map position
            // => collect all details to this position so the UI can present these details
            //    but stay in the current game state and input context and wait for another
            //    input that progresses the game
            return StepResult::new().switch_state(CombatState::WaitForUserInput(
                ctxt.clone(),
                Some(SelectedPos {
                    pos: *pos,
                    objects: find_objects_at(*pos, &w),
                }),
            ));
        }

        Some(UserInput::SelectActivationCard(idx)) => {
            if let InputContext::ActivateActor {
                hand,
                possible_actors,
                ..
            } = ctxt
            {
                return StepResult::new().switch_state(CombatState::WaitForUserInput(
                    InputContext::ActivateActor {
                        hand: hand.clone(),
                        possible_actors: possible_actors.clone(),
                        selected_card_idx: Some(*idx),
                    },
                    selected_pos.as_ref().cloned(),
                ));
            }
        }

        Some(UserInput::AssigneActivation(actor_id, card_idx)) => {
            let mut team_data = td.get_active_team().clone();
            let card = team_data.hand.remove(*card_idx);

            return StepResult::new().modify_team(team_data).switch_state(
                CombatState::ResolveAction(vec![PlayerAction::AssigneActivation(*actor_id, card)]),
            );
        }

        Some(UserInput::SelectPlayerAction(action)) => {
            // user has selected an action
            // => resolve that action
            return StepResult::new()
                .switch_state(CombatState::ResolveAction(vec![action.clone()]));
        }

        // no user input
        // => we wait some more
        _ => {}
    }

    StepResult::new()
}

// fn handle_wait_for_user_action(
//     _a: &Actor,
//     ctxt: &Option<InputContext>,
//     i: &Option<UserInput>,
//     _w: &CoreWorld,
// ) -> StepResult {
//     match i {
//         Some(UserInput::SelectWorldPos(_pos)) => {
//             // user tries to select a new world pos to get new options
//             // => if if the current context allows to change the selected
//             //    world pos (e.g. it is not allowed to switch when resolving
//             //    a combat)
//             match ctxt {
//                 _ => StepResult::new(),
//                 // Some(InputContext::SelectActionAt { options, .. }) => {
//                 //     let input_ctxt = Some(InputContext::SelectActionAt {
//                 //         selected_pos: *pos,
//                 //         objects_at_selected_pos: find_objects_at(*pos, &w),
//                 //         options: options.clone(),
//                 //     });

//                 //     StepResult::new()
//                 //         .switch_state(CombatState::WaitForUserAction(a.clone(), input_ctxt))
//                 // }

//                 // None => {
//                 //     let input_ctxt = Some(InputContext::SelectActionAt {
//                 //         selected_pos: *pos,
//                 //         objects_at_selected_pos: find_objects_at(*pos, &w),
//                 //         options: possible_player_actions(a, &w),
//                 //     });

//                 //     StepResult::new()
//                 //         .switch_state(CombatState::WaitForUserAction(a.clone(), input_ctxt))
//                 // } // _ => StepResult::new(),
//             }
//         }

//         Some(UserInput::SelectPlayerAction(action)) => {
//             // user has selected an action
//             // => resolve that action
//             StepResult::new().switch_state(CombatState::ResolveAction(vec![action.clone()]))
//         }

//         // no user input
//         // => we wait some more
//         _ => StepResult::new(),
//     }
// }

fn find_objects_at(mpos: MapPos, world: &CoreWorld) -> Vec<GameObject> {
    let mut result = Vec::new();

    for o in world.game_objects() {
        if mpos == MapPos::from_world_pos(o.pos()) {
            result.push(o.clone());
        }
    }

    result
}

fn perform_step<'a, 'b>(
    turn: &TurnState,
    current_state: &CombatState,
    w: &World,
    user_input: &Option<UserInput>,
) -> StepResult {
    match current_state {
        CombatState::Init(game_objects) => handle_init(game_objects, w, turn),

        CombatState::StartTurn() => handle_start_turn(&CoreWorld::new(w), turn),

        CombatState::FindActor() => handle_find_actor(turn, &CoreWorld::new(w)),

        CombatState::AdvanceGame() => handle_advance_game(turn),

        CombatState::ResolveAction(actions) => handle_resolve_action(turn, actions, w),

        CombatState::SelectInitiative() => handle_select_initiative(turn, &CoreWorld::new(w)),

        CombatState::SelectPlayerAction(id) => handle_select_player_action(*id, CoreWorld::new(w)),

        CombatState::WaitUntil(time, actions) => handle_wait_until(time, actions),

        // CombatState::WaitForUserAction(a, ctxt) => {
        //     handle_wait_for_user_action(a, ctxt, user_input, &CoreWorld::new(w))
        // }
        CombatState::WaitForUserInput(ctxt, selected_pos) => {
            handle_wait_for_user_input(ctxt, selected_pos, user_input, turn, &CoreWorld::new(w))
        }
    }
}

fn handle_init(game_objects: &Vec<GameObject>, w: &World, turn: &TurnState) -> StepResult {
    for o in game_objects {
        insert_game_object_components(o.clone(), w);
    }

    if let Some(TeamData {
        team: enemy_team, ..
    }) = turn.teams.iter().find(|td| !td.team.is_pc)
    {
        spawn_obstacles(w);
        spawn_enemies(0, w, enemy_team.clone());
    }

    StepResult::new().switch_state(CombatState::StartTurn())
}

// fn next_state<'a, 'b>(
//     round: u64,
//     state: &CombatState,
//     active_team_idx: usize,
//     teams: &Vec<Team>,
//     i: &Option<UserInput>,
//     w: &World,
//     score: u64,
// ) -> (u64, usize, Option<CombatState>, Option<DisplayStr>, u64) {
//     match state {
//         // CombatState::Init(game_objects) => {
//         //     for o in game_objects {
//         //         insert_game_object_components(o.clone(), w);
//         //     }

//         //     spawn_obstacles(w);
//         //     spawn_enemies(0, w);

//         //     (
//         //         round,
//         //         active_team_idx,
//         //         Some(CombatState::StartTurn()),
//         //         None,
//         //         score,
//         //     )
//         // }

//         // CombatState::StartTurn() => {
//         //     let mut entity_actions: Vec<EntityAction> = Vec::new();
//         //     let (entities, objects): (Entities, ReadStorage<GameObjectCmp>) = w.system_data();
//         //     let active_team: &Team = teams.get(active_team_idx).unwrap();

//         //     // first run all pending action (e.g. an attack or charge action)
//         //     // for (e, GameObjectCmp(o)) in (&entities, &objects).join() {
//         //     //     if let GameObject::Actor(a) = o {
//         //     //         if &a.team == active_team {
//         //     //             if a.pending_action.is_some() {
//         //     //                 let prepared_act = a.pending_action.as_ref().cloned().unwrap();

//         //     //                 if a.is_pc() {
//         //     //                     let input_cxt = InputContext::TriggerPreparedAction(prepared_act);
//         //     //                     let next_state = CombatState::WaitForUserAction((e, a.clone()), Some(input_cxt));

//         //     //                     return (round, active_team_idx, Some(next_state), None, score)
//         //     //                 } else {
//         //     //                     // TODO handle A.I.
//         //     //                 }
//         //     //             }

//         //     //             // entity_actions.push((
//         //     //             //     e,
//         //     //             //     a.clone(),
//         //     //             //     Act::new(Action::ResolvePendingActions()),
//         //     //             // ));
//         //     //         }
//         //     //     }
//         //     // }

//         //     // afterwards init a new turn for each actor of the current team
//         //     for (e, GameObjectCmp(o)) in (&entities, &objects).join() {
//         //         if let GameObject::Actor(a) = o {
//         //             if &a.team == active_team {
//         //                 entity_actions.push((e, a.clone(), Act::new(Action::StartTurn())));
//         //             }
//         //         }
//         //     }

//         //     if let Some((entity_action, tail)) = entity_actions.split_first() {
//         //         // wait time is up but there are more action queued up
//         //         // => continue with next action in queue
//         //         (
//         //             round,
//         //             active_team_idx,
//         //             Some(CombatState::ResolveAction(
//         //                 entity_action.clone(),
//         //                 tail.to_vec(),
//         //             )),
//         //             None,
//         //             score,
//         //         )
//         //     } else {
//         //         // wait time is up and no further reactions to handle
//         //         // => continue with next actor
//         //         (
//         //             round,
//         //             active_team_idx,
//         //             Some(CombatState::FindActor()),
//         //             None,
//         //             score,
//         //         )
//         //     }
//         // }
//         // CombatState::FindActor() => {
//         //     // TODO handle WIN/LOSE condition
//         //     // if let Some(team) = find_winning_team(w) {
//         //     //     if team == TEAM_CPU {
//         //     //         return (round, Some(CombatState::Win(team)));
//         //     //     }
//         //     // }

//         //     if let Some(ea) = find_active_actor(w) {
//         //         // there is an active actor
//         //         // -> check if it can do some action
//         //         return (
//         //             round,
//         //             active_team_idx,
//         //             Some(CombatState::SelectAction(ea)),
//         //             None,
//         //             score,
//         //         );
//         //     }

//         //     let active_team: &Team = teams.get(active_team_idx).unwrap();
//         //     if let Some(ea) = next_ready_entity(w, active_team) {
//         //         let id = ea.1.id;
//         //         let next_state = CombatState::ResolveAction(vec![(ea.0, ea.1, Act::activate(id))]);

//         //         (round, active_team_idx, Some(next_state), None, score)
//         //     } else {
//         //         // there are no more entities with a turn left...
//         //         if active_team_idx < teams.len() - 1 {
//         //             // ... then continue with next team
//         //             (
//         //                 round,
//         //                 active_team_idx + 1,
//         //                 Some(CombatState::StartTurn()),
//         //                 None,
//         //                 score,
//         //             )
//         //         } else {
//         //             // ... or start a new round beginning with the first team
//         //             let new_round = round + 1;
//         //             if new_round % 5 == 0 {
//         //                 spawn_enemies(new_round / 5, w);
//         //             }

//         //             (round + 1, 0, Some(CombatState::StartTurn()), None, score)
//         //         }
//         //     }
//         // }
//         // CombatState::SelectAction(ea) => {
//         //     if ea.1.is_pc() {
//         //         // the next ready actor is a player controlled entity
//         //         // => wait for user input;
//         //         //    So far we have no context for the input (e.g. selected
//         //         //    world position, ...) but we can default to preselecting
//         //         //    the actors position. This will reduce the number of clicks
//         //         //    for some actions (eg resting) while not increasing it for
//         //         //    others.
//         //         let pos = MapPos::from_world_pos(ea.1.pos);
//         //         let objects = find_objects_at(pos, &w);
//         //         let actions = actions_at(&ea.1, pos.to_world_pos(), CoreWorld::new(&w));
//         //         let input_ctxt = InputContext::SelectedArea(pos, objects, actions);

//         //         (
//         //             round,
//         //             active_team_idx,
//         //             Some(CombatState::WaitForUserAction(ea.clone(), Some(input_ctxt))),
//         //             None,
//         //             score,
//         //         )
//         //     } else {
//         //         // the next ready actor is a player controlled entity
//         //         // => let the AI compute an action and resolve it
//         //         //    so far we have no reactions
//         //         let act = action(&ea.1, CoreWorld::new(w));

//         //         (
//         //             round,
//         //             active_team_idx,
//         //             Some(CombatState::ResolveAction(vec![(ea.1.id, act)])),
//         //             None,
//         //             score,
//         //         )
//         //     }
//         // }

//         // CombatState::ResolveAction(entity_action, remaining_actions) => {
//         //     let old_score = score;
//         //     let (_, actor, a) = entity_action;
//         //     let ActionResult {
//         //         changes,
//         //         fx_seq,
//         //         log,
//         //         score,
//         //     } = act(actor.id, a.clone(), CoreWorld::new(w));
//         //     let mut wait_until = Instant::now();

//         //     for c in changes {
//         //         match c {
//         //             Change::Update(e, o) => update_components(e, o, w),
//         //             Change::Insert(o) => insert_game_object_components(o, w),
//         //             Change::Remove(e) => remove_components(e, w),
//         //         }
//         //     }

//         //     for fx in fx_seq.into_fx_vec(Instant::now()).drain(..) {
//         //         if wait_until < fx.ends_at() {
//         //             wait_until = fx.ends_at();
//         //         }

//         //         fx.run(w);
//         //     }

//         //     (
//         //         round,
//         //         active_team_idx,
//         //         Some(CombatState::WaitUntil(
//         //             wait_until,
//         //             remaining_actions.to_vec(),
//         //         )),
//         //         log,
//         //         old_score + score,
//         //     )
//         // }

//         // CombatState::WaitForUserAction(e, ctxt) => (
//         //     round,
//         //     active_team_idx,
//         //     handle_wait_for_user_action(&e, &ctxt, i, w),
//         //     None,
//         //     score,
//         // ),

//         // CombatState::WaitUntil(t, ol) => (
//         //     round,
//         //     active_team_idx,
//         //     handle_wait_until(t, ol),
//         //     None,
//         //     score,
//         // ),
//         _ => (round, active_team_idx, None, None, score),
//     }
// }

fn spawn_enemies(wave: u64, w: &World, team: Team) {
    let generator: Read<ObjectGenerator> = w.system_data();
    let wave = generator.generate_enemy_wave(wave);

    for (pos_idx, actor_type) in wave {
        let (x, y) = ENEMY_SPAWN_POS[pos_idx as usize];
        let pos = WorldPos::new(x as f32, y as f32, 0.0);
        let enemy = generator.generate_enemy_by_type(pos, team.clone(), actor_type);

        insert_game_object_components(GameObject::Actor(enemy), w);
    }
}

fn handle_find_actor(turn: &TurnState, world: &CoreWorld) -> StepResult {
    if let Some(id) = find_active_actor(world) {
        // there already is an active (activated) actor
        // -> let it select and perform its action
        // (NOTE: this happens after performing the ActivatActor action)
        StepResult::new().switch_state(CombatState::SelectPlayerAction(id))
    } else {
        let candidates = find_actor_ready_for_activation(turn, world);
        if candidates.len() == 1 {
            // exactly one candidate
            // => just activate the actor
            let next_state =
                CombatState::ResolveAction(vec![PlayerAction::ActivateActor(candidates[0].0)]);
            StepResult::new().switch_state(next_state)
        } else if candidates.len() > 1 {
            // more than one possible candidate
            // => let the user (or AI) choose which one to activate
            let is_pc = candidates.first().unwrap().2;
            if is_pc {
                let selected_pos = candidates[0].1;
                let options = candidates
                    .iter()
                    .map(|(actor_id, pos, _)| (*pos, vec![PlayerAction::ActivateActor(*actor_id)]))
                    .collect::<HashMap<_, _>>();

                StepResult::new().switch_state(CombatState::WaitForUserInput(
                    InputContext::SelectAction { options },
                    Some(SelectedPos {
                        pos: selected_pos,
                        objects: find_objects_at(selected_pos, world),
                    }),
                ))
            } else {
                // just activate the first one
                StepResult::new().switch_state(CombatState::ResolveAction(vec![
                    PlayerAction::ActivateActor(candidates[0].0),
                ]))
            }
        } else {
            // no more candidates
            // => progress to the next game phase
            StepResult::new().switch_state(CombatState::AdvanceGame())
        }
    }
}

fn handle_advance_game(turn: &TurnState) -> StepResult {
    let current_turn_number = turn.turn_number;
    let new_game_phase = turn.clone().step();
    let start_new_turn = new_game_phase.turn_number > current_turn_number;
    let result = StepResult::new().advance_game(new_game_phase);

    if start_new_turn {
        result.switch_state(CombatState::StartTurn())
    } else {
        result.switch_state(CombatState::FindActor())
    }
}

fn handle_start_turn(world: &CoreWorld, turn: &TurnState) -> StepResult {
    let mut actions: Vec<PlayerAction> = Vec::new();
    let mut actor_per_team: HashMap<TeamId, u8> = HashMap::new();
    let mut step_result = StepResult::new();

    for o in world.game_objects() {
        if let GameObject::Actor(a) = o {
            actions.push(PlayerAction::StartTurn(a.id));

            let curr_amout = actor_per_team.get(&a.team.id).copied().unwrap_or(0);
            actor_per_team.insert(a.team.id, curr_amout + 1);
        }
    }

    for (team_id, num_actors) in actor_per_team.iter() {
        step_result =
            step_result.modify_team(turn.get_team(*team_id).clone().start_new_turn(*num_actors))
    }

    step_result.switch_state(CombatState::ResolveAction(actions))
}

fn handle_select_initiative(turn: &TurnState, world: &CoreWorld) -> StepResult {
    // let mut turn = turn.clone();
    let team_data = turn.get_active_team();

    if team_data.ready || (team_data.team.is_pc && team_data.hand.is_empty()) {
        // there are no more activations for this team
        // (eithere by choise or because of lack of resources)
        // => progress to next game state (either next activate next team or
        //     go to next combat phase)
        return StepResult::new().switch_state(CombatState::AdvanceGame());
    }

    if team_data.team.is_pc {
        // The team is controlled by the a human player
        // => wait for the user's input
        StepResult::new().switch_state(CombatState::WaitForUserInput(
            InputContext::ActivateActor {
                hand: team_data.hand.clone(),
                possible_actors: activ_team_members(&team_data.team, world),
                selected_card_idx: None,
            },
            None,
        ))
    } else {
        // The team is controlled by the A.I.
        // distibute the activations randomly
        // TODO: figure out a need mechanic for ai activation (one that is
        //       more challenging and allows e.g. for multiple activations)
        let mut actions: Vec<PlayerAction> = Vec::new();
        let mut team_data = team_data.clone();

        for o in world.game_objects() {
            if let GameObject::Actor(a) = o {
                if team_data.team.is_member(a) {
                    let card = team_data.deck.deal();
                    actions.push(PlayerAction::AssigneActivation(a.id, card));
                }
            }
        }

        team_data.ready = true;
        StepResult::new()
            .modify_team(team_data)
            .switch_state(CombatState::ResolveAction(actions))
    }
}

fn handle_select_player_action(id: ID, w: CoreWorld) -> StepResult {
    let a = w.get_actor(id);
    if a.is_none() {
        return StepResult::new().switch_state(CombatState::FindActor());
    }

    let actor = a.unwrap().clone();
    if actor.is_pc() {
        // the next ready actor is a player controlled entity
        // => wait for user input;
        let options = possible_player_actions(&actor, &w);
        if let Some(action) = single_option(&options) {
            StepResult::new().switch_state(CombatState::ResolveAction(vec![action]))
        } else {
            let selected_pos = MapPos::from_world_pos(actor.pos);

            StepResult::new().switch_state(CombatState::WaitForUserInput(
                InputContext::SelectAction {
                    // active_actor: actor.clone(),
                    options: possible_player_actions(&actor, &w),
                },
                Some(SelectedPos {
                    pos: selected_pos,
                    objects: find_objects_at(selected_pos, &w),
                }),
            ))
        }
    } else {
        // the next ready actor is a player controlled entity
        // => let the AI compute an action and resolve it
        //    so far we have no reactions
        if let Some(action) = determine_actor_action(&actor, w) {
            StepResult::new().switch_state(CombatState::ResolveAction(vec![action]))
        } else {
            StepResult::new().switch_state(CombatState::FindActor())
        }
    }
}

// fn handle_select_action_to_execute(id: ID, w: CoreWorld) -> StepResult {
//     let a = w.get_actor(id);
//     if a.is_none() {
//         return StepResult::new().switch_state(CombatState::FindActor());
//     }

//     let actor = a.unwrap().clone();
//     if actor.is_pc() {
//         // the next ready actor is a player controlled entity
//         // => wait for user input;
//         let prepared_act = actor
//             .pending_action
//             .as_ref()
//             .expect("No action prepared! This should not happen!")
//             .clone();

//         let input_ctxt = Some(InputContext::TriggerPreparedAction(prepared_act));

//         StepResult::new().switch_state(CombatState::WaitForUserAction(actor.clone(), input_ctxt))
//     } else {
//         // the next ready actor is a player controlled entity
//         // => let the AI compute an action and resolve it
//         //    so far we have no reactions
//         let action = PlayerAction::SaveEffort(actor.id, "TODO: AI actions".to_string());
//         // let act = action(&actor, w);

//         StepResult::new().switch_state(CombatState::ResolveAction(vec![action]))
//         // StepResult::new().switch_state(CombatState::ResolveAction(vec![(actor.id, act)]))
//     }
// }

fn handle_resolve_action(turn: &TurnState, actions: &Vec<PlayerAction>, w: &World) -> StepResult {
    // fn handle_resolve_action(actions: &Vec<(ID, Act)>, w: &World) -> StepResult {
    if actions.is_empty() {
        return match turn.phase {
            CombatPhase::Planning => {
                StepResult::new().switch_state(CombatState::SelectInitiative())
            }
            CombatPhase::Action => StepResult::new().switch_state(CombatState::FindActor()),
        };
    }

    let mut remaining_actions = actions.to_vec();
    let mut wait_until = Instant::now();
    let action = remaining_actions.pop().unwrap();
    // let (actor_id, a) = remaining_actions.pop().unwrap();
    let ActionResult {
        changes,
        fx_seq,
        log,
        score,
    } = run_player_action(action, CoreWorld::new(w));
    // } = act(actor_id, a, CoreWorld::new(w));

    for c in changes {
        match c {
            Change::Update(e, o) => update_components(e, o, w),
            Change::Insert(o) => insert_game_object_components(o, w),
            Change::Remove(e) => remove_components(e, w),
        }
    }

    for fx in fx_seq.into_fx_vec(Instant::now()).drain(..) {
        if wait_until < fx.ends_at() {
            wait_until = fx.ends_at();
        }

        fx.run(w);
    }

    StepResult::new()
        .add_score(score)
        .append_log(log)
        .switch_state(CombatState::WaitUntil(wait_until, remaining_actions))
}

fn spawn_obstacles(w: &World) {
    let (_map, texture_map, updater, entities): (
        Read<Map>,
        Read<TextureMap>,
        Read<LazyUpdate>,
        Entities,
    ) = w.system_data();
    let pos = vec![
        (5.0, 6.0),
        (7.0, 4.0),
        (5.0, 9.0),
        (10.0, 4.0),
        (8.0, 9.0),
        (10.0, 7.0),
    ];
    let sprite = texture_map.get("wall-1").unwrap();

    for (x, y) in pos.iter() {
        updater
            .create_entity(&entities)
            .with(Sprites::new(vec![sprite.clone()]))
            .with(Position(WorldPos::new(*x, *y, 0.0)))
            .with(ZLayerGameObject)
            .with(ObstacleCmp {
                movement: (Some(Obstacle::Blocker), Some(Obstacle::Blocker), None),
                reach: Some(Hitbox::new_pillar()),
            })
            .build();
    }
}

fn next_ready_actor(turn: &TurnState, world: &World) -> Option<ID> {
    CoreWorld::new(world)
        .find_actor(|a| match turn.phase {
            // CombatPhase::Plan => {
            //     a.state != ReadyState::Done
            //         && a.prepared_action.is_none()
            //         && turn.active_team().is_member(a)
            // }

            // CombatPhase::React => !turn.active_team().is_member(a) && a.prepared_action.is_none(),

            // CombatPhase::Resolve => a.prepared_action.is_some(),
            _ => false, // TODO implement this using new initiative mechanic
        })
        .map(|a| {
            // println!("[DEBUG] next_ready_actor \n  * turn => {:?}\n  * actor => {} {:?}", turn, a.name, a);
            a.id
        })
}

fn single_option(options: &HashMap<MapPos, Vec<PlayerAction>>) -> Option<PlayerAction> {
    if options.len() != 1 {
        return None;
    }

    let actions = options.values().next().unwrap();
    if actions.len() != 1 {
        return None;
    }

    Some(actions.get(0).cloned().unwrap())
}

fn activ_team_members(active_team: &Team, w: &CoreWorld) -> HashMap<MapPos, (ID, u8)> {
    let mut result = HashMap::new();
    for go in w.game_objects() {
        if let GameObject::Actor(a) = go {
            if active_team.is_member(a) {
                result.insert(
                    MapPos::from_world_pos(a.pos),
                    (a.id, a.max_available_activation_val()),
                );
            }
        }
    }
    result
}
