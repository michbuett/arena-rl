extern crate rand;

use rand::prelude::*;
use std::cmp::{max, min};

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

#[derive(Debug, Clone)]
pub struct Roll {
    dice: Vec<D6>,
    advantage: RollAdvantage,
    extra: Option<D6>,
}

impl Roll {
    pub fn new(num_dice: u8, advantage: RollAdvantage) -> Self {
        let dice: Vec<D6> = (1..=num_dice)
            .map(|_| D6::roll().modify(advantage.0))
            .collect();
        let extra = if dice.contains(&D6::new(6)) {
            Some(D6::roll().modify(advantage.0))
        } else {
            None
        };

        Self {
            dice,
            advantage,
            extra,
        }
    }

    pub fn successes(&self) -> u8 {
        let extra = match self.extra {
            Some(D6(d)) if d >= 4 => 1,
            _ => 0
        };
                
        extra + self.dice
            .iter()
            .filter(|die| die.0 >= 4)
            .count() as u8
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RollAdvantage(i8);

impl RollAdvantage {
    pub fn new(skill: i8, difficulty: i8) -> Self {
        Self(i8::signum(skill - difficulty))
    }
}
