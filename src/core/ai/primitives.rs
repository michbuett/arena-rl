use std::collections::HashMap;
use std::cmp::Ordering;

use specs::prelude::*;

use crate::components::*;
use crate::core::*;
use super::super::actors::*;

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
    // let Position(p) = positions.get(e).unwrap();

    for (te, GameObjectCmp(obj)) in (entities, objects).join() {
        if let GameObject::Actor(ta) = obj {
            if ta.team != actor.team {
                result.push((te, ta.clone()));
            }
        }
    }

    result
}

pub fn find_enemy_at(w: &World, at: &WorldPos, team: &Team) -> Option<(Entity, Actor)> {
    let (entities, objects): (Entities, ReadStorage<GameObjectCmp>) = w.system_data();

    for (e, GameObjectCmp(o)) in (&entities, &objects).join() {
        if let GameObject::Actor(a) = o {
            let WorldPos(x, y) = a.pos;

            if x.floor() == at.0.floor() && y.floor() == at.1.floor() && &a.team != team {
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
) -> Option<(u8, Tile)> {
    if !actor.can_move() {
        return None
    }

    let st = map.find_tile(actor.pos);
    let tt = map.find_tile(target.pos);

    if let (Some(source_tile), Some(target_tile)) = (st, tt) {
        let distance = source_tile.distance(&target_tile);
        if distance <= 1.0 {
            // entity is already next to the target
            return None
        }

        let obstacles = find_all_obstacles(&map, &objects);
        let mut neighbors: Vec<Tile> = map.neighbors(target_tile, &obstacles).collect();

        neighbors.sort_by(|t1, t2| {
            let d1 = source_tile.distance(&t1);
            let d2 = source_tile.distance(&t2);
            if d1 <= d2 { Ordering::Less } else { Ordering::Greater }
        });

        for n in neighbors {
            let result = map
                .find_path(source_tile.to_world_pos(), n.to_world_pos(), &obstacles)
                .and_then(|p| p.take_step());

            if let Some((t, _)) = result {
                return Some((1, t))
            }
        }
    }

    None
}


pub fn can_attack(actor: &Actor, target: &Actor) -> Option<AttackOption> {
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
