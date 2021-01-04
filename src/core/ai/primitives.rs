use std::cmp::Ordering;
use std::collections::HashMap;

use specs::prelude::*;

use super::super::actors::*;
use crate::components::*;
use crate::core::*;

pub fn find_all_obstacles(
    map: &Read<Map>,
    objects: &ReadStorage<GameObjectCmp>,
) -> HashMap<Tile, Obstacle> {
    let mut obstacles = HashMap::new();

    for GameObjectCmp(obj) in objects.join() {
        match obj {
            GameObject::Actor(a) => {
                if let Some(t) = map.find_tile(a.pos) {
                    obstacles.insert(t, Obstacle::Inaccessible());
                }
            }

            GameObject::Item(pos, _item) => {
                if let Some(t) = map.find_tile(*pos) {
                    obstacles.insert(t, Obstacle::Impediment(1.5));
                }
            }
        }
    }

    obstacles
}

pub fn find_enemies(
    actor: &Actor,
    entities: &Entities,
    objects: &ReadStorage<GameObjectCmp>,
) -> Vec<(Entity, Actor)> {
    let mut result: Vec<(Entity, Actor)> = Vec::new();

    for (te, GameObjectCmp(obj)) in (entities, objects).join() {
        if let GameObject::Actor(ta) = obj {
            if ta.team != actor.team {
                result.push((te, ta.clone()));
            }
        }
    }

    result.sort_by(|(_, a1), (_, a2)| {
        let d1 = WorldPos::distance(&actor.pos, &a1.pos);
        let d2 = WorldPos::distance(&actor.pos, &a2.pos);

        if d1 <= d2 {
            Ordering::Less
        } else {
            Ordering::Greater
        }
    });

    result
}

pub fn find_actor_at(w: &World, at: &WorldPos) -> Option<(Entity, Actor)> {
    let (entities, objects): (Entities, ReadStorage<GameObjectCmp>) = w.system_data();

    for (e, GameObjectCmp(o)) in (&entities, &objects).join() {
        if let GameObject::Actor(a) = o {
            let WorldPos(x, y) = a.pos;

            if x.floor() == at.0.floor() && y.floor() == at.1.floor() {
                return Some((e, a.clone()));
            }
        }
    }
    None
}

pub fn can_move_towards(
    actor: &Actor,
    target: &Actor,
    map: &Read<Map>,
    objects: &ReadStorage<GameObjectCmp>,
) -> Option<Path> {
    if !actor.can_move() {
        return None;
    }

    let st = map.find_tile(actor.pos);
    let tt = map.find_tile(target.pos);

    if let (Some(source_tile), Some(target_tile)) = (st, tt) {
        find_path_next_to_tile(&source_tile, &target_tile, map, objects)
    } else {
        None
    }
}

pub fn find_path_next_to_tile(
    source_tile: &Tile,
    target_tile: &Tile,
    map: &Read<Map>,
    objects: &ReadStorage<GameObjectCmp>,
) -> Option<Path> {
    let distance = source_tile.distance(&target_tile);
    if distance <= 1.0 {
        // entity is already next to the target
        return None;
    }

    let mut obstacles = find_all_obstacles(&map, &objects);
    if obstacles.contains_key(&target_tile) {
        // ignore obstacles at target since we only want to move next to it
        obstacles.remove(&target_tile);
    }

    map.find_path(
        source_tile.to_world_pos(),
        target_tile.to_world_pos(),
        &obstacles,
    )
    .map(|p| p.iter().take(p.len() - 1).cloned().collect())
}

// pub fn find_attack_options(
//     attacker: &Actor,
//     target: &Actor,
//     map: &Read<Map>,
//     objects: &ReadStorage<GameObjectCmp>,
// ) -> Vec<AttackOption> {
//     let distance = WorldPos::distance(&attacker.pos, &target.pos);
//     let source_tile = map.find_tile(attacker.pos).unwrap();
//     let _target_tile = map.find_tile(target.pos).unwrap();

//     for a in attacker.attacks.iter() {}

//     attacker
//         .attacks
//         .iter()
//         .filter(|o| o.distance.0 <= distance && distance <= o.distance.1)
//         .cloned()
//         .collect()

//     // result
// }

pub fn can_attack_melee(actor: &Actor, target: &Actor) -> Option<AttackOption> {
    // find_attack_options(actor, target).iter().cloned().next()
    actor.attacks(target).iter().cloned().next()
}
// fn flatten<T>(oot: Option<Option<T>>) -> Option<T> {
//     oot.and_then(std::convert::identity)
// }
//
// fn flat_map<T, U, F>(ot: Option<T>, f: F) -> Option<U>
//     where F: FnOnce(T) -> Option<U>
// {
//     match ot {
//         Some(t) => f(t),
//         None => None,
//     }
// }
