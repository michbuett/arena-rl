use std::time::Instant;

use specs::prelude::*;

use super::super::action::{act, Action};
use super::super::ai::{action, actions_at};
use super::types::*;
use crate::components::*;
use crate::core::*;

// const TEAM_PLAYER: Team = Team("Player", 1, true);
const TEAM_CPU: Team = Team("Computer", 2, false);
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

    CombatData {
        turn: 0,
        active_team_idx: 0,
        teams,
        world,
        dispatcher,
        state: CombatState::Init(game_objects),
        log: vec![],
        score: 0,
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
        score,
    } = g;

    let (next_turn, next_active_team, next_state, log_entry, new_score) =
        next_state(turn, &state, active_team_idx, &teams, i, &world, score);

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
        score: new_score,
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
                let objects = find_objects_at(*pos, &w);
                let actions = actions_at(&e.1, pos.to_world_pos(), CoreWorld::new(&w));
                let ui = InputContext::SelectedArea(*pos, objects, actions);

                Some(CombatState::WaitForUserAction(e.clone(), Some(ui)))
            } else {
                // user tries to select a new area but is not allowed to
                // change it (e.g. when handling an reaction)
                // => ignore the input and wait some more
                None
            }
        }

        Some(UserInput::SelectAction(act)) => {
            // user has selected an action
            // => resolve that action
            Some(CombatState::ResolveAction(
                (e.0.clone(), e.1.clone(), act.clone()),
                Vec::new(),
            ))
        }

        // no user input
        // => we wait some more
        _ => None,
    }
}

fn find_objects_at(mpos: MapPos, world: &World) -> Vec<GameObject> {
    let game_objects: ReadStorage<GameObjectCmp> = world.system_data();
    let mut result = Vec::new();
    // let mpos = MapPos::from_world_pos(*pos);

    for GameObjectCmp(o) in (&game_objects).join() {
        match o {
            GameObject::Actor(a) => {
                if mpos == MapPos::from_world_pos(a.pos) {
                    result.push(o.clone());
                }
            }

            GameObject::Item(item_pos, _) => {
                if mpos == MapPos::from_world_pos(*item_pos) {
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
    score: u64,
) -> (u64, usize, Option<CombatState>, Option<DisplayStr>, u64) {
    match state {
        CombatState::Init(game_objects) => {
            for o in game_objects {
                insert_game_object_components(o.clone(), w);
            }

            spawn_obstacles(w);
            spawn_enemies(0, w);

            (
                round,
                active_team_idx,
                Some(CombatState::StartTurn()),
                None,
                score,
            )
        }

        CombatState::StartTurn() => {
            let mut entity_actions: Vec<EntityAction> = Vec::new();
            let (entities, objects): (Entities, ReadStorage<GameObjectCmp>) = w.system_data();
            let active_team: &Team = teams.get(active_team_idx).unwrap();

            // first run all pending action (e.g. an attack or charge action)
            for (e, GameObjectCmp(o)) in (&entities, &objects).join() {
                if let GameObject::Actor(a) = o {
                    if &a.team == active_team {
                        entity_actions.push((
                            e,
                            a.clone(),
                            Act::new(Action::ResolvePendingActions()),
                        ));
                    }
                }
            }

            // afterwards init a new turn for each actor of the current team
            for (e, GameObjectCmp(o)) in (&entities, &objects).join() {
                if let GameObject::Actor(a) = o {
                    if &a.team == active_team {
                        entity_actions.push((e, a.clone(), Act::new(Action::StartTurn())));
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
                    score,
                )
            } else {
                // wait time is up and no further reactions to handle
                // => continue with next actor
                (
                    round,
                    active_team_idx,
                    Some(CombatState::FindActor()),
                    None,
                    score,
                )
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
                return (
                    round,
                    active_team_idx,
                    Some(CombatState::SelectAction(ea)),
                    None,
                    score,
                );
            }

            let active_team: &Team = teams.get(active_team_idx).unwrap();
            if let Some(ea) = next_ready_entity(w, active_team) {
                let id = ea.1.id;
                let next_state =
                    CombatState::ResolveAction((ea.0, ea.1, Act::activate(id)), vec![]);

                (round, active_team_idx, Some(next_state), None, score)
            } else {
                // there are no more entities with a turn left...
                if active_team_idx < teams.len() - 1 {
                    // ... then continue with next team
                    (
                        round,
                        active_team_idx + 1,
                        Some(CombatState::StartTurn()),
                        None,
                        score,
                    )
                } else {
                    // ... or start a new round beginning with the first team
                    let new_round = round + 1;
                    if new_round % 5 == 0 {
                        spawn_enemies(new_round / 5, w);
                    }

                    (round + 1, 0, Some(CombatState::StartTurn()), None, score)
                }
            }
        }

        CombatState::SelectAction(ea) => {
            if ea.1.is_pc() {
                // the next ready actor is a player controlled entity
                // => wait for user input;
                //    So far we have no context for the input (e.g. selected
                //    world position, ...) but we can default to preselecting
                //    the actors position. This will reduce the number of clicks
                //    for some actions (eg resting) while not increasing it for
                //    others.
                let pos = MapPos::from_world_pos(ea.1.pos);
                let objects = find_objects_at(pos, &w);
                let actions = actions_at(&ea.1, pos.to_world_pos(), CoreWorld::new(&w));
                let input_ctxt = InputContext::SelectedArea(pos, objects, actions);

                (
                    round,
                    active_team_idx,
                    Some(CombatState::WaitForUserAction(ea.clone(), Some(input_ctxt))),
                    None,
                    score,
                )
            } else {
                // the next ready actor is a player controlled entity
                // => let the AI compute an action and resolve it
                //    so far we have no reactions
                let act = action(&ea.1, CoreWorld::new(w));

                (
                    round,
                    active_team_idx,
                    Some(CombatState::ResolveAction(
                        (ea.0, ea.1.clone(), act),
                        Vec::new(),
                    )),
                    None,
                    score,
                )
            }
        }

        CombatState::ResolveAction(entity_action, remaining_actions) => {
            let old_score = score;
            let (_, actor, a) = entity_action;
            let ActionResult {
                changes,
                fx_seq,
                log,
                score,
            } = act(actor.id, a.clone(), CoreWorld::new(w));
            let mut wait_until = Instant::now();

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

            (
                round,
                active_team_idx,
                Some(CombatState::WaitUntil(
                    wait_until,
                    remaining_actions.to_vec(),
                )),
                log,
                old_score + score,
            )
        }

        CombatState::WaitForUserAction(e, ctxt) => (
            round,
            active_team_idx,
            handle_wait_for_user_action(&e, &ctxt, i, w),
            None,
            score,
        ),

        CombatState::WaitUntil(t, ol) => (
            round,
            active_team_idx,
            handle_wait_until(t, ol),
            None,
            score,
        ),
        // CombatState::Win(_) => {
        //     // ignore
        //     (round, active_team_idx, None)
        // }
    }
}

fn spawn_enemies(wave: u64, w: &World) {
    let generator: Read<ObjectGenerator> = w.system_data();
    let wave = generator.generate_enemy_wave(wave);

    for (pos_idx, actor_type) in wave {
        let (x, y) = ENEMY_SPAWN_POS[pos_idx as usize];
        let pos = WorldPos::new(x as f32, y as f32, 0.0);
        let enemy = generator.generate_enemy_by_type(pos, TEAM_CPU, actor_type);

        insert_game_object_components(GameObject::Actor(enemy), w);
    }
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
