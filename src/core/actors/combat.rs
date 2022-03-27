use std::cmp::max;

pub use super::actor::*;
pub use super::traits::*;
use super::traits::HitEffect as AttackHitEffect;

use crate::core::WorldPos;
use crate::core::dice::*;
use crate::core::{MapPos, Obstacle};

#[derive(Debug)]
pub struct AttackTarget {
    pub pos: MapPos,
    pub target: Option<Actor>,
    pub is_target: bool,
    pub obstacle: Obstacle,
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
    Wound(Wound, ID),

    Miss(),

    Defence(Roll, ID),

    ForceMove {
        id: ID,
        dx: i32,
        dy: i32,
        distance: u8,
    },
}

pub fn resolve_combat(attack: &Attack, vector: Vec<AttackTarget>) -> HitResult {
    HitResult {
        attack: attack.clone(),
        hits: resolve_hits(attack, vector),
    }
}

fn resolve_hits(attack: &Attack, mut vector: Vec<AttackTarget>) -> Vec<Hit> {
    let mut result = vec![];
    let mut remaing_effort = attack.num_dice;

    for t in vector.drain(..) {
        if remaing_effort == 0 {
            return result;
        }

        if t.is_target {
            if let Some(target_actor) = t.target {
                let hit = resolve_hit_at_target(&attack, target_actor, t.pos, remaing_effort);

                remaing_effort = hit.roll.num_fails;
                result.push(hit);
            }
        } else {
            let hit = resolve_hit_at_obstacle(t.pos, remaing_effort, t.obstacle.0, &attack, t.target);

            if let Some(hit) = hit {
                remaing_effort = hit.roll.num_successes;
                result.push(hit);
            }
        }
    }

    result
}

fn resolve_hit_at_target(attack: &Attack, target_actor: Actor, pos: MapPos, effort: u8) -> Hit {
    let attack_roll = Roll::new(effort, to_hit_threshold(attack, &target_actor));

    Hit {
        pos,
        effects: calculate_hit_effects(attack_roll.num_successes, attack, target_actor),
        roll: attack_roll,
    }
}

fn resolve_hit_at_obstacle(
    pos: MapPos,
    effort: u8,
    difficulty: u8,
    attack: &Attack,
    target: Option<Actor>,
) -> Option<Hit> {
    let roll = Roll::new(effort, difficulty);
    let num_hits = roll.num_fails; // a failed attack roll on an obstacle means an accicental hit

    println!("Hit obstacle (difficulty={}, roll={:?}, num hits={})", difficulty, roll, num_hits);

    if num_hits > 0 {
        let effects = if let Some(target) = target {
            calculate_hit_effects(num_hits, attack, target)
        } else {
            vec![]
        };

        Some(Hit { pos, roll, effects })
    } else {
        None
    }
}

fn calculate_hit_effects(num_hits: u8, attack: &Attack, target_actor: Actor) -> Vec<HitEffect> {
    if num_hits == 0 {
        return vec![HitEffect::Miss()];
    }

    let mut effects = vec![];
    let num_hits = if let Some(defence_roll) = roll_defence(&target_actor) {
        let successes = defence_roll.num_successes;
        effects.push(HitEffect::Defence(defence_roll, target_actor.id));
        num_hits.checked_sub(successes).unwrap_or(0)
    } else {
        num_hits
    };

    if num_hits > 0 {
        let wound_roll = Roll::new(num_hits, to_wound_threshold(attack, &target_actor));
        let w = Wound {
            pain: num_hits,
            wound: wound_roll.num_successes,
        };

        effects.push(HitEffect::Wound(w, target_actor.id));

        if let Some(attack_eff_list) = &attack.effects {
            for (cond, eff) in attack_eff_list {
                match cond {
                    HitEffectCondition::OnHit => {
                        match eff {
                            AttackHitEffect::PushBack(d) => {
                                let (dx, dy) = direction(attack.origin_pos, target_actor.pos);
                                effects.push(HitEffect::ForceMove {
                                    id: target_actor.id,
                                    dx,
                                    dy,
                                    distance: *d,
                                    
                                })
                            }

                            AttackHitEffect::PullCloser(d) => {
                                let (dx, dy) = direction(target_actor.pos, attack.origin_pos);
                                effects.push(HitEffect::ForceMove {
                                    id: target_actor.id,
                                    dx,
                                    dy,
                                    distance: *d,
                                    
                                })
                            }
                        }
                    }
                }
            }
        }
    }

    effects
}

fn roll_defence(defender: &Actor) -> Option<Roll> {
    if defender.available_effort() == 0 {
        return None;
    }

    Some(Roll::new(defender.available_effort(), defence_threshold()))
}

fn to_hit_threshold(attack: &Attack, target: &Actor) -> u8 {
    // TODO implement expertise system
    max(
        0,
        4 + attack.to_hit.val() + target.attr(Attr::MeleeDefence).val(),
    ) as u8
}

fn defence_threshold() -> u8 {
    4
}

fn to_wound_threshold(attack: &Attack, target: &Actor) -> u8 {
    max(
        0,
        4 + attack.to_wound.val() + target.attr(Attr::Protection).val(),
    ) as u8
}

fn direction(p1: WorldPos, p2: WorldPos) -> (i32, i32) {
    let mp1 = MapPos::from_world_pos(p1);
    let mp2 = MapPos::from_world_pos(p2);
    let dx = mp2.0 - mp1.0;
    let dy = mp2.1 - mp1.1;
    (dx, dy)
}
