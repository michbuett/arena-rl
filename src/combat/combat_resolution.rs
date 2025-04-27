use crate::core::{resolve_challenge, Card, Challenge, Deck, Suite};

pub struct Attack {
    pub damage: u8,
    pub challenge: Challenge,
}

pub struct Defence {
    pub armor: u8,
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
    // step 1: see if attack needs a fumble check
    let effort = effort_card.value(attack.challenge.target_suite);
    if effort < attack.challenge.target_value {
        // step 1.1 if yes, then perform fumble check
        let fumble_result = resolve_challenge(&attack.challenge, attack_deck);
        if fumble_result.success_lvl < 0 {
            // step 1.2 if fumble check fails => apply effects (possible exit)
            return CombatResult::Fumble;
        }
    }

    // step 2: check if defence succeeds (possible exit)
    let defence_result = resolve_challenge(&defence.challenge, defence_deck);
    if defence_result.success_lvl > 0 {
        // defence succeeds
        return CombatResult::Defence;
    }

    // step 3: flip for damage
    let damage_flip = attack_deck.deal();
    let damage_value =
        damage_flip.value(Suite::Any) as i16 + attack.damage as i16 - defence.armor as i16;

    if damage_value > 10 {
        CombatResult::OutOfAction
    } else if damage_value > 0 {
        CombatResult::Wounded(damage_flip)
    } else {
        CombatResult::Bounced
    }
}
