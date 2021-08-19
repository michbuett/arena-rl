pub use super::actor::*;
pub use super::traits::*;
use std::cmp::max;

use crate::core::dice::*;
use crate::core::{MapPos, Obstacle};

pub type AttackVector<T> = Vec<(MapPos, T, Obstacle, bool)>;

#[derive(Debug)]
pub struct Hit<T> {
    pub attack: Attack,
    pub pos: MapPos,
    pub roll: Roll,
    pub target: T,
    pub accicental_hit: bool,
}

impl<A> Hit<A> {
    pub fn set_target<B>(self, target: B) -> Hit<B> {
        Hit {
            target,
            attack: self.attack,
            pos: self.pos,
            roll: self.roll,
            accicental_hit: self.accicental_hit,
        }
    }

    pub fn successes(&self) -> u8 {
        if self.accicental_hit {
            self.roll.fails()
        } else {
            self.roll.successes()
        }
    }
}

pub fn resolve_to_hit<T>(attack: &Attack, mut vector: AttackVector<T>) -> Vec<Hit<T>> {
    let mut remaining_dice = attack.num_dice;
    let mut result = vec![];

    for (pos, target, Obstacle(difficulty), is_target) in vector.drain(..) {
        let to_hit_adv = RollAdvantage::new(attack.to_hit.val(), difficulty);
        let roll = Roll::new(remaining_dice, to_hit_adv);

        if is_target {
            remaining_dice = remaining_dice - roll.normal_successes();
        } else {
            remaining_dice = roll.normal_successes();
        }

        result.push(Hit {
            attack: attack.clone(),
            pos,
            roll,
            target,
            accicental_hit: !is_target,
        });

        // println!("[DEBUG] resolve_to_hit: difficulty={}, roll={:?}", difficulty, roll);
        if remaining_dice == 0 {
            return result;
        }
    }

    result
}

pub struct ToWoundResult {
    pub roll: Roll,
    pub target: Actor,
    pub defence: Option<(Defence, Roll)>,
}

pub fn resolve_to_wound(hit: Hit<Actor>) -> ToWoundResult {
    let to_wound_adv = RollAdvantage::new(
        hit.attack.to_wound.val(),
        hit.target.attr(Attr::Protection).val(),
    );

    let (num_hits, defence) = handle_defence(&hit);

    let wound_roll = Roll::new(num_hits, to_wound_adv);

    ToWoundResult {
        target: hit.target.wound(Wound::from_wound_roll(&wound_roll)),
        roll: wound_roll,
        defence,
    }
}

fn handle_defence(hit: &Hit<Actor>) -> (u8, Option<(Defence, Roll)>) {
    if let Some(d) = hit.target.defence(&hit.attack) {
        let a = &hit.attack;
        let adv = match d.defence_type {
            DefenceType::Block => RollAdvantage::new(d.defence.val(), a.to_wound.val()),

            DefenceType::Dodge(..) => RollAdvantage::new(d.defence.val(), a.to_hit.val()),

            DefenceType::Parry => {
                RollAdvantage::new(d.defence.val(), max(a.to_hit.val(), a.to_wound.val()))
            }

            // TODO: use skills for cover mechanic
            DefenceType::TakeCover => RollAdvantage::new(0, 0),
        };

        let def_roll = Roll::new(d.num_dice, adv);
        let num_hits = hit
            .roll
            .successes()
            .checked_sub(def_roll.successes())
            .unwrap_or(0);

        (num_hits, Some((d, def_roll)))
    } else {
        (hit.roll.successes(), None)
    }
}
