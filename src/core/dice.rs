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
    dice: Vec<D6>,
    pub num_successes: u8,
    pub num_fails: u8,
}

impl Roll {
    pub fn new(num_dice: u8, threshold: u8) -> Self {
        Self::from_dice((1..=num_dice).map(|_| D6::roll()).collect(), threshold)
    }

    pub fn from_dice(dice: Vec<D6>, threshold: u8) -> Self {
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
            dice,
            num_successes,
            num_fails,
        }
    }
}
