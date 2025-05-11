use crate::core::{resolve_challenge, Card, Challenge, Deck, Suite, SUCCESS_THRESHOLD};

#[derive(Debug)]
pub struct Attack {
    pub damage: i16,
    pub challenge: Challenge,
}

#[derive(Debug)]
pub struct Defence {
    pub armor: i16,
    pub challenge: Challenge,
}

// pub struct CombatAction {
//     pub target_suite: Suite,
//     pub target_value: u8,
// }

pub enum CombatResult {
    Fumble,
    Defence,
    Bounced,
    Wounded(Card),
    OutOfAction,
}

pub fn handle_attack(
    attack: Attack,
    effort_card: Card,
    attack_deck: &mut Deck,
    defence: Defence,
    defence_deck: &mut Deck,
) -> CombatResult {
    println!(
        "\n[DEBUG] handle_attack\n  - effort={:?}\n  - attack={:?}\n  - defence={:?}",
        effort_card, attack, defence
    );

    // step 1: see if attack needs a fumble check
    let effort = effort_card.value(attack.challenge.target_suite);
    let attack_quality = effort + attack.challenge.skill_value;
    if (attack_quality as i16) < SUCCESS_THRESHOLD {
        // step 1.1 if yes, then perform fumble check
        let fumble_result = resolve_challenge(
            &Challenge {
                advantage: attack.challenge.advantage,
                target_suite: attack.challenge.target_suite,
                skill_value: attack_quality,
            },
            attack_deck,
        );

        if fumble_result.success_lvl < 0 {
            // step 1.2 if fumble check fails => apply effects (possible exit)
            println!(
                "[DEBUG] handle_attack - FUMBLE - fumble_result={:?}",
                fumble_result
            );
            return CombatResult::Fumble;
        }
    }

    // step 2: check if defence succeeds (possible exit)
    let defence_result = resolve_challenge(&defence.challenge, defence_deck);
    if defence_result.success_lvl > 0 {
        // defence succeeds
        println!(
            "[DEBUG] handle_attack - DEFENCE - defence_result={:?}",
            defence_result
        );
        return CombatResult::Defence;
    }

    // step 3: flip for damage
    let damage_flip = attack_deck.deal();
    let damage_value = damage_flip.value(Suite::Any) as i16 + attack.damage - defence.armor;

    println!(
        "[DEBUG] handle_attack - damage_flip={:?}, damage_value={}",
        damage_flip, damage_value
    );

    if damage_value > 10 {
        CombatResult::OutOfAction
    } else if damage_value > 0 {
        CombatResult::Wounded(damage_flip)
    } else {
        CombatResult::Bounced
    }
}
