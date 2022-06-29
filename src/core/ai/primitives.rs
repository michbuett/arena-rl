use std::cmp::Ordering;
use std::collections::HashMap;

use super::super::actors::*;
use crate::components::*;
use crate::core::*;

pub type AttackVector = Vec<(MapPos, bool, Cover, Option<ID>)>;
// pub type AttackVector = Vec<(MapPos, bool, Option<(ID, Cover)>)>;
// pub type AttackVector = Vec<(MapPos, bool, Option<(Obstacle, Option<ID>)>)>;

pub fn find_path_for(a: &Actor, to: impl Into<MapPos>, world: &CoreWorld) -> Option<Path> {
    let from: MapPos = MapPos::from_world_pos(a.pos);
    let to: MapPos = to.into();
    let obstacles = world
        .collect_obstacles()
        .drain()
        .filter_map(|(p, (oc, _))| movement_obstacle(a, &oc).map(|o| (p, o)))
        .collect::<HashMap<_, _>>();

    world.map().find_path(from, to, &ObstacleSet(obstacles))
}

pub fn find_path_towards(actor: &Actor, target: &Actor, world: &CoreWorld) -> Option<Path> {
    let obstacles = ObstacleSet(
        world
            .collect_obstacles()
            .drain()
            .filter_map(|(p, (oc, _))| movement_obstacle(actor, &oc).map(|o| (p, o)))
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
        })
        .collect::<Vec<_>>();

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

fn movement_obstacle(a: &Actor, oc: &ObstacleCmp) -> Option<Obstacle> {
    if a.is_flying() {
        return oc.movement.1
    }

    if a.is_underground() {
        return oc.movement.2
    }

    oc.movement.0
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

    let obstacles = world.collect_obstacles();
    let line_of_attack = SuperLineIter::new(from, to);
    let mut result: AttackVector = vec![];
    let mut is_advancing = true;
    let mut cover = Cover::none();

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
                if let Some(_) = movement_obstacle(attacker, &obs) {
                    // the path for advancing is blocked
                    // => no attack possible
                    return None;
                }
            } else if let Some(obs) = obs.reach.as_ref().and_then(|hitbox| {
                hitbox.obstacle_at(
                    pos.to_world_pos().as_xy(),
                    from.to_world_pos().as_xy(),
                    to.to_world_pos().as_xy(),
                )
            }) {
                match obs {
                    Obstacle::Blocker => {
                        // cannot reach target; obstacle blocks the way completely
                        // => no attack possible
                        return None;
                    }

                    Obstacle::Impediment(..) => {
                        result.push((pos, is_target, cover.clone(), *id));

                        if from.distance(pos) == 1 && !is_target {
                            // the first obstacle in the line of attack which is directly
                            // next to the attacker should not act as an obstacle for the
                            // attacker (imagine e.g. some crates used for cover)
                            // => ignore this case
                        } else {
                            // consider the current obstacle as cover for the next obstacle in the path
                            cover = cover.add_obstacle(obs, pos, *id);
                        }
                    }
                }

            }
        } else {
            // no obstacle (or anything to attack) here
            result.push((pos, is_target, cover.clone(), None))
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
        let hinderance = obstacles
            .get(&tile_pos)
            .and_then(|(obs, _)| movement_obstacle(moving_actor, obs));

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
