mod primitives;

use std::cmp::max;

use crate::core::*;
use primitives::*;

pub use primitives::{attack_vector, can_attack_with, find_charge_path, AttackVector};

pub fn action(a: &Actor, w: CoreWorld) -> Act {
    zombi_action(a, w)
}

fn zombi_action(a: &Actor, cw: CoreWorld) -> Act {
    for ta in find_enemies(&a, &cw) {
        let attacks = possible_attacks(a, &ta, &cw)
            .drain(..)
            // exclude attacks where the required efford exeeds the avaliable efford (this could lead to a self-KO)
            .filter(|(attack, _)| attack.required_effort <= a.available_effort())
            .collect::<Vec<_>>();

        if let Some((attack, attack_vector)) = pick_one(attacks) {
            return Act::attack(ta.id, attack, attack_vector, ta.name);
        }

        if let Some(path) = find_path_towards(a, &ta, &cw) {
            let p = path.iter()
                    .take(a.move_distance().into())
                    .cloned()
                    .collect();

            return Act::move_to(move_effort(a, &p), p)
        }
    }

    Act::pass()
}

pub fn actions_at(actor: &Actor, selected_pos: WorldPos, cw: CoreWorld) -> Vec<Act> {
    let mut result = vec![];

    if let Some(other_actor) = find_actor_at(&cw, &selected_pos) {
        if actor.id == other_actor.id {
            // selected position contains the acting character itself
            // for (k, t, d) in actor.ability_self() {
            //     result.push(Act::use_ability(actor.id, k, t, d));
            // }

            for attack in actor.attacks() {
                result.push(Act::ambush(attack));
            }

            result.push(Act::rest());
        } else {
            if actor.team == other_actor.team {
                if other_actor.can_activate() {
                    result.push(Act::activate(other_actor.id));
                }
            } else {
                for (attack, attack_vector) in possible_attacks(actor, &other_actor, &cw) {
                    result.push(Act::attack(
                        other_actor.id,
                        attack,
                        attack_vector,
                        other_actor.name.clone(),
                    ));
                }
            }
        }
    }

    if actor.can_move() {
        if let Some(path) = find_path_for(actor, selected_pos, &cw) {
            if path.len() > 0 && path.len() <= actor.move_distance().into() {
                result.push(Act::move_to(move_effort(actor, &path), path));
            }
        }
    }

    // TODO create concept for leaving combat (existing Disengage mechanic does not feel good)
    // if actor.engaged_in_combat {
    //     let from = MapPos::from_world_pos(actor.pos);
    //     let to = MapPos::from_world_pos(selected_pos);
    //     let obstacles = find_movement_obstacles(&positions, &obstacle_cmp, &actor.team);

    //     if from.distance(to) == 1 && !obstacles.0.contains_key(&to) {
    //         if let Some(tile) = map.get_tile(to) {
    //             result.push(Act::disengage(tile));
    //         }
    //     }
    // }

    result
}

fn pick_one<T>(mut list: Vec<T>) -> Option<T> {
    extern crate rand;
    use rand::prelude::*;

    if list.is_empty() {
        return None;
    }

    let range = rand::distributions::Uniform::from(0..list.len());
    let mut rng = rand::thread_rng();
    let idx = rng.sample(range);

    Some(list.remove(idx))
}

fn move_effort(a: &Actor, p: &Path) -> u8 {
    let move_mod = a.attr(Attr::Movement).val() as i32;
    let path_len = p.len() as i32;

    max(1, path_len - move_mod) as u8
}
