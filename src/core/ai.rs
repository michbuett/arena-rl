mod primitives;

use specs::prelude::*;

use crate::components::*;
use crate::core::*;
use primitives::*;

pub fn action(e: &(Entity, Actor), w: &World) -> (Action, u8) {
    zombi_action(&e, w)
}

fn zombi_action((_, a): &(Entity, Actor), w: &World) -> (Action, u8) {
    let (entities, game_objects, map): (Entities, ReadStorage<GameObjectCmp>, Read<Map>) =
        w.system_data();

    for (te, ta) in find_enemies(a, &entities, &game_objects) {
        if let Some(attack) = can_attack_melee(a, &ta) {
            return Action::melee_attack(te, attack);
        }

        if a.can_move() {
            if let Some(path) = can_move_towards(a, &ta, &map, &game_objects) {
                let tile = path.iter().take(a.move_distance().into()).last();
                if let Some(tile) = tile {
                    return Action::move_to(tile.clone());
                }
            }
        }
    }

    Action::wait(2)
}

pub fn actions_at(
    (entity, actor): &(Entity, Actor),
    selected_pos: WorldPos,
    world: &World,
) -> Vec<(Action, u8)> {
    let (map, objects): (Read<Map>, ReadStorage<GameObjectCmp>) = world.system_data();

    let p = actor.pos;
    let mut result = vec![];

    if let Some((other_entity, other_actor)) = find_actor_at(world, &selected_pos) {
        if entity.id() == other_entity.id() {
            result.push(Action::recover());
        } else {
            if actor.team == other_actor.team {
                result.push(Action::activate(other_entity));
            } else {
                for attack in actor.attacks(&other_actor) {
                    result.push(Action::melee_attack(other_entity.clone(), attack));
                }
            }
        }
    }

    if let Some(target_tile) = map.find_tile(selected_pos) {
        if actor.can_move() {
            let obstacles = find_all_obstacles(&map, &objects);
            if let Some(path) = map.find_path(p, selected_pos, &obstacles) {
                if path.len() <= actor.move_distance().into() {
                    result.push(Action::move_to(target_tile));
                }
            }
        }
    }

    result
}
