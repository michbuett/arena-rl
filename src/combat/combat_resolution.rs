use std::cmp::{max, min};

use bevy::prelude::Entity;

use crate::core::{
    AttributeType, Attributes, Card, Deck, Health, ProgressCheckNew, Protection, SkillCheck,
};

#[derive(Debug)]
pub struct Attack {
    pub speed: Card,
    pub attribute: AttributeType,
    pub difficulty: u8,
    pub damage: u8,
    pub penetration: u8,
    pub attacker: Combatant,
    pub target: Target,
}

#[derive(Debug)]
pub enum Target {
    SingleMelee(Combatant),
}

#[derive(Debug)]
pub struct Combatant {
    pub id: Entity,
    pub attributes: Attributes,
    pub health: Health,
    pub protection: Protection,
}

#[derive(Debug)]
pub enum CombatConsequence {
    /// Deals damage
    Wound {
        damage: Vec<Card>,
    },

    /// Makes you more vulnarable (e.g. for counter attacks)
    // TODO: Stumble,

    /// Makes your attack easier to be avoided
    ClumsyAttack,

    ArmorBreak,
}

pub type CombatResult = Vec<(Entity, CombatConsequence)>;

pub fn handle_attack(attack: Attack, deck: &mut Deck) -> CombatResult {
    // println!("[DEBUG] handle_attack - attack={:?}", attack);

    let result = match &attack.target {
        Target::SingleMelee(target) => handle_melee_attack(&attack, &target, deck),
    };

    // println!("  => result={:?}", result);
    result
}

pub fn handle_melee_attack(attack: &Attack, target: &Combatant, deck: &mut Deck) -> CombatResult {
    let mut result: CombatResult = vec![];
    let fumble_effect = perform_quality_check(&attack, deck);
    let hit_effect = perform_hit_check(&attack, &target.protection, deck, &fumble_effect);

    if let Some(c) = fumble_effect {
        result.push((attack.attacker.id, c));
    }

    for c in hit_effect {
        result.push((target.id, c));
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

    let attributes = attack
        .attacker
        .health
        .current_attribute_values(attack.attacker.attributes);
    let fumble_result = fumble_check.perform_check(deck, &attributes);
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
    protection: &Protection,
    deck: &mut Deck,
    fumble_effect: &Option<CombatConsequence>,
) -> Vec<CombatConsequence> {
    let target_number = if matches!(fumble_effect, Some(CombatConsequence::ClumsyAttack)) {
        TO_HIT_DIFFICULY_HARD
    } else {
        TO_HIT_DIFFICULY_DEFAULT
    };

    let hit_check = SkillCheck {
        target_number,
        attribute: attack.attribute,
    };

    let attributes = attack
        .attacker
        .health
        .current_attribute_values(attack.attacker.attributes);
    let hit_result = hit_check.perform_check(deck, &attributes);
    if hit_result.is_success() {
        determine_damage(attack, protection, deck)
    } else {
        vec![]
    }
}

fn determine_damage(
    attack: &Attack,
    protection: &Protection,
    deck: &mut Deck,
) -> Vec<CombatConsequence> {
    let mut result = vec![];
    let resistence = protection
        .total_resistance()
        .checked_sub(attack.penetration)
        .unwrap_or(0);

    let damage_result = ProgressCheckNew {
        num_cards: attack.damage,
        resistence,
    }
    .perform_check(deck);

    for blocked_dmg in damage_result.discard.iter() {
        if blocked_dmg.value_high() >= 10 {
            result.push(CombatConsequence::ArmorBreak);
        }
    }

    if !damage_result.draw.is_empty() {
        result.push(CombatConsequence::Wound {
            damage: damage_result.draw,
        });
    }

    result
}
