pub use super::actor::*;
pub use super::traits::*;
use std::cmp::max;

use crate::core::dice::*;
use crate::core::{MapPos, Obstacle};

pub struct AttackTarget<T: Clone> {
    pub pos: MapPos,
    pub target_ref: T,
    pub target_actor: Option<Actor>,
    pub is_target: bool,
    pub obstacle: Obstacle,
}

#[derive(Clone, Debug)]
pub struct HitResult<T: Clone> {
    pub attack: Attack,
    pub hit_roll: Roll,
    pub hits: Vec<Hit<T>>,
}

#[derive(Clone, Debug)]
pub struct Hit<T: Clone> {
    pub pos: MapPos,
    pub target: T,
    pub accicental_hit: bool,
    pub wound: Option<WoundResult>,
    pub defence: Option<Roll>,
}

#[derive(Clone, Debug)]
pub struct WoundResult {
    pub target: Actor,
    pub wound: Option<Wound>,
}

pub fn resolve_combat<T: Clone>(attack: &Attack, vector: Vec<AttackTarget<T>>) -> HitResult<T> {
    let hit_roll = Roll::new(attack.num_dice, attack.to_hit.val());
    let attack_power = hit_roll.result().checked_sub(attack.difficulty).unwrap_or(0);
    // TODO: apply negativ effects if attack_power < attack difficulty

    HitResult {
        attack: attack.clone(),
        hit_roll,
        hits: resolve_hits(attack_power, &attack, vector),
    }
}

fn resolve_hits<T: Clone>(
    mut attack_power: u8,
    attack: &Attack,
    mut vector: Vec<AttackTarget<T>>,
) -> Vec<Hit<T>> {
    let mut hits = vec![];
    let mut cover: (MapPos, u8) = (MapPos(-1000, -1000), 0);

    for target in vector.drain(..) {
        if target.is_target {
            // this is the actual target of the attack
            if let Some(actor) = target.target_actor {
                let cover_mod = if target.pos.distance(cover.0) > 1 {
                    0
                } else {
                    cover.1
                };

                hits.push(resolve_hitting_target(
                    target.pos,
                    target.target_ref,
                    actor,
                    attack_power,
                    attack,
                    cover_mod,
                ));
            } else {
                panic!("You targeted a Non-Actor. This should not happen!");
            }
        } else {
            // this is an accicental hit (e.g. hitting an obstacle which is in the way)
            let obstacle_difficulty = max(0, target.obstacle.0) as u8;
            if attack_power < obstacle_difficulty {
                // the obstacle could not be avoided
                let wound = if let Some(actor) = target.target_actor {
                    Some(resolve_wound(attack, actor, 1))
                } else {
                    None
                };

                hits.push(Hit {
                    target: target.target_ref,
                    pos: target.pos,
                    accicental_hit: true,
                    wound,
                    defence: None,
                });
            }

            attack_power = attack_power.checked_sub(obstacle_difficulty).unwrap_or(0);
            cover = (target.pos, obstacle_difficulty);
        }

        if attack_power <= 0 {
            return hits;
        }
    }

    hits
}

fn resolve_hitting_target<T: Clone>(
    pos: MapPos,
    target_ref: T,
    target_actor: Actor,
    attack_power: u8,
    attack: &Attack,
    cover_mod: u8,
) -> Hit<T> {
    if attack_power == 0 {
        // the attacker missed
        return Hit {
            pos,
            target: target_ref,
            accicental_hit: false,
            wound: None,
            defence: None,
        };
    }

    let max_defence_effort = target_actor.available_effort() as usize + 1;
    let defence_modifier = get_defence_modifier(attack, cover_mod, &target_actor);
    let mut defence_power = 0;
    let mut defence_dice = vec![];

    while defence_dice.len() < max_defence_effort && defence_power < attack_power {
        let roll = D6::roll();

        defence_power += roll.0;
        defence_dice.push(roll);
    }

    let target_actor = target_actor.use_effort(defence_dice.len() as u8);

    let to_wound_result = if attack_power > defence_power {
        // its a hit!
        // => roll to wound
        let num_def_dice = attack_power / max(1, defence_power);
        resolve_wound(attack, target_actor, num_def_dice)
    } else {
        // targets defence was successful
        // => no wound, no pain, just the effort used for the defence
        WoundResult {
            target: target_actor,
            wound: None,
        }
    };

    Hit {
        pos,
        target: target_ref,
        accicental_hit: false,
        wound: Some(to_wound_result),
        defence: Some(Roll::from_dice(defence_dice, defence_modifier)),
    }
}

fn get_defence_modifier(attack: &Attack, cover: u8, target_actor: &Actor) -> i8 {
    match attack.attack_type {
        AttackType::Melee(..) => target_actor.attr(Attr::MeleeDefence).val(),
        AttackType::Ranged(..) => -3 + cover as i8 + target_actor.attr(Attr::RangeDefence).val(),
    }
}

fn resolve_wound(attack: &Attack, mut target: Actor, hit_dice: u8) -> WoundResult {
    let roll = Roll::new(hit_dice, attack.to_wound.val());
    let phys_strength = target.attr(Attr::Physical).abs_val();
    let protection = target.attr(Attr::Protection).abs_val();
    let resistence = max(1, phys_strength + protection);
    let wound = if roll.result() < resistence {
        if resistence >= 2 * roll.result() {
            // the hit only scratched the armor
            // => no damage whatsoever
            None
        } else {
            // it was a hit but the armor was not penetrated
            // => the target feels pain, but no real damage was done
            Some(Wound { pain: 1, wound: 0 })
        }
    } else {
        // The hit penetrates the armor
        // => a real wound is the result
        let w = roll.result() / resistence;
        Some(Wound { pain: w, wound: w })
    };

    if let Some(w) = wound.clone() {
        target = target.wound(w);
    }

    WoundResult { wound, target }
}

// pub fn resolve_to_wound(hit: Hit<Actor>) -> ToWoundResult {
//     // let to_wound_adv = RollAdvantage::new(
//     //     hit.attack.to_wound.val(),
//     //     hit.target.attr(Attr::Protection).val(),
//     // );

//     let (num_hits, defence) = handle_defence(&hit);
//     let wound_roll = Roll::new(num_hits); // TODO get difficulty to wound from armor stat
//     let wound = Wound::from_wound_roll(&wound_roll);

//     // println!(
//     //     "[DEBUG] resolve_to_wound *** \n\tadvantage: {:?}, \n\troll: {:?}, \n\twound: {:?}",
//     //     to_wound_adv, wound_roll, wound
//     // );

//     ToWoundResult {
//         target: hit.target.wound(wound),
//         roll: wound_roll,
//         // defence,
//     }
// }

// fn handle_defence(hit: &Hit<Actor>) -> (u8, Option<(Defence, Roll)>) {
//     // TODO implement new defence mechanic
//     (0, None)

//     // if let Some(d) = hit.target.defence(&hit.attack) {
//     //     let a = &hit.attack;
//     //     let adv = match d.defence_type {
//     //         DefenceType::Block => RollAdvantage::new(d.defence.val(), a.to_wound.val()),

//     //         DefenceType::Dodge(..) => RollAdvantage::new(d.defence.val(), a.to_hit.val()),

//     //         DefenceType::Parry => {
//     //             RollAdvantage::new(d.defence.val(), max(a.to_hit.val(), a.to_wound.val()))
//     //         }

//     //         // TODO: use skills for cover mechanic
//     //         DefenceType::TakeCover => RollAdvantage::new(0, 0),
//     //     };

//     //     let def_roll = Roll::new(d.num_dice, adv);
//     //     let num_hits = hit
//     //         .roll
//     //         .successes()
//     //         .checked_sub(def_roll.successes())
//     //         .unwrap_or(0);

//     //     (num_hits, Some((d, def_roll)))
//     // } else {
//     //     (hit.roll.successes(), None)
//     // }
// }
