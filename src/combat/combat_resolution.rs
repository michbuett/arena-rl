use std::cmp::{max, min};

use bevy::prelude::Entity;

use crate::core::{Attribute, AttributeValues, Card, Deck, ProgressCheck, SkillCheck};

#[derive(Debug)]
pub struct Attack {
    pub speed: Card,
    pub attribute: Attribute,
    pub difficulty: u8,
    pub damage: i16,
    pub attacker: ActorData,
    pub target: Target,
}

#[derive(Debug)]
pub enum Target {
    SingleMelee(ActorData),
}

pub type ActorData = (Entity, AttributeValues);

#[derive(Debug)]
pub enum CombatConsequence {
    /// Deals damage
    Hit { damage: Vec<Card> },

    /// Makes you more vulnarable (e.g. for counter attacks)
    // TODO: Stumble,

    /// Makes your attack easier to be avoided
    ClumsyAttack,
}

// pub struct CombatSequence {
//     action: Attack,
//     fumble: Vec<CombatConsequence>,
//     hit: Vec<CombatConsequence>,
// }

// pub enum AttackResult {
//     MeleeAttack {
//         attacker: entity,
//         target: entity,
//     }
// }

pub type CombatResult = Vec<(Entity, CombatConsequence)>;

pub fn handle_attack(attack: Attack, deck: &mut Deck) -> CombatResult {
    println!("[DEBUG] handle_attack - attack={:?}", attack);

    let result = match attack.target {
        Target::SingleMelee(target) => handle_melee_attack(&attack, &target, deck),
    };

    println!("  => result={:?}", result);
    result
}

pub fn handle_melee_attack(attack: &Attack, target: &ActorData, deck: &mut Deck) -> CombatResult {
    let mut result: CombatResult = vec![];
    let fumble_effect = perform_quality_check(&attack, deck);
    let hit_effect = perform_hit_check(&attack, deck, &fumble_effect);

    if let Some(c) = fumble_effect {
        result.push((attack.attacker.0, c));
    }
    if let Some(c) = hit_effect {
        result.push((target.0, c));
    }

    result
}

fn perform_quality_check(attack: &Attack, deck: &mut Deck) -> Option<CombatConsequence> {
    let target_number = if attack.speed.suite().matches(&attack.attribute) {
        min(attack.difficulty, attack.speed.value_low())
    } else {
        max(attack.difficulty, attack.speed.value_low())
    };

    let fumble_check = SkillCheck {
        attribute: attack.attribute,
        target_number,
    };

    let fumble_result = fumble_check.perform_check(deck, &attack.attacker.1);
    if !fumble_result.is_success() {
        Some(CombatConsequence::ClumsyAttack)
    } else {
        None
    }
}

const TO_HIT_DIFFICULY_DEFAULT: u8 = 7;
const TO_HIT_DIFFICULY_HARD: u8 = 9;

fn perform_hit_check(
    attack: &Attack,
    deck: &mut Deck,
    fumble_effect: &Option<CombatConsequence>,
) -> Option<CombatConsequence> {
    let target_number = if matches!(fumble_effect, Some(CombatConsequence::ClumsyAttack)) {
        TO_HIT_DIFFICULY_HARD
    } else {
        TO_HIT_DIFFICULY_DEFAULT
    };

    let hit_check = SkillCheck {
        target_number,
        attribute: attack.attribute,
    };

    let hit_result = hit_check.perform_check(deck, &attack.attacker.1);
    if !hit_result.is_success() {
        let magnitude = (hit_result.magnitude() as i16 + attack.damage).clamp(0, 10) as u8;
        let damage_check = ProgressCheck { magnitude };
        let damage_result = damage_check.perform_check(deck);

        Some(CombatConsequence::Hit {
            damage: damage_result.draw,
        })
    } else {
        None
    }
}
