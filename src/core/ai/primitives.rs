// use std::iter::Sum;
use std::cmp::Ordering;
use std::collections::HashMap;

use specs::prelude::*;

use super::super::actors::*;
use crate::components::*;
use crate::core::*;

pub type AttackVector = Vec<(MapPos, bool, Option<(Entity, Obstacle)>)>;

pub fn find_obstacles<F>(
    positions: &ReadStorage<Position>,
    obstacles: &ReadStorage<ObstacleCmp>,
    costs: F,
) -> ObstacleSet
where
    F: Fn(&ObstacleCmp) -> Option<i8>,
{
    let mut result = HashMap::new();

    for (o, Position(p)) in (obstacles, positions).join() {
        if let Some(c) = costs(o) {
            result.insert(MapPos::from_world_pos(*p), Obstacle(c));
        }
    }

    ObstacleSet(result)
}

pub fn find_movement_obstacles(
    positions: &ReadStorage<Position>,
    obstacles: &ReadStorage<ObstacleCmp>,
    team: &Team,
) -> ObstacleSet {
    find_obstacles(&positions, &obstacles, |obs| {
        restriction_costs(&obs.restrict_movement, team)
    })
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
    let at = MapPos::from_world_pos(*at);

    for (e, GameObjectCmp(o)) in (&entities, &objects).join() {
        if let GameObject::Actor(a) = o {
            let apos = MapPos::from_world_pos(a.pos);

            if apos == at {
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
        let obstacles = find_obstacles(&positions, &obstacles, |obs| {
            restriction_costs(&obs.restrict_melee_attack, attackers_team)
        })
        .ignore(to);

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
    // let move_distance: usize = max_move_distance(actor).into();

    if d <= 1 {
        // you cannot charge what is right next to you
        return None;
    }
    
    if !actor.can_move() {
        // actor may be engaged in combat or something like this
        return None;
    }

    let obstacles = find_movement_obstacles(positions, obstacles, &actor.team).ignore(to);
    if let Some(p) = map.find_straight_path(from, to, &obstacles) {
        if p.len() == 2 {
            return Some(attack);
        }
    }

    None
}

pub fn can_attack_at_range(
    actor: &Actor,
    target_pos: &WorldPos,
) -> Option<AttackOption> {
    let from = MapPos::from_world_pos(actor.pos);
    let to = MapPos::from_world_pos(*target_pos);
    let d = from.distance(to);

    actor.range_attack(d as u8)
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
        find_path_next_to_tile(
            &source_tile,
            &target_tile,
            &actor.team,
            map,
            positions,
            obstacles,
        )
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

fn restriction_costs(restriction: &Restriction, actual_team: &Team) -> Option<i8> {
    match restriction {
        Restriction::ForAll(costs) => *costs,
        Restriction::ForTeam(allowed_team, costs_for_members, costs_for_others) => {
            if allowed_team == actual_team {
                *costs_for_members
            } else {
                *costs_for_others
            }
        }
    }
}

pub fn attack_vector(
    attacker: &Actor,
    target: &Actor,
    attack: &AttackOption,
    data: (
        Entities,
        Read<Map>,
        ReadStorage<Position>,
        ReadStorage<ObstacleCmp>,
    ),
) -> Vec<(MapPos, bool, Option<(Entity, Obstacle)>)> {
    match attack.attack_type {
        AttackType::Ranged(..) => ranged_attack_vector(attacker, target, data),

        AttackType::Melee(..) => melee_attack_vector(
            attacker,
            MapPos::from_world_pos(target.pos),
            attack.max_distance,
            data,
        ),
    }
}

fn melee_attack_vector(
    attacker: &Actor,
    target_pos: MapPos,
    max_distance: u8,
    (entities, map, positions, obstacles): (
        Entities,
        Read<Map>,
        ReadStorage<Position>,
        ReadStorage<ObstacleCmp>,
    ),
) -> Vec<(MapPos, bool, Option<(Entity, Obstacle)>)> {
    let from = MapPos::from_world_pos(attacker.pos);
    let attacker_team = &attacker.team;
    let mut result = vec![];
    let obstacles = collect_relevant_obstacles(&entities, &positions, &obstacles, |_, obs| {
        restriction_costs(&obs.restrict_melee_attack, &attacker_team)
    });

    for (d, t) in map.tiles_along_line(from, target_pos).enumerate() {
        let mp = t.to_map_pos();
        let target = obstacles.get(&mp);

        if target.is_some() {
            result.push((mp, true, target.cloned()));
        }

        if d == max_distance as usize {
            break;
        }
    }

    // println!("melee_attack_vector {:?}", result);
    result
}

fn ranged_attack_vector(
    attacker: &Actor,
    target: &Actor,
    (entities, map, positions, obstacles): (
        Entities,
        Read<Map>,
        ReadStorage<Position>,
        ReadStorage<ObstacleCmp>,
    ),
) -> Vec<(MapPos, bool, Option<(Entity, Obstacle)>)> {
    let from = MapPos::from_world_pos(attacker.pos);
    let to = MapPos::from_world_pos(target.pos);
    let attacker_team = &attacker.team;
    let mut result = vec![];
    let obstacles = collect_relevant_obstacles(&entities, &positions, &obstacles, |pos, obs| {
        if from.distance(MapPos::from_world_pos(*pos)) <= 1 {
            None
        } else {
            restriction_costs(&obs.restrict_ranged_attack, &attacker_team)
        }
    });

    for t in map.tiles_along_line(from, to) {
        let mp = t.to_map_pos();
        let is_target_pos = mp == to;

        result.push((mp, is_target_pos, obstacles.get(&mp).cloned()));
    }

    result
}

fn collect_relevant_obstacles<F>(
    entities: &Entities,
    positions: &ReadStorage<Position>,
    obstacles: &ReadStorage<ObstacleCmp>,
    costs: F,
) -> HashMap<MapPos, (Entity, Obstacle)>
where
    F: Fn(&WorldPos, &ObstacleCmp) -> Option<i8>,
{
    let mut result = HashMap::new();

    for (e, Position(p), obs) in (entities, positions, obstacles).join() {
        if let Some(c) = costs(&p, obs) {
            result.insert(MapPos::from_world_pos(*p), (e, Obstacle(c)));
        }
    }

    result
}

pub fn max_move_distance(_: &Actor) -> u8 {
    // TODO implement logic
    // 1 - walk (reserves effort but slow)
    // 2 - run (fast but no effort left afterwards)
    // 3 - sprint (very fast but may causes stress/pain)
    2
}
