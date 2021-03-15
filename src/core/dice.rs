extern crate rand;

use rand::prelude::*;
use std::cmp::{min, max};

#[derive(Debug, Copy, Clone, Eq, PartialEq, PartialOrd, Ord)]
pub struct D6(pub u8);

impl D6 {
    pub fn new(raw: i8) -> Self {
        Self(max(1, min(6, raw)) as u8)
    }

    pub fn roll() -> Self {
        let range = rand::distributions::Uniform::from(1..=6);
        let mut rng = rand::thread_rng();

        Self(rng.sample(range))
    }

    pub fn modify(self, modifier: i8) -> Self {
        Self::new(self.0 as i8 + modifier)
    }
}

/// A roll result
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum RR {
    /// a failure; the number indicates to malus
    FF(u8), 
    /// a success, but with a catch
    SF, 
    /// a success; the number indicates to bonus
    SS(u8), 
}

fn exploding_rolls(num: u8, modifier: i8) -> u8 {
    let range = rand::distributions::Uniform::from(1..=6);
    let mut rng = rand::thread_rng();
    let mut exploding = 0;
    let mut roll = num;

    while roll == num {
        exploding += 1;
        roll = max(1, min(6, rng.sample(range) + modifier)) as u8;
    }

    exploding
}

impl RR {
    pub fn from_roll(dice: D6, d: i8) -> Self {
        let difficulty = max(-3, min(3, d));
        let roll = dice.modify(-1 * difficulty);

        match roll {
            D6(1) => RR::FF(exploding_rolls(1, difficulty)),
            D6(2) => RR::FF(1),
            D6(5) => RR::SS(1),
            D6(6) => RR::SS(exploding_rolls(6, difficulty)),
            _ => RR::SF,
        }
    }
}

#[test]
fn it_matches_rolls_correctly() {
    assert_eq!(RR::from_roll(D6::new(2), -1), RR::SF);
    assert_eq!(RR::from_roll(D6::new(2), -2), RR::SF);
    assert_eq!(RR::from_roll(D6::new(3), 0), RR::SF);
    assert_eq!(RR::from_roll(D6::new(4), 0), RR::SF);
    assert_eq!(RR::from_roll(D6::new(5), 1), RR::SF);
    assert_eq!(RR::from_roll(D6::new(5), 2), RR::SF);
}
