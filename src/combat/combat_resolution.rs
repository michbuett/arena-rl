// use std::cmp::{max, min};

use bevy::prelude::Entity;

use crate::core::{
    ActionCheck, AttackData, Card, CheckResult, Complication, Deck, EffectiveAttributes,
    ProgressCheckNew, Protection, perform_action_check,
};

#[derive(Debug)]
pub struct Attack {
    pub effort: Card,
    pub attacker: Combatant,
    pub target: Target,
    pub risky_complication: bool,
    pub data: AttackData,
}

#[derive(Debug)]
pub enum Target {
    SingleMelee(Combatant),
}

#[derive(Debug)]
pub struct Combatant {
    pub id: Entity,
    pub attributes: EffectiveAttributes,
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

    /// A clumsy attack thows you off balance
    ClumsyAttack,

    ArmorBreak,
}

#[derive(Debug)]
pub struct CombatResult {
    pub attack_quality_result: CheckResult,
    pub consequences: Vec<(Entity, CombatConsequence)>,
}

// pub type CombatResult = Vec<(Entity, CombatConsequence)>;

pub fn handle_attack(attack: Attack, deck: &mut Deck) -> CombatResult {
    // println!("[DEBUG] handle_attack - attack={:?}", attack);

    let result = match &attack.target {
        Target::SingleMelee(target) => handle_melee_attack(&attack, &target, deck),
    };

    // println!("  => result={:?}", result);
    result
}

pub fn handle_melee_attack(attack: &Attack, target: &Combatant, deck: &mut Deck) -> CombatResult {
    let mut consequences = vec![];
    let check = ActionCheck {
        req_attributes: attack.data.req_attributes,
        req_effort: attack.data.req_effort,
        risk_complication: attack.risky_complication,
    };

    let attack_quality_result =
        perform_action_check(check, attack.effort, &attack.attacker.attributes, deck);

    match attack_quality_result.complication {
        Complication::None => {}
        _ => {
            consequences.push((attack.attacker.id, CombatConsequence::ClumsyAttack));
        }
    }

    let hit_effect = if attack_quality_result.success {
        determine_damage(attack, &target.protection, deck)
    } else {
        vec![]
    };

    for c in hit_effect {
        consequences.push((target.id, c));
    }

    CombatResult {
        consequences,
        attack_quality_result,
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
        .checked_sub(attack.data.penetration)
        .unwrap_or(0);

    let damage_result = ProgressCheckNew {
        num_cards: attack.data.damage,
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
