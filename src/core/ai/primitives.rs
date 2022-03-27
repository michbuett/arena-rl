use std::cmp::Ordering;
use std::collections::HashMap;

use super::super::actors::*;
use crate::components::*;
use crate::core::*;

pub type AttackVector = Vec<(MapPos, bool, Option<(Obstacle, Option<ID>)>)>;

pub fn find_path_for(a: &Actor, to: impl Into<MapPos>, world: &CoreWorld) -> Option<Path> {
    let team = &a.team;
    let from: MapPos = MapPos::from_world_pos(a.pos);
    let to: MapPos = to.into();
    let obstacles = world
        .collect_obstacles()
        .drain()
        .filter_map(|(p, (oc, _))| to_obstancle(team, &oc.restrict_movement).map(|o| (p, o)))
        .collect::<HashMap<_, _>>();

    world.map().find_path(from, to, &ObstacleSet(obstacles))
}

pub fn find_path_towards(actor: &Actor, target: &Actor, world: &CoreWorld) -> Option<Path> {
    let team = &actor.team;
    let obstacles = ObstacleSet(
        world
            .collect_obstacles()
            .drain()
            .filter_map(|(p, (oc, _))| to_obstancle(team, &oc.restrict_movement).map(|o| (p, o)))
            .collect::<HashMap<_, _>>(),
    )
    // We can ignore the target for pathfinding because we are looking for the place
    // right before the target
    .ignore(target.pos.into());

    if let Some(mut p) = world
        .map()
        .find_path(actor.pos.into(), target.pos.into(), &obstacles)
    {
        if p.len() >= 2 {
            p.pop();
            return Some(p);
        }
    }
    None
}

pub fn find_enemies(actor: &Actor, world: &CoreWorld) -> Vec<Actor> {
    let team = &actor.team;
    let apos = MapPos::from_world_pos(actor.pos);
    let mut enemies = world
        .collect_obstacles()
        .drain()
        .filter_map(|(_, (_, id))| {
            if let Some(a) = id.and_then(|id| world.get_actor(id)) {
                if &a.team != team {
                    return Some(a.clone());
                }
            }
            None
        }).collect::<Vec<_>>();

    enemies.sort_by(|a1, a2| {
        let d1 = apos.distance(MapPos::from_world_pos(a1.pos));
        let d2 = apos.distance(MapPos::from_world_pos(a2.pos));

        if d1 <= d2 {
            Ordering::Less
        } else {
            Ordering::Greater
        }
    });

    enemies
}

pub fn find_actor_at<I: Into<MapPos>>(w: &CoreWorld, at: I) -> Option<Actor> {
    let at: MapPos = at.into();
    w.find_actor(|a| at == a.pos.into())
}

pub fn possible_attacks(
    actor: &Actor,
    target: &Actor,
    world: &CoreWorld,
) -> Vec<(AttackOption, AttackVector)> {
    actor
        .attacks()
        .drain(..)
        .filter_map(|attack| attack_vector(actor, target, &attack, world).map(|av| (attack, av)))
        .collect()
}

pub fn can_attack_with(
    attacker: &Actor,
    target: &Actor,
    attack: &AttackOption,
    world: &CoreWorld,
) -> bool {
    attack_vector(attacker, target, attack, world).is_some()
}

fn to_obstancle(t: &Team, r: &Restriction) -> Option<Obstacle> {
    let c = match r {
        Restriction::ForAll(c) => c,
        Restriction::ForTeam(team, c1, c2) => {
            if team == t {
                c1
            } else {
                c2
            }
        }
    };

    if *c > 0 {
        Some(Obstacle(*c))
    } else {
        None
    }
}

pub fn attack_vector(
    attacker: &Actor,
    target: &Actor,
    attack: &AttackOption,
    world: &CoreWorld,
) -> Option<AttackVector> {
    let from = MapPos::from_world_pos(attacker.pos);
    let to = MapPos::from_world_pos(target.pos);
    let max_distance = attack.advance + attack.max_distance;
    let d = from.distance(to);

    if d > max_distance.into() {
        // target is out of reach => no need to check for obstacles
        return None;
    }

    if d < attack.min_distance.into() {
        // target too close, since there is no auto-fall-back we can stop here
        return None;
    }

    let attackers_team = &attacker.team;
    let obstacles = world.collect_obstacles();
    let line_of_attack = SuperLineIter::new(from, to);
    let mut result: AttackVector = vec![];
    let mut is_advancing = true;

    for pos in line_of_attack {
        if pos == from {
            // ignore to first pos (that's where the attacker is)
            continue;
        }

        if world.map().get_tile(pos).is_none() {
            // the world has ended
            // => return what we've got so far
            return Some(result);
        }

        let is_target = pos == to;

        is_advancing = is_advancing && !is_target && from.distance(pos) <= attack.advance.into();

        if let Some((obs, id)) = obstacles.get(&pos) {
            if is_advancing {
                // we are still advancing
                // => check for obstacles which hinder movement
                if let Some(_) = to_obstancle(attackers_team, &obs.restrict_movement) {
                    // the path for advancing is blocked
                    // => no attack possible
                    return None;
                }
            } else if attack.attack_type.is_melee() {
                if let Some(obs) = to_obstancle(attackers_team, &obs.restrict_melee_attack) {
                    result.push((pos, is_target, Some((obs, *id))));
                }
            } else if attack.attack_type.is_ranged() {
                if let Some(obs) = to_obstancle(attackers_team, &obs.restrict_ranged_attack) {
                    result.push((pos, is_target, Some((obs, *id))));
                }
            }
        } else {
            result.push((pos, is_target, None))
        }

        if from.distance(pos) >= max_distance.into() {
            return Some(result);
        }
    }

    None
}

pub fn find_charge_path(
    moving_actor: &Actor,
    target_actor: &Actor,
    world: &CoreWorld,
) -> Option<Path> {
    let from_pos = MapPos::from_world_pos(moving_actor.pos);
    let target_pos = MapPos::from_world_pos(target_actor.pos);
    if from_pos == target_pos {
        return None;
    }

    let mut result = vec![];
    let mut obstacles = world.collect_obstacles();
    let super_line_iter =
        SuperLineIter::new(from_pos, target_pos).map_while(|p| world.map().get_tile(p));

    // it is clear that the attacker pos and target pos will be occupied with by the attacker/target
    // => ignore them
    obstacles.remove(&from_pos);
    obstacles.remove(&target_pos);

    for t in super_line_iter {
        let tile_pos = t.to_map_pos();
        let team = &moving_actor.team;
        let hinderance = obstacles
            .get(&tile_pos)
            .and_then(|(obs, _)| to_obstancle(team, &obs.restrict_movement));

        if hinderance.is_some() {
            return None;
        }

        result.push(t);

        if tile_pos == target_pos {
            return Some(result);
        }
    }

    None
}
