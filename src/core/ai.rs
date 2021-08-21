mod primitives;

use specs::prelude::*;

use crate::components::*;
use crate::core::*;
use primitives::*;

pub use primitives::{
    attack_vector, can_attack_melee, can_charge, can_move_towards, find_movement_obstacles,
    AttackVector,
};

pub fn action(e: &(Entity, Actor), w: &World) -> (Action, u8) {
    zombi_action(&e, w)
}

fn zombi_action((_, a): &(Entity, Actor), w: &World) -> (Action, u8) {
    let (entities, game_objects, positions, obstacle_cmp, map): (
        Entities,
        ReadStorage<GameObjectCmp>,
        ReadStorage<Position>,
        ReadStorage<ObstacleCmp>,
        Read<Map>,
    ) = w.system_data();

    for (te, ta) in find_enemies(a, &entities, &game_objects) {
        if let Some(attack) = can_attack_melee(a, &ta, &map, &positions, &obstacle_cmp) {
            return Action::melee_attack(te, attack, ta.name);
        }
        if let Some(attack) = can_charge(a, &ta, &map, &positions, &obstacle_cmp) {
            return Action::charge(te, attack, ta.name);
        }

        if a.can_move() {
            if let Some(path) = can_move_towards(a, &ta, &map, &positions, &obstacle_cmp) {
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
    let (map, positions, obstacle_cmp): (
        Read<Map>,
        ReadStorage<Position>,
        ReadStorage<ObstacleCmp>,
    ) = world.system_data();
    let mut result = vec![];

    if let Some((other_entity, other_actor)) = find_actor_at(world, &selected_pos) {
        if entity.id() == other_entity.id() {
            for (k, t, d) in actor.ability_self() {
                result.push(Action::use_ability(*entity, k, t, d));
            }
        } else {
            if actor.team == other_actor.team {
                if other_actor.can_activate() {
                    result.push(Action::activate(other_entity));
                }
            } else {
                if let Some(attack_option) = can_attack_at_range(actor, &selected_pos) {
                    let attack_vector =
                        attack_vector(actor, &other_actor, &attack_option, world.system_data());

                    result.push(Action::ranged_attack(
                        other_entity,
                        attack_option,
                        attack_vector,
                        other_actor.name.clone(),
                    ));
                }

                if let Some(attack) =
                    can_attack_melee(actor, &other_actor, &map, &positions, &obstacle_cmp)
                {
                    result.push(Action::melee_attack(other_entity, attack, other_actor.name.clone()));
                }

                if let Some(attack) =
                    can_charge(actor, &other_actor, &map, &positions, &obstacle_cmp)
                {
                    result.push(Action::charge(other_entity, attack, other_actor.name.clone()));
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
                result.push(Action::move_to(path));
            }
        }
    }

    if actor.engaged_in_combat {
        let from = MapPos::from_world_pos(actor.pos);
        let to = MapPos::from_world_pos(selected_pos);
        let obstacles = find_movement_obstacles(&positions, &obstacle_cmp, &actor.team);

        if from.distance(to) == 1 && !obstacles.0.contains_key(&to) {
            if let Some(tile) = map.get_tile(to) {
                result.push(Action::dodge(tile));
            }
        }
    }

    result
}
