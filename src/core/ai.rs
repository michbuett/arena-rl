mod primitives;

// use std::cmp::max;
use std::collections::HashMap;

use crate::core::*;
use primitives::*;

pub use primitives::{attack_vector, find_charge_path, AttackVector, PlayerActionOptions};

pub fn determine_actor_action(actor: &Actor, cw: CoreWorld) -> Action {
    zombi_action(actor, cw)
}

fn zombi_action(actor: &Actor, cw: CoreWorld) -> Action {
    for ta in find_enemies(&actor, &cw) {
        let attacks = possible_attacks(actor, &ta, &cw)
            .drain(..)
            .collect::<Vec<_>>();

        if let Some((attack, attack_vector)) = pick_one(attacks) {
            return Action::Attack {
                attacker: actor.id,
                target: ta.id,
                attack,
                attack_vector,
                msg: ta.name,
            };
        }

        if let Some(path) = find_path_towards(actor, &ta, &cw) {
            let p = path
                .iter()
                .take(actor.move_distance().into())
                .cloned()
                .collect();

            return Action::MoveTo {
                actor: actor.id,
                effort: move_effort(actor, &p),
                path: p,
            };
        }
    }

    Action::DoNothing(actor.id)
}

pub fn possible_player_actions(actor: &Actor, cw: &CoreWorld) -> PlayerActionOptions {
    let mut result = HashMap::new();

    add_move_to_options(actor, cw, &mut result);
    add_combat_options(actor, cw, &mut result);

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
