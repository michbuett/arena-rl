use std::collections::HashMap;
use std::time::Instant;

use specs::prelude::*;

use super::types::*;
use crate::components::*;
use crate::core::ai::determine_actor_action;
use crate::core::ai::possible_player_actions;
use crate::core::*;

pub fn init_combat_data<'a, 'b>(
    game_objects: Vec<GameObject>,
    teams: Vec<Team>,
    generator: ObjectGenerator,
    texture_map: TextureMap,
) -> CombatData<'a, 'b> {
    let dispatcher = DispatcherBuilder::new()
        // .with(SpriteSystem, "SpriteSystem", &[])
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

fn perform_step<'a, 'b>(
    turn: &TurnState,
    current_state: &CombatState,
    w: &World,
    user_input: &Option<UserInput>,
) -> StepResult {
    // match current_state {
    //     CombatState::WaitUntil(..) => {}
    //     CombatState::WaitForUserInput(..) => {}
    //     _ => {
    //         println!("[DEBUG perform_step - IN]");
    //         println!("  - current state: {:?}", current_state);
    //     }
    // }

    match current_state {
        CombatState::Init(game_objects) => handle_init(game_objects, w),

        CombatState::StartTurn() => handle_start_turn(CoreWorld::new(w), turn),

        CombatState::FindActor() => handle_find_actor(turn, &CoreWorld::new(w)),

        CombatState::AdvanceGame() => handle_advance_game(turn),

        CombatState::ResolveAction(actions) => handle_resolve_action(actions, w),

        CombatState::AssignActivations() => handle_assign_activations(turn, &CoreWorld::new(w)),

        CombatState::SelectPlayerAction(id) => handle_select_player_action(*id, CoreWorld::new(w)),

        CombatState::WaitUntil(time, actions) => handle_wait_until(time, actions),

        CombatState::WaitForUserInput(ctxt, selected_pos) => {
            handle_wait_for_user_input(ctxt, selected_pos, user_input, &CoreWorld::new(w))
        }
    }
}

fn find_active_actor(world: &CoreWorld) -> Option<ID> {
    world.find_actor(|a| a.active).map(|a| a.id)
}

fn find_actor_ready_for_activation(turn: &TurnState, world: &CoreWorld) -> Vec<(ID, MapPos, bool)> {
    let candidates = world
        .game_objects()
        .filter_map(|go| {
            if let GameObject::Actor(a) = go {
                if !a.activations.is_empty() {
                    return Some((a.id, a.pos, a.is_pc(), a.team, a.initiative()));
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

fn handle_wait_until(t: &Instant, remaining_actions: &Vec<Action>) -> StepResult {
    // fn handle_wait_until(t: &Instant, remaining_actions: &Vec<(ID, Act)>) -> StepResult {
    if *t > Instant::now() {
        // now is not the time!
        // => do nothing and wait some more
        return StepResult::new();
    }

    StepResult::new().switch_state(CombatState::ResolveAction(remaining_actions.to_vec()))
}

fn handle_wait_for_user_input(
    ctxt: &InputContext,
    selected_pos: &Option<SelectedPos>,
    i: &Option<UserInput>,
    w: &CoreWorld,
) -> StepResult {
    // if let Some(i) = i {
    //     println!("\n[DEBUG] handle user input {:?}", i);
    // }
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
                team,
                hand,
                possible_actors,
                ..
            } = ctxt
            {
                return StepResult::new().switch_state(CombatState::WaitForUserInput(
                    InputContext::ActivateActor {
                        team: *team,
                        hand: hand.clone(),
                        possible_actors: possible_actors.clone(),
                        selected_card_idx: Some(*idx),
                    },
                    selected_pos.as_ref().cloned(),
                ));
            }
        }

        Some(UserInput::BoostActivation(actor_id, team_id, card)) => {
            // let mut team_data = w.teams().get(team_id).clone();
            //     let card = team_data.hand.remove(*card_idx);

            return StepResult::new()
                // .modify_team(team_data)
                .remove_card_from_hand(*team_id, *card)
                .switch_state(CombatState::ResolveAction(vec![Action::BoostActivation(
                    *actor_id, *card,
                )]));
        }

        Some(UserInput::AssigneActivationDone(team_id)) => {
            let mut team_data = w.teams().get(team_id).clone();
            team_data.ready = true;

            return StepResult::new()
                .modify_team(team_data)
                .switch_state(CombatState::AssignActivations());
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

fn find_objects_at(mpos: MapPos, world: &CoreWorld) -> Vec<GameObject> {
    let mut result = Vec::new();

    for o in world.game_objects() {
        if mpos == MapPos::from_world_pos(o.pos()) {
            result.push(o.clone());
        }
    }

    result
}

fn handle_init(game_objects: &Vec<GameObject>, w: &World) -> StepResult {
    let (texture_map, updater, entities): (Read<TextureMap>, Read<LazyUpdate>, Entities) =
        w.system_data();

    for o in game_objects {
        let e = insert_game_object(o, &entities, &updater);
        update_game_object(e, o, &texture_map, &updater);
        // insert_game_object_components(o.clone(), w);
    }

    spawn_obstacles(w);

    StepResult::new().switch_state(CombatState::StartTurn())
}

fn handle_find_actor(turn: &TurnState, world: &CoreWorld) -> StepResult {
    if let CombatPhase::Planning = turn.phase {
        // this can happen after advancing the game or resolving an action
        return StepResult::new().switch_state(CombatState::AssignActivations());
    }

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
                CombatState::ResolveAction(vec![Action::ActivateActor(candidates[0].0)]);
            StepResult::new().switch_state(next_state)
        } else if candidates.len() > 1 {
            // more than one possible candidate
            // => let the user (or AI) choose which one to activate
            let is_pc = candidates.first().unwrap().2;
            if is_pc {
                let selected_pos = candidates[0].1;
                let options = candidates
                    .iter()
                    .map(|(actor_id, pos, _)| (*pos, vec![Action::ActivateActor(*actor_id)]))
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
                    Action::ActivateActor(candidates[0].0),
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

fn handle_start_turn(world: CoreWorld, turn: &TurnState) -> StepResult {
    let mut actions: Vec<Action> = Vec::new();
    let mut actor_per_team: HashMap<TeamId, u8> = HashMap::new();
    let mut step_result = StepResult::new();

    for (team, pos, template) in turn.reinforcements() {
        let actor = world.generate_enemy(pos, team, template);
        let curr_amout = actor_per_team.get(&actor.team).copied().unwrap_or(0);
        let actor_id = actor.id;

        actor_per_team.insert(actor.team, curr_amout + actor.num_activation());
        actions.push(Action::SpawnActor { actor });
        actions.push(Action::StartTurn(actor_id));
    }

    for o in world.game_objects() {
        if let GameObject::Actor(a) = o {
            actions.push(Action::StartTurn(a.id));

            let curr_amout = actor_per_team.get(&a.team).copied().unwrap_or(0);
            actor_per_team.insert(a.team, curr_amout + a.num_activation());
        }
    }

    let max_activations = actor_per_team.values().copied().max().unwrap_or(0);
    for (team_id, num_activations) in actor_per_team.iter() {
        let num_draws = max_activations - *num_activations;
        step_result = step_result.start_new_turn(*team_id, num_draws);
    }

    step_result.switch_state(CombatState::ResolveAction(actions))
}

fn handle_assign_activations(turn: &TurnState, world: &CoreWorld) -> StepResult {
    let teams = world.teams();
    let active_team = teams.get(&turn.get_active_team().unwrap());
    // unwrap is save since we are only selecting initiativ in planning phase
    // and there is always an activ team in the planning phase

    if active_team.ready {
        // if active_team.ready || (active_team.team.is_pc && active_team.hand.is_empty()) {
        // there are no more activations for this team
        // (eithere by choise or because of lack of resources)
        // => progress to next game state (either next activate next team or
        //     go to next combat phase)
        return StepResult::new().switch_state(CombatState::AdvanceGame());
    }

    if active_team.team.is_pc {
        // The team is controlled by the a human player
        // => wait for the user's to distribute hand
        StepResult::new().switch_state(CombatState::WaitForUserInput(
            InputContext::ActivateActor {
                team: active_team.team.id,
                hand: active_team.hand.clone(),
                possible_actors: activ_team_members(&active_team.team, world),
                selected_card_idx: None,
            },
            None,
        ))
    } else {
        // The team is controlled by the AI
        // => no hand for the AI, so continue with the next step
        let mut team_data = active_team.clone();
        team_data.ready = true;
        StepResult::new().modify_team(team_data)
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
        let action = determine_actor_action(&actor, w);
        StepResult::new().switch_state(CombatState::ResolveAction(vec![action]))
    }
}

fn handle_resolve_action(actions: &Vec<Action>, w: &World) -> StepResult {
    if actions.is_empty() {
        return StepResult::new().switch_state(CombatState::FindActor());
    }

    let mut remaining_actions = actions.to_vec();
    let mut wait_until = Instant::now();
    let action = remaining_actions.remove(0);
    let ActionResult {
        decks,
        fx_seq,
        log,
        score,
    } = run_player_action(action, CoreWorld::new(w));

    let mut result = StepResult::new().add_score(score).append_log(log);

    if let Some(mut draws) = decks {
        for (team_id, deck) in draws.drain() {
            result = result.update_deck(team_id, deck);
        }
    }

    for fx in fx_seq.into_fx_vec(Instant::now()).drain(..) {
        if wait_until < fx.ends_at() {
            wait_until = fx.ends_at();
        }

        fx.run(w);
    }

    result.switch_state(CombatState::WaitUntil(wait_until, remaining_actions))
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

fn single_option(options: &HashMap<MapPos, Vec<Action>>) -> Option<Action> {
    if options.len() != 1 {
        return None;
    }

    let actions = options.values().next().unwrap();
    if actions.len() != 1 {
        return None;
    }

    Some(actions.get(0).cloned().unwrap())
}

fn activ_team_members(active_team: &Team, w: &CoreWorld) -> HashMap<MapPos, ID> {
    let mut team_members = HashMap::new();

    for go in w.game_objects() {
        if let GameObject::Actor(a) = go {
            if a.can_activate() && active_team.is_member(a) {
                team_members.insert(MapPos::from_world_pos(a.pos), a.id);
            }
        }
    }
    team_members
}
