mod primitives;

use std::cmp::Ordering;

use specs::prelude::*;

use crate::components::*;
use crate::core::*;
use primitives::*;

pub fn action(e: &(Entity, Actor), w: &World) -> Action {
    zombi_action(&e, w)
}

pub fn reaction(e: &(Entity, Actor), o: Opportunity) -> Vec<Action> {
    match o {
        Opportunity::IncommingAttack(attacker, attack) => {
            e.1.defences(&attack)
                .iter()
                .map(|d| Action::defence(attacker.clone(), e.clone(), attack.clone(), d.clone()))
                .collect()
        }
    }
}

fn zombi_action((e, a): &(Entity, Actor), w: &World) -> Action {
    let (entities, game_objects, map): (Entities, ReadStorage<GameObjectCmp>, Read<Map>) =
        w.system_data();

    // let ActorCmp(actor) = actors.get(e).unwrap();
    let pos = a.pos;
    let mut enemies = find_enemies(a, &entities, &game_objects);
    enemies.sort_by(|(_, a1), (_, a2)| {
        let d1 = WorldPos::distance(&pos, &a1.pos);
        let d2 = WorldPos::distance(&pos, &a2.pos);

        if d1 <= d2 {
            Ordering::Less
        } else {
            Ordering::Greater
        }
    });

    for (te, ta) in enemies {
        let movement = can_move_towards(a, &ta, &map, &game_objects);
        if let Some((cost, tile)) = movement {
            return Action::move_to(cost, tile);
        }

        if let Some(attack) = can_attack(a, &ta) {
            return Action::attack((te, ta), attack);
        }
    }

    Action::wait(2)
}

pub fn select_action(
    _e: (Entity, Actor),
    _o: &Opportunity,
    actions: &Vec<Action>,
    _w: &World,
) -> Action {
    // TODO real implementation
    actions.first().unwrap().clone()
}


pub fn actions_at(
    (_, actor): &(Entity, Actor),
    selected_pos: WorldPos,
    world: &World,
) -> Vec<Action> {
    let (map, objects): (Read<Map>, ReadStorage<GameObjectCmp>) = world.system_data();

    let p = actor.pos;
    let mut result = vec![Action::wait(2)];

    if let Some(target_tile) = map.find_tile(selected_pos) {
        if let Some(enemy) = find_enemy_at(world, &selected_pos, &actor.team) {
            for attack in actor.attacks(&enemy.1) {
                result.push(Action::attack(enemy.clone(), attack));
            }
        }

        let obstacles = find_all_obstacles(&map, &objects);
        if let Some(path) = map.find_path(p, selected_pos, &obstacles) {
            if path.len() <= 2 {
                result.push(Action::move_to(path.len() as u8, target_tile));
            }
        }
    }

    result
}
