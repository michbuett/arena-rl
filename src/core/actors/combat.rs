pub use super::traits::*;
pub use super::actor::*;

use crate::core::dice::*;

#[derive(Debug, Clone)]
pub struct CombatResult {
    pub attack: Attack,
    pub attacker: Actor,
    pub target: Actor,
    pub hit_roll: Roll,
    pub wound_roll: Roll,
}

pub fn resolve_attack(attack: AttackOption, attacker: Actor, target: Actor) -> CombatResult {
    let attack = attack.into_attack(&attacker);
    let to_hit_adv = RollAdvantage::new(attack.to_hit.val(), target.attr(Attr::RangeDefence).val());
    let hit_roll = Roll::new(attack.num_dice, to_hit_adv);
    let to_wound_adv = RollAdvantage::new(attack.to_wound.val(), target.attr(Attr::Protection).val());
    let wound_roll = Roll::new(hit_roll.successes(), to_wound_adv);
    let target = target.wound(Wound::from_wound_roll(&wound_roll));

    CombatResult {
        attack,
        attacker,
        target,
        hit_roll,
        wound_roll,
    }
}
