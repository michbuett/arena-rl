extern crate rand;

use rand::prelude::*;

#[derive(Debug, Copy, Clone, Eq, PartialEq, PartialOrd, Ord)]
pub struct D6(pub u8);

impl D6 {
    pub fn roll() -> Self {
        let range = rand::distributions::Uniform::from(1..=6);
        let mut rng = rand::thread_rng();

        Self(rng.sample(range))
    }
}

#[derive(Debug, Clone)]
pub struct Roll {
    // dice: Vec<D6>,
    pub num_successes: u8,
    pub num_fails: u8,
}

impl Roll {
    pub fn new(advantage: i8, threshold: u8) -> Self {
        let num_dice = 3 + i8::abs(advantage);
        let mut dice: Vec<D6> = (1..=num_dice).map(|_| D6::roll()).collect();

        if advantage != 0 {
            if advantage > 0 {
                dice.sort_by(|a, b| b.cmp(a));
            } else {
                dice.sort();
            }

            dice = dice.drain(0..3).collect();
        }

        Self::from_dice(dice, threshold)
    }

    pub fn from_dice(dice: Vec<D6>, threshold: u8) -> Self {
        debug_assert_eq!(dice.len(), 3);
            
        let mut num_successes = 0;
        let mut num_fails = 0;

        for d in dice.iter() {
            if d.0 >= threshold {
                num_successes += 1;
            } else {
                num_fails += 1;
            }
        }

        Self {
            // dice,
            num_successes,
            num_fails,
        }
    }
}
