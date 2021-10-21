mod primitives;

use specs::prelude::*;

use crate::components::*;
use crate::core::*;
use primitives::*;

pub use primitives::{
    attack_vector, can_attack_melee, can_charge, can_move_towards, find_movement_obstacles,
    AttackVector, max_move_distance
};

pub fn action(e: &(Entity, Actor), w: &World) -> Act {
    zombi_action(&e, w)
}

fn zombi_action((_, a): &(Entity, Actor), w: &World) -> Act {
    let (entities, game_objects, positions, obstacle_cmp, map): (
        Entities,
        ReadStorage<GameObjectCmp>,
        ReadStorage<Position>,
        ReadStorage<ObstacleCmp>,
        Read<Map>,
    ) = w.system_data();

    for (te, ta) in find_enemies(a, &entities, &game_objects) {
        if let Some(attack) = can_attack_melee(a, &ta, &map, &positions, &obstacle_cmp) {
            return Act::melee_attack(te, attack, ta.name, a.available_effort());
        }

        if let Some(attack) = can_charge(a, &ta, &map, &positions, &obstacle_cmp) {
            return Act::charge(te, attack, ta.name);
        }

        
        if let Some(path) = can_move_towards(a, &ta, &map, &positions, &obstacle_cmp) {
            return Act::move_to(
                path.iter()
                    .take(a.move_distance().into())
                    .cloned()
                    .collect(),
            );
        }
    }

    Act::pass()
}

pub fn actions_at(
    (entity, actor): &(Entity, Actor),
    selected_pos: WorldPos,
    world: &World,
) -> Vec<Act> {
    let mut result = vec![];
    let (map, positions, obstacle_cmp): (
        Read<Map>,
        ReadStorage<Position>,
        ReadStorage<ObstacleCmp>,
    ) = world.system_data();

    if let Some((other_entity, other_actor)) = find_actor_at(world, &selected_pos) {
        if entity.id() == other_entity.id() {
            // selected position contains the acting character itself
            for (k, t, d) in actor.ability_self() {
                result.push(Act::use_ability(*entity, k, t, d));
            }

            result.push(Act::ambush(actor.melee_attack()));
        } else {
            if actor.team == other_actor.team {
                if other_actor.can_activate() {
                    result.push(Act::activate(other_entity));
                }
            } else {
                if let Some(attack_option) = can_attack_at_range(actor, &selected_pos) {
                    let attack_vector =
                        attack_vector(actor, &other_actor, &attack_option, world.system_data());

                    result.push(Act::ranged_attack(
                        other_entity,
                        attack_option,
                        attack_vector,
                        other_actor.name.clone(),
                        actor.available_effort(),
                    ));
                }

                if let Some(attack) =
                    can_attack_melee(actor, &other_actor, &map, &positions, &obstacle_cmp)
                {
                    result.push(Act::melee_attack(
                        other_entity,
                        attack,
                        other_actor.name.clone(),
                        actor.available_effort(),
                    ));
                }

                if let Some(attack) =
                    can_charge(actor, &other_actor, &map, &positions, &obstacle_cmp)
                {
                    result.push(Act::charge(other_entity, attack, other_actor.name.clone()));
                }
            }
        }
    }

    if actor.can_move() {
        let from = MapPos::from_world_pos(actor.pos);
        let to = MapPos::from_world_pos(selected_pos);
        let obstacles = find_movement_obstacles(&positions, &obstacle_cmp, &actor.team);

        if let Some(path) = map.find_path(from, to, &obstacles) {
            if path.len() > 0 && path.len() <= actor.move_distance().into() {
                result.push(Act::move_to(path));
            }
        }
    }

    if actor.engaged_in_combat {
        let from = MapPos::from_world_pos(actor.pos);
        let to = MapPos::from_world_pos(selected_pos);
        let obstacles = find_movement_obstacles(&positions, &obstacle_cmp, &actor.team);

        if from.distance(to) == 1 && !obstacles.0.contains_key(&to) {
            if let Some(tile) = map.get_tile(to) {
                result.push(Act::disengage(tile));
            }
        }
    }

    result
}
