use std::cmp::max;
use std::collections::HashMap;

use super::actor::*;
use super::traits::HitEffect as AttackHitEffect;

use crate::core::cards::SuiteSubstantiality::*;
use crate::core::{dice::*, resolve_challenge, Challenge, Deck, MapPos, Obstacle, Suite, WorldPos};

#[derive(Debug, Clone)]
pub struct Cover {
    pub obscured: u8,
    pub last_obstacle: Option<(MapPos, i8, Option<ID>)>,
}

impl Cover {
    pub fn none() -> Self {
        Self {
            obscured: 0,
            last_obstacle: None,
        }
    }

    pub fn add_obstacle(self, obs: Obstacle, pos: MapPos, id: Option<ID>) -> Self {
        match obs {
            Obstacle::Blocker => Self {
                obscured: 100,
                last_obstacle: Some((pos, i8::MAX, id)),
            },

            Obstacle::Impediment(o, max_block) => {
                let curr_block = self.last_obstacle.map(|(_, b, _)| b).unwrap_or(0);
                Self {
                    obscured: max(self.obscured, o.get()),
                    last_obstacle: Some((pos, max(curr_block, max_block), id)),
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct AttackTarget {
    pub pos: MapPos,
    pub is_target: bool,
    pub actor: Option<Actor>,
    pub cover: Cover,
}

#[derive(Clone, Debug)]
pub struct HitResult {
    pub attack: Attack,
    pub hits: Vec<Hit>,
}

#[derive(Clone, Debug)]
pub struct Hit {
    pub roll: Roll,
    pub pos: MapPos,
    pub effects: Vec<HitEffect>,
}

#[derive(Clone, Debug)]
pub enum HitEffect {
    Miss(),

    Block(MapPos, ID),

    Wound(Wound, ID),

    ForceMove {
        id: ID,
        dx: i32,
        dy: i32,
        distance: u8,
    },
}

pub fn resolve_combat_new(
    attack: &Attack,
    attacker: &Actor, // required for team id => maybe adde to Attack?
    targets: Vec<AttackTarget>,
    decks: &mut HashMap<TeamId, Deck>,
) -> HitResult {
    let mut hits = vec![];

    for t in targets {
        if let Some(target_actor) = t.actor {
            if t.is_target {
                hits.push(resolve_attack(
                    attack,
                    attacker,
                    &target_actor,
                    decks,
                    t.pos,
                ));
            } else {
                // accidentally caught in the line of fire
            }
        }
    }

    HitResult {
        attack: attack.clone(),
        hits,
    }
}

pub fn resolve_attack(
    attack: &Attack,
    attacker: &Actor,
    target: &Actor,
    decks: &mut HashMap<TeamId, Deck>,
    pos: MapPos,
) -> Hit {
    let quality =
        attacker.skill(attack.challenge_suite) + attack.effort_card.value(attack.challenge_suite);

    // STEP defender flips agains the attack to determine if the attack hits
    let defence_result = resolve_challenge(
        Challenge {
            target_num: quality,
            advantage: 0,
            challenge_type: defence_suite(attack),
            skill_val: target.skill(Suite::Spades),
        },
        decks.get_mut(&target.team).unwrap(),
    );

    println!(
        "\n[DEBUG COMBAT] {} attacks with '{}': {} (effort: {:?})",
        attacker.name, attack.name, quality, attack.effort_card
    );
    println!(" - {} defends: {:?}", target.name, defence_result);

    let effects = if defence_result.success_lvl > 0 {
        println!(" - no hit");
        vec![HitEffect::Miss()]
    } else {
        println!(" - Hit! (advantage: {})", -1 * defence_result.success_lvl);

        // STEP attacker flips against defenders armor to determine if/how much
        // damage the attack causes
        let dmg_result = resolve_challenge(
            Challenge {
                advantage: -1 * defence_result.success_lvl,
                challenge_type: damage_suite(attack),
                skill_val: attack.damage,
                target_num: target.soak(),
            },
            decks.get_mut(&attacker.team).unwrap(),
        );
        println!(" - check for damage: {:?} ", dmg_result);

        let mut effects = if dmg_result.success_lvl <= -2 {
            // armor more then twice as high as damage
            // => the hit was completely negated
            vec![HitEffect::Block(pos, target.id)]
        } else {
            let w = Wound {
                pain: 1,
                wound: i8::max(0, dmg_result.success_lvl) as u8,
            };

            vec![HitEffect::Wound(w, target.id)]
        };

        add_attack_effects(HitEffectCondition::OnHit, attack, &target, &mut effects);

        effects
    };

    let roll = Roll::new(0, 0); // deprecated
    Hit { pos, roll, effects }
}

fn defence_suite(attack: &Attack) -> Suite {
    if let Physical = attack.challenge_suite.substantiality() {
        Suite::Spades
    } else {
        Suite::Diamonds
    }
}

fn damage_suite(attack: &Attack) -> Suite {
    if let Physical = attack.challenge_suite.substantiality() {
        Suite::Clubs
    } else {
        Suite::Hearts
    }
}

// pub fn resolve_combat(attack: &Attack, vector: Vec<AttackTarget>) -> HitResult {
//     HitResult {
//         attack: attack.clone(),
//         hits: resolve_hits(attack, vector),
//     }
// }

// fn resolve_hits(attack: &Attack, mut vector: Vec<AttackTarget>) -> Vec<Hit> {
//     let mut result = vec![];

//     for t in vector.drain(..) {
//         if t.is_target {
//             if let Some(target_actor) = t.actor {
//                 result.push(resolve_hit(&attack, &target_actor, t.pos, &t.cover));
//             }
//         }
//     }

//     result
// }

// fn resolve_hit(attack: &Attack, target_actor: &Actor, pos: MapPos, cover: &Cover) -> Hit {
//     let attack_roll = roll_to_hit(attack, target_actor, cover);

//     Hit {
//         pos,
//         effects: calculate_hit_effects(attack_roll.num_successes, attack, target_actor, cover),
//         roll: attack_roll,
//     }
// }

// fn calculate_hit_effects(
//     mut attack_quality: u8,
//     attack: &Attack,
//     target_actor: &Actor,
//     cover: &Cover,
// ) -> Vec<HitEffect> {
//     if attack_quality == 0 {
//         return vec![HitEffect::Miss()];
//     }

//     if let Some((block_pos, block_mod)) = determine_blocking(attack, target_actor, cover) {
//         attack_quality = max(0, attack_quality as i8 - block_mod) as u8;

//         if attack_quality == 0 {
//             return vec![HitEffect::Block(block_pos, target_actor.id)];
//         }
//     }

//     let mut effects = vec![];

//     add_attack_effects(
//         HitEffectCondition::OnHit,
//         attack,
//         target_actor,
//         &mut effects,
//     );

//     add_wound_effects(attack_quality as i8 - 2, attack, target_actor, &mut effects);

//     effects
// }

// fn add_wound_effects(
//     roll_advantage: i8,
//     attack: &Attack,
//     target_actor: &Actor,
//     effects: &mut Vec<HitEffect>,
// ) {
//     let wound_roll = Roll::new(roll_advantage, to_wound_threshold(attack, &target_actor));
//     let wound_quality =
//         wound_roll.num_successes as i8 + attack.rend - target_actor.attr(Attr::Resilience).val();

//     if wound_quality > 0 {
//         let w = Wound {
//             pain: 1,
//             wound: wound_quality as u8 - 1,
//         };

//         effects.push(HitEffect::Wound(w, target_actor.id));
//     }
// }

fn add_attack_effects(
    when_cond: HitEffectCondition,
    attack: &Attack,
    target_actor: &Actor,
    effects: &mut Vec<HitEffect>,
) {
    if let Some(attack_eff_list) = &attack.effects {
        for (cond, eff) in attack_eff_list {
            if when_cond == *cond {
                effects.push(convert_attack_hit_effect(eff, attack, target_actor));
            }
        }
    }
}

fn convert_attack_hit_effect(
    eff: &AttackHitEffect,
    attack: &Attack,
    target_actor: &Actor,
) -> HitEffect {
    match eff {
        AttackHitEffect::PushBack(d) => {
            let (dx, dy) = direction(attack.origin_pos, target_actor.pos);

            HitEffect::ForceMove {
                id: target_actor.id,
                dx,
                dy,
                distance: *d,
            }
        }

        AttackHitEffect::PullCloser(d) => {
            let (dx, dy) = direction(target_actor.pos, attack.origin_pos);

            HitEffect::ForceMove {
                id: target_actor.id,
                dx,
                dy,
                distance: *d,
            }
        }
    }
}

// fn roll_to_hit(attack: &Attack, target_actor: &Actor, cover: &Cover) -> Roll {
//     match attack.attack_type {
//         AttackType::Melee(..) => Roll::new(
//             attack.advantage,
//             melee_to_hit_threshold(attack, target_actor),
//         ),

//         AttackType::Ranged(..) => Roll::new(
//             attack.advantage,
//             ranged_to_hit_threshold(attack, target_actor, cover),
//         ),
//     }
// }

// fn determine_blocking(
//     attack: &Attack,
//     target_actor: &Actor,
//     cover: &Cover,
// ) -> Option<(MapPos, i8)> {
//     match attack.attack_type {
//         AttackType::Melee(..) => {
//             let block = target_actor.attr(Attr::MeleeBlock).val();
//             if block == 0 {
//                 None
//             } else {
//                 Some((MapPos::from_world_pos(target_actor.pos), block))
//             }
//         }

//         AttackType::Ranged(..) => {
//             if let Some((pos, max_block, _)) = cover.last_obstacle {
//                 if pos.distance(MapPos::from_world_pos(target_actor.pos)) == 1 {
//                     return Some((pos, max_block));
//                 }
//             }

//             None
//         }
//     }
// }

// fn melee_to_hit_threshold(attack: &Attack, target: &Actor) -> u8 {
//     if !target.is_concious() {
//         return 2;
//     }

//     let offence_mod = attack.to_hit.val();
//     let defence_mod = target.attr(Attr::MeleeSkill).val() + target.attr(Attr::Evasion).val();
//     let th = 4 + defence_mod - offence_mod;

//     max(2, th) as u8
// }

// fn ranged_to_hit_threshold(attack: &Attack, target: &Actor, cover: &Cover) -> u8 {
//     let base_th = match cover.obscured {
//         0..=24 => 2,
//         25..=49 => 3,
//         50..=74 => 4,
//         75..=99 => 5,
//         _ => {
//             // If target is completly obscured than it is not possible to hit it
//             // even for the most skilled marksman
//             // => exit here
//             return 7;
//         }
//     };

//     let th = base_th + target.attr(Attr::Evasion).val() - attack.to_hit.val();
//     max(2, th) as u8
// }

// fn to_wound_threshold(attack: &Attack, target: &Actor) -> u8 {
//     max(
//         0,
//         4 + target.attr(Attr::Protection).val() - attack.to_wound.val(),
//     ) as u8
// }

fn direction(p1: WorldPos, p2: WorldPos) -> (i32, i32) {
    let mp1 = MapPos::from_world_pos(p1);
    let mp2 = MapPos::from_world_pos(p2);
    let dx = mp2.0 - mp1.0;
    let dy = mp2.1 - mp1.1;
    (dx, dy)
}
