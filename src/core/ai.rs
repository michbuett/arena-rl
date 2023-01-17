mod primitives;

// use std::cmp::max;
use std::collections::HashMap;

use crate::core::*;
use primitives::*;

pub use primitives::{
    attack_vector, find_charge_path, AttackVector, PlayerActionOptions,
    // attack_vector, can_attack_with, find_charge_path, AttackVector, PlayerActionOptions,
};

// pub struct OptionToAllocateEffort {
//     txt: DisplayStr,
//     actor_id: ID,
//     act: Act,
//     threshold: u8,
//     pos: MapPos,
// }

// pub fn action(a: &Actor, w: CoreWorld) -> Act {
//     zombi_action(a, w)
// }

pub fn determine_actor_action(actor: &Actor, cw: CoreWorld) -> Option<PlayerAction> {
    zombi_action(actor, cw)
}

fn zombi_action(actor: &Actor, cw: CoreWorld) -> Option<PlayerAction> {
    match &actor.state {
        ReadyState::ExecutePreparedAction => actor
            .prepared_action
            .as_ref()
            .map(|action| PlayerAction::TriggerAction(actor.id, action.clone())),

        ReadyState::SelectAction => {
            for ta in find_enemies(&actor, &cw) {
                let attacks = possible_attacks(actor, &ta, &cw)
                    .drain(..)
                    .collect::<Vec<_>>();

                if let Some((attack, attack_vector)) = pick_one(attacks) {
                    let attack_action = ActorAction::Attack {
                        target: ta.id, attack, attack_vector,
                        msg: ta.name
                    };
                    return Some(PlayerAction::PrepareAction(actor.id, attack_action))
                }

                if let Some(path) = find_path_towards(actor, &ta, &cw) {
                    let p = path
                        .iter()
                        .take(actor.move_distance().into())
                        .cloned()
                        .collect();

                    let move_action = ActorAction::MoveTo {
                        effort: move_effort(actor, &p),
                        path: p,
                    };
                    
                    return Some(PlayerAction::TriggerAction(actor.id, move_action));
                }
            }

            Some(PlayerAction::SaveEffort(actor.id))
        }

        ReadyState::AllocateEffort => {
            let th = actor
                .prepared_action
                .as_ref()
                .map(|a| a.charge_threshold())
                .unwrap_or(D6(7));

            if actor.can_afford_effort(th) {
                Some(PlayerAction::ModifyCharge(actor.id, 1))
            } else if actor.available_effort() > 1 {
                Some(PlayerAction::CombineEffortDice(actor.id))
            } else {
                Some(PlayerAction::SaveEffort(actor.id))
            }
        }

        ReadyState::Done => None,
    }
}

// fn zombi_action(a: &Actor, cw: CoreWorld) -> Act {
//     for ta in find_enemies(&a, &cw) {
//         let attacks = possible_attacks(a, &ta, &cw)
//             .drain(..)
//             // exclude attacks where the required efford exeeds the avaliable efford (this could lead to a self-KO)
//             .filter(|(attack, _)| attack.required_effort <= a.available_effort())
//             .collect::<Vec<_>>();

//         if let Some((attack, attack_vector)) = pick_one(attacks) {
//             return Act::attack(ta.id, attack, attack_vector, ta.name);
//         }

//         if let Some(path) = find_path_towards(a, &ta, &cw) {
//             let p = path
//                 .iter()
//                 .take(a.move_distance().into())
//                 .cloned()
//                 .collect();

//             return Act::move_to(move_effort(a, &p), p);
//         }
//     }

//     Act::pass()
// }

pub fn possible_player_actions(actor: &Actor, cw: &CoreWorld) -> PlayerActionOptions {
    let mut result = HashMap::new();

    match &actor.state {
        ReadyState::ExecutePreparedAction => {
            add_change_active_actor_options(actor, cw, &mut result);

            if let Some(a) = actor.prepared_action.as_ref() {
                // allow executing the prepared action ...
                add_option(
                    actor.pos,
                    PlayerAction::TriggerAction(actor.id, a.clone()),
                    &mut result,
                );
                // ... wait a little bit more and boost the action further
                add_option(
                    actor.pos,
                    PlayerAction::PrepareAction(actor.id, a.clone()),
                    &mut result,
                );
            }
        }

        ReadyState::SelectAction => {
            add_change_active_actor_options(actor, cw, &mut result);
            add_move_to_options(actor, cw, &mut result);
            add_combat_options(actor, cw, &mut result);
            add_noop_option(actor, &mut result);
        }

        ReadyState::AllocateEffort => {
            add_options_to_allocate_effort_in_combat(actor, &cw, &mut result);
            add_combine_effort_option(actor, &mut result);
            add_noop_option(actor, &mut result);
        }

        ReadyState::Done => {
            add_change_active_actor_options(actor, &cw, &mut result);
        }
    }

    result
}

// pub fn actions_at(actor: &Actor, selected_pos: WorldPos, cw: CoreWorld) -> Vec<Act> {
//     let mut result = vec![];

//     if let Some(other_actor) = find_actor_at(&cw, &selected_pos) {
//         if actor.id == other_actor.id {
//             // selected position contains the acting character itself
//             // for (k, t, d) in actor.ability_self() {
//             //     result.push(Act::use_ability(actor.id, k, t, d));
//             // }

//             for attack in actor.attacks() {
//                 result.push(Act::ambush(attack));
//             }

//             result.push(Act::rest());
//         } else {
//             if actor.team == other_actor.team {
//                 if other_actor.can_activate() {
//                     result.push(Act::activate(other_actor.id));
//                 }
//             } else {
//                 for (attack, attack_vector) in possible_attacks(actor, &other_actor, &cw) {
//                     result.push(Act::attack(
//                         other_actor.id,
//                         attack,
//                         attack_vector,
//                         other_actor.name.clone(),
//                     ));
//                 }
//             }
//         }
//     }

//     if actor.can_move() {
//         if let Some(path) = find_path_for(actor, selected_pos, &cw) {
//             if path.len() > 0 && path.len() <= actor.move_distance().into() {
//                 result.push(Act::move_to(move_effort(actor, &path), path));
//             }
//         }
//     }

//     result
// }

// pub fn options_to_allowcate_effort(actor: &Actor, w: &CoreWorld) -> Vec<(DisplayStr, ID, Act, u8)> {
//     let mut result = vec![];

//     match actor.pending_action.as_ref() {
//         Some(act @ Act { .. }) => {}
//         _ => {}
//     }

//     match actor.pending_action.as_ref().map(|a| &a.action) {
//         Some(Action::Attack(id, attack, av, name)) => {
//             allowcate_effort_options_for_attack(actor.id, *id, attack, av, name, w, &mut result);
//         }
//         _ => {}
//     }

//     result
// }

// fn allowcate_effort_options_for_attack(
//     attacker: ID,
//     target: ID,
//     attack: &AttackOption,
//     vector: &AttackVector,
//     name: &String,
//     w: &CoreWorld,
//     options: &mut Vec<(DisplayStr, ID, Act, u8)>,
// ) {
//     match &attack.attack_type {
//         AttackType::Melee(..) => {
//             let th = max(2, 4 + attack.to_hit) as u8;
//             let act = Act::attack(target, attack.clone(), vector.clone(), name.clone());

//             options.push((DisplayStr::new("Boost offence"), attacker, act, th));

//             for (enemy, act) in find_enemy_attacker(attacker, w).drain(..) {
//                 let txt = DisplayStr::new(format!("Counter attack of {}", enemy.name));
//                 options.push((txt, enemy.id, act, th));
//             }
//         }

//         AttackType::Ranged(..) => {}
//     }
// }

// fn find_enemy_attacker(target: ID, w: &CoreWorld) -> Vec<(Actor, Act)> {
//     w.game_objects()
//         .filter_map(|go| {
//             if let GameObject::Actor(a) = go {
//                 if let Some(Act {
//                     action:
//                         Action::Attack(
//                             id,
//                             AttackOption {
//                                 attack_type: AttackType::Melee(..),
//                                 ..
//                             },
//                             ..,
//                         ),
//                     ..
//                 }) = a.pending_action.as_ref()
//                 {
//                     if *id == target {
//                         return Some((a.clone(), a.pending_action.as_ref().unwrap().clone()));
//                     }
//                 }
//             }

//             None
//         })
//         .collect()
//     // for a in w.game_objects() {
//     //     if
//     // }
// }

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

// fn can_trigger_action(a: &ActorAction, w: &CoreWorld) -> bool {
//     match a {
//         ActorAction::Attack { target, ..} => w.get_actor(*target).is_some(),
//         _ => true,
//     }
// }
