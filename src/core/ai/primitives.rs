use std::cmp::Ordering;
use std::collections::HashMap;

use specs::prelude::*;

use super::super::actors::*;
use crate::components::*;
use crate::core::*;

pub fn find_obstacles<F>(
    positions: &ReadStorage<Position>,
    obstacles: &ReadStorage<ObstacleCmp>,
    allow: F,
)  -> ObstacleSet where F: Fn(&ObstacleCmp) -> bool {
    let mut result = HashMap::new();

    for (o, Position(p)) in (obstacles, positions).join() {
        if !allow(o) {
            result.insert(MapPos::from_world_pos(*p), Obstacle(f32::MAX));
        }
    }

    ObstacleSet(result)
}

pub fn find_movement_obstacles(
    positions: &ReadStorage<Position>,
    obstacles: &ReadStorage<ObstacleCmp>,
    team: &Team,
) -> ObstacleSet {
    find_obstacles(
        &positions,
        &obstacles,
        | ObstacleCmp { restrict_movement, .. } |
        match restrict_movement {
            Restriction::AllowAll => true,
            Restriction::AllowTeam(allowed_team) => allowed_team == team,
            _ => false,
        }
    )
}

pub fn find_enemies(
    actor: &Actor,
    entities: &Entities,
    objects: &ReadStorage<GameObjectCmp>,
) -> Vec<(Entity, Actor)> {
    let mut result: Vec<(Entity, Actor)> = Vec::new();
    let apos = MapPos::from_world_pos(actor.pos);

    for (te, GameObjectCmp(obj)) in (entities, objects).join() {
        if let GameObject::Actor(ta) = obj {
            if ta.team != actor.team {
                result.push((te, ta.clone()));
            }
        }
    }

    result.sort_by(|(_, a1), (_, a2)| {
        let d1 = apos.distance(MapPos::from_world_pos(a1.pos));
        let d2 = apos.distance(MapPos::from_world_pos(a2.pos));

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
            let (x, y) = a.pos.as_xy();

            if x.floor() == at.x().floor() && y.floor() == at.y().floor() {
                return Some((e, a.clone()));
            }
        }
    }

    None
}

fn can_attack_with(
    actor: &Actor,
    target: &Actor,
    attack: &AttackOption,
    map: &Read<Map>,
    positions: &ReadStorage<Position>,
    obstacles: &ReadStorage<ObstacleCmp>,
) -> bool {
    let from = MapPos::from_world_pos(actor.pos);
    let to = MapPos::from_world_pos(target.pos);
    let d = from.distance(to);

    if d > attack.max_distance.into() {
        return false;
    } else {
        let attackers_team = &actor.team;
        let obstacles = find_obstacles(
            &positions,
            &obstacles,
            | ObstacleCmp { restrict_melee_attack, .. } | {
                match restrict_melee_attack {
                    Restriction::AllowAll => true,
                    Restriction::AllowTeam(allowed_team) => {
                        allowed_team == attackers_team
                    }
                    _ => false,
                }
            }
        ).ignore(to);
        
        if let Some(_) = map.find_straight_path(from, to, &obstacles) {
            return true;
        }
    }

    false
}

pub fn can_attack_melee(
    actor: &Actor,
    target: &Actor,
    map: &Read<Map>,
    positions: &ReadStorage<Position>,
    obstacles: &ReadStorage<ObstacleCmp>,
) -> Option<AttackOption> {
    let attack = actor.melee_attack();
    if can_attack_with(actor, target, &attack, map, positions, obstacles) {
        Some(attack)
    } else {
        None
    }
}

pub fn can_charge(
    actor: &Actor,
    target: &Actor,
    map: &Read<Map>,
    positions: &ReadStorage<Position>,
    obstacles: &ReadStorage<ObstacleCmp>,
) -> Option<AttackOption> {
    let attack = actor.melee_attack();
    let from = MapPos::from_world_pos(actor.pos);
    let to = MapPos::from_world_pos(target.pos);
    let d = from.distance(to);
    // let reach: usize = attack.reach.into();
    let move_distance: usize = actor.move_distance().into();

    if actor.can_move() && 1 < d && d <= 1 + move_distance {
        let obstacles = find_movement_obstacles(positions, obstacles, &actor.team).ignore(to);
        if let Some(_) = map.find_straight_path(from, to, &obstacles) {
            return Some(attack);
        }
    }

    None
}

pub fn can_move_towards(
    actor: &Actor,
    target: &Actor,
    map: &Read<Map>,
    positions: &ReadStorage<Position>,
    obstacles: &ReadStorage<ObstacleCmp>,
) -> Option<Path> {
    if !actor.can_move() {
        return None;
    }

    let st = map.find_tile(actor.pos);
    let tt = map.find_tile(target.pos);

    if let (Some(source_tile), Some(target_tile)) = (st, tt) {
        find_path_next_to_tile(&source_tile, &target_tile, &actor.team, map, positions, obstacles)
    } else {
        None
    }
}

fn find_path_next_to_tile(
    source_tile: &Tile,
    target_tile: &Tile,
    team: &Team,
    map: &Read<Map>,
    positions: &ReadStorage<Position>,
    obstacles: &ReadStorage<ObstacleCmp>,
) -> Option<Path> {
    let distance = source_tile.distance(&target_tile);
    if distance <= 1.0 {
        // entity is already next to the target
        return None;
    }

    let obstacles = find_movement_obstacles(positions, obstacles, team)
        // ignore obstacles at target since we only want to move next to it
        .ignore(target_tile.to_map_pos());

    map.find_path(
        source_tile.to_map_pos(),
        target_tile.to_map_pos(),
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
