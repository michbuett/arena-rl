mod primitives;

use specs::prelude::*;

use crate::components::*;
use crate::core::*;
use primitives::*;

pub use primitives::{can_attack_melee, can_charge, can_move_towards, find_movement_obstacles};

pub fn action(e: &(Entity, Actor), w: &World) -> (Action, u8) {
    zombi_action(&e, w)
}

fn zombi_action((_, a): &(Entity, Actor), w: &World) -> (Action, u8) {
    let (entities, game_objects, map): (Entities, ReadStorage<GameObjectCmp>, Read<Map>) =
        w.system_data();

    for (te, ta) in find_enemies(a, &entities, &game_objects) {
        if let Some(attack) = can_attack_melee(a, &ta, &map, &game_objects) {
            return Action::melee_attack(te, attack);
        }
        if let Some(attack) = can_charge(a, &ta, &map, &game_objects) {
            return Action::charge(te, attack);
        }

        if a.can_move() {
            if let Some(path) = can_move_towards(a, &ta, &map, &game_objects) {
                return Action::move_to(
                    path.iter()
                        .take(a.move_distance().into())
                        .cloned()
                        .collect(),
                );
            }
        }
    }

    Action::recover()
}

pub fn actions_at(
    (entity, actor): &(Entity, Actor),
    selected_pos: WorldPos,
    world: &World,
) -> Vec<(Action, u8)> {
    let (map, objects): (Read<Map>, ReadStorage<GameObjectCmp>) = world.system_data();
    let mut result = vec![];

    if let Some((other_entity, other_actor)) = find_actor_at(world, &selected_pos) {
        if entity.id() == other_entity.id() {
            for (n, t, d) in actor.ability_self() {
                result.push(Action::use_ability(*entity, n, t, d));
            }
        } else {
            if actor.team == other_actor.team {
                if other_actor.can_activate() {
                    result.push(Action::activate(other_entity));
                }
            } else {
                if let Some(attack) = can_attack_melee(actor, &other_actor, &map, &objects) {
                    result.push(Action::melee_attack(other_entity, attack));
                }
                if let Some(attack) = can_charge(actor, &other_actor, &map, &objects) {
                    result.push(Action::charge(other_entity, attack));
                }
            }
        }
    }

    if actor.can_move() {
        let from = MapPos::from_world_pos(actor.pos);
        let to = MapPos::from_world_pos(selected_pos);
        let obstacles = find_movement_obstacles(&objects);

        if let Some(path) = map.find_path(from, to, &obstacles) {
            if path.len() > 0 && path.len() <= actor.move_distance().into() {
                result.push(Action::move_to(path));
            }
        }
    }

    result
}
