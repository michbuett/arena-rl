extern crate rand;

use rand::prelude::*;

#[derive(Debug, Copy, Clone, Eq, PartialEq, PartialOrd)]
pub struct Dice(u8);

impl Dice {
    pub fn new(x: u8) -> Self {
        assert!(1 <= x && x <= 6);
        Dice(x)
    }
}

#[derive(Debug, Clone)]
pub struct Roll {
    pub dices: Vec<Dice>,
    pub threshold: Dice,
    pub successes: u8,
}

impl Roll {
    pub fn new(dice: u8, threshold: Dice) -> Self {
        let range = rand::distributions::Uniform::from(1..=6);
        let mut rng = rand::thread_rng();
        let mut dices: Vec<Dice> = Vec::new();
        let mut successes = 0;

        for _ in 1..=dice {
            let d = Dice::new(rng.sample(range));

            if d >= threshold {
                successes += 1;
            }

            dices.push(d);
        }

        Self { dices, threshold, successes }
    }

    pub fn total(&self) -> u32 {
        self.dices.iter().map(|Dice(d)| *d as u32).sum()
    }
}
