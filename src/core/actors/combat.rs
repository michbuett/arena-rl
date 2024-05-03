use std::cmp::max;
use std::collections::HashMap;

use super::actor::*;
use super::traits::HitEffect as AttackHitEffect;

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
    let quality = attacker.skill(attack.to_hit.0, attack.to_hit.1)
        + attack.effort_card.value(attack.to_hit.0);

    // STEP defender flips agains the attack to determine if the attack hits
    let defence_result = resolve_challenge(
        Challenge {
            target_num: quality,
            advantage: 0,
            challenge_type: attack.defence,
            skill_val: target.skill(Suite::PhysicalAg, 0),
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
                challenge_type: attack.to_wound.0,
                skill_val: attacker.skill(attack.to_wound.0, attack.to_wound.1),
                target_num: max(3, target.soak().checked_sub(attack.rend).unwrap_or(0)),
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

fn direction(p1: WorldPos, p2: WorldPos) -> (i32, i32) {
    let mp1 = MapPos::from_world_pos(p1);
    let mp2 = MapPos::from_world_pos(p2);
    let dx = mp2.0 - mp1.0;
    let dy = mp2.1 - mp1.1;
    (dx, dy)
}
