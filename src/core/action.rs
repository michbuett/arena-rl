// use std::cmp::max;
use std::time::Duration;

// use specs::prelude::Entity;
use specs::prelude::*;

use crate::components::{Fx, GameObjectCmp};
use crate::core::ai::find_movement_obstacles;
use crate::core::{Map, Tile,MapPos};
// use crate::core::{Tile, WorldPos};

use super::actors::{Actor, AttackOption, CombatResult, Condition, GameObject, Team};

#[derive(Debug, Clone)]
pub enum Action {
    StartTurn(),
    Wait(),
    MoveTo(Tile),
    Activate(Entity),
    MeleeAttack(Entity, AttackOption),
    Charge(Entity, AttackOption),
    EndTurn(Team),
}

impl Action {
    pub fn end_turn(t: Team) -> Act {
        (Self::EndTurn(t), 0)
    }

    // pub fn wait() -> Act {
    //     (Self::Wait(), 1)
    // }

    pub fn recover() -> Act {
        (Self::Wait(), 1)
    }

    pub fn activate(e: Entity) -> Act {
        (Self::Activate(e), 0)
    }

    pub fn move_to(to: Tile) -> Act {
        (Self::MoveTo(to), 0)
    }

    pub fn melee_attack(target: Entity, attack: AttackOption) -> Act {
        (Self::MeleeAttack(target, attack), 1)
    }

    pub fn charge(target: Entity, attack: AttackOption) -> Act {
        (Self::Charge(target, attack), 1)
    }
}

pub enum Change {
    Fx(Fx),
    Update(Entity, GameObject),
    Insert(GameObject),
    Remove(Entity),
}

pub type Act = (Action, u8);
pub type EA = (Entity, Actor);
pub type ActionResult = (Vec<Change>, Duration);

pub fn act((entity, actor, action, delay): (Entity, Actor, Action, u8), w: &World) -> ActionResult {
    if delay > 0 {
        single_update(entity, actor.prepare((action, delay - 1)), 0)
    } else {
        run_action((entity, actor), action, w)
    }
}

pub fn run_action<'a>((entity, actor): EA, action: Action, w: &World) -> ActionResult {
    match action {
        Action::StartTurn() => {
            let (actor, pending_action) = actor.start_next_turn();
            let mut updates = vec![update_actor(entity, actor.clone())];
            let mut wait_time = millis(0);

            if let Some(pending_action) = pending_action {
                let (action, delay) = pending_action;
                let (mut more_updates, more_wait_time) = act((entity, actor, action, delay), w);

                updates.append(&mut more_updates);
                wait_time += more_wait_time;
            }

            (updates, wait_time)
        }

        Action::EndTurn(team) => {
            let mut updates = vec![];
            let (entities, actors): (Entities, ReadStorage<GameObjectCmp>) = w.system_data();

            for (e, o) in (&entities, &actors).join() {
                if let GameObject::Actor(a) = &o.0 {
                    if a.team == team && a.pending_action.is_none() {
                        updates.push(update_actor(e, a.clone().prepare((Action::Wait(), 0))));
                    }
                }
            }

            (updates, millis(0))
        }

        Action::Wait() => no_op(),

        Action::MoveTo(to) => single_update(entity, actor.move_to(to), 200),

        Action::Activate(e) => {
            let storage = w.read_storage::<GameObjectCmp>();
            if let Some(GameObjectCmp(GameObject::Actor(a))) = storage.get(e) {
                (
                    vec![
                        update_actor(entity, actor.clone().deactivate()),
                        update_actor(e, a.clone().activate()),
                    ],
                    millis(0),
                )
            } else {
                no_op()
            }
        }

        Action::MeleeAttack(target_entity, attack) => {
            if let Some(target_actor) = get_actor(target_entity, w) {
                handle_attack((entity, actor), (target_entity, target_actor), attack)
            } else {
                no_op()
            }
        }

        Action::Charge(target_entity, attack) => {
            if let Some(target_actor) = get_actor(target_entity, w) {
                // let attack = actor.melee_attack();
                let from = MapPos::from_world_pos(actor.pos);
                let to = MapPos::from_world_pos(target_actor.pos);
                let steps_needed = from.distance(to) - 1;
                let move_distance: usize = actor.move_distance().into();

                if steps_needed <= 0 || steps_needed > move_distance {
                    // cannot reach opponent
                    // => cancel charge
                    return no_op();
                }
                
                let (map, game_objects): (Read<Map>, ReadStorage<GameObjectCmp>) = w.system_data();
                let obstacles = find_movement_obstacles(&game_objects).ignore(to);

                if let Some(p) = map.find_straight_path(from, to, &obstacles) {
                    let tile = p[steps_needed - 1];
                    // println!("Charge from {:?} to {:?}", from, to);
                    // println!(" - steps needed: {}", steps_needed);
                    // println!(" - path: {:?}", p);
                    // println!(" - move to: {:?}", tile);
                    let actor = actor.move_to(tile);
                    let mut updates = vec![update_actor(entity, actor.clone())];
                    let (mut combat_updates, delay) = handle_attack((entity, actor), (target_entity, target_actor), attack);

                    updates.append(&mut combat_updates);

                    return (updates, delay);
                }


                // if d >= reach {
                //     return handle_attack(
                //         (entity, actor.move_to(*tile)),
                //         (target_entity, target_actor),
                //         attack,
                //     );
                // } else if reach < d && d <= reach + move_distance {
                //     let obstacles = find_movement_obstacles(&objects).ignore(to);
                //     if let Some(_) = map.find_straight_path(from, to, &obstacles) {
                //     }
                // }

                // if attack.distance.0 <= distance && distance <= attack.distance.1 {
                //     // target already within reaching distance
                //     return handle_attack((entity, actor), (target_entity, target_actor), attack);
                // }

                // if let Some(path) = can_move_towards(&actor, &target_actor, &map, &game_objects) {
                //     let l = max(path.len(), actor.move_distance().into());
                //     if let Some(tile) = path.iter().take(l).last() {
                //         return handle_attack(
                //             (entity, actor.move_to(*tile)),
                //             (target_entity, target_actor),
                //             attack,
                //         );
                //     }
                // }
            }

            return no_op();
        }
    }
}

fn handle_attack<'a>(
    attacker: (Entity, Actor),
    target: (Entity, Actor),
    attack: AttackOption,
) -> ActionResult {
    let fx_pos = target.1.pos.clone();
    let (combat_result, log) = super::actors::combat(attack, attacker.1, target.1);
    let changes = match combat_result {
        CombatResult::Miss(new_actor) => vec![
            update_actor(target.0, new_actor),
            Change::Fx(Fx::text("Miss".to_string(), &fx_pos, 100)),
        ],

        CombatResult::Block() => vec![Change::Fx(Fx::text("Blocked".to_string(), &fx_pos, 100))],

        CombatResult::Hit(condition) => match condition {
            Condition::Alive(new_actor) => vec![
                update_actor(target.0, new_actor),
                Change::Fx(Fx::text("Hit!".to_string(), &fx_pos, 100)),
            ],

            Condition::Dead(pos, corpse) => vec![
                Change::Remove(target.0),
                Change::Insert(GameObject::Item(pos, corpse)),
                Change::Fx(Fx::text("KILL!!!".to_string(), &fx_pos, 100)),
            ],
        },
    };

    println!("\n");
    for entry in log.iter() {
        println!("{}", entry);
    }
    println!("\n");

    (changes, millis(1000))
}

fn millis(ms: u64) -> Duration {
    Duration::from_millis(ms)
}

fn update_actor(e: Entity, a: Actor) -> Change {
    Change::Update(e, GameObject::Actor(a))
}

fn get_actor(e: Entity, w: &World) -> Option<Actor> {
    let game_objects = w.read_storage::<GameObjectCmp>();

    if let Some(GameObjectCmp(GameObject::Actor(a))) = game_objects.get(e) {
        return Some(a.clone());
    }

    None
}

fn no_op() -> ActionResult {
    (vec![], millis(0))
}

fn single_update(e: Entity, a: Actor, ms: u64) -> ActionResult {
    (vec![update_actor(e, a)], millis(ms))
}
