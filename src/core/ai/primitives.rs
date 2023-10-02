use std::cmp::{max, Ordering};
use std::collections::HashMap;
use std::num::NonZeroU8;

use super::super::actors::*;
use crate::components::*;
use crate::core::*;

pub type AttackVector = Vec<(MapPos, bool, Cover, Option<ID>)>;
pub type PlayerActionOptions = HashMap<MapPos, Vec<PlayerAction>>;

pub fn find_path_towards(actor: &Actor, target: &Actor, world: &CoreWorld) -> Option<Path> {
    let obstacles = movment_obstacles(actor, world)
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

fn movment_obstacles(actor: &Actor, world: &CoreWorld) -> ObstacleSet {
    ObstacleSet(
        world
            .collect_obstacles()
            .drain()
            .filter_map(|(p, (oc, _))| movement_obstacle(actor, &oc).map(|o| (p, o)))
            .collect::<HashMap<_, _>>(),
    )
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

fn movement_obstacle(a: &Actor, oc: &ObstacleCmp) -> Option<Obstacle> {
    if a.is_flying() {
        return oc.movement.1;
    }

    if a.is_underground() {
        return oc.movement.2;
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

pub fn move_effort(a: &Actor, p: &Path) -> u8 {
    let move_mod = a.attr(Attr::Movement).val() as i32;
    let path_len = p.len() as i32;

    max(1, path_len - move_mod) as u8
}

pub fn add_option<P: Into<MapPos>>(p: P, a: PlayerAction, result: &mut PlayerActionOptions) {
    let p = p.into();
    if result.contains_key(&p) {
        result.get_mut(&p).unwrap().push(a);
    } else {
        result.insert(p, vec![a]);
    }
}

pub fn add_move_to_options(active_actor: &Actor, w: &CoreWorld, result: &mut PlayerActionOptions) {
    let p0 = MapPos::from_world_pos(active_actor.pos);
    let t0 = w.map().get_tile(p0).unwrap();
    let d = NonZeroU8::new(active_actor.move_distance()).unwrap();
    let obstacles = movment_obstacles(active_actor, w);

    for t in w.map().neighbors(t0, d, &obstacles) {
        if let Some(path) = w.map().find_path(p0, t.to_map_pos(), &obstacles) {
            if path.len() <= d.get() as usize {
                let effort = move_effort(active_actor, &path);
                let action = ActorAction::MoveTo { path, effort };
                add_option(
                    t,
                    PlayerAction::TriggerAction(active_actor.id, action),
                    result,
                );
            }
        }
    }
}

pub fn add_combat_options(active_actor: &Actor, w: &CoreWorld, result: &mut PlayerActionOptions) {
    let attacks = active_actor.attacks();

    for go in w.game_objects() {
        if let GameObject::Actor(other) = go {
            if other.id != active_actor.id && other.team != active_actor.team {
                for a in attacks.iter() {
                    if let Some(attack_vector) = attack_vector(active_actor, other, a, w) {
                        let msg = format!("{} at {}", a.name, other.name);
                        let action = ActorAction::Attack {
                            target: other.id,
                            attack: a.clone(),
                            attack_vector,
                            msg,
                        };

                        add_option(
                            other.pos,
                            PlayerAction::TriggerAction(active_actor.id, action),
                            result,
                        );
                    }
                }
            }
        }
    }

    add_option(
        active_actor.pos,
        PlayerAction::TriggerAction(
            active_actor.id,
            ActorAction::AddTrait {
                targets: vec![active_actor.id],
                trait_ref: "temp#Trait_Block".to_string(),
                msg: "Block".to_string(),
            },
        ),
        result,
    );
}

// pub fn add_noop_option(active_actor: &Actor, result: &mut PlayerActionOptions) {
//     add_option(
//         active_actor.pos,
//         PlayerAction::SaveEffort(active_actor.id),
//         result,
//     );
// }
