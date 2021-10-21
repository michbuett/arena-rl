extern crate rand;

use rand::prelude::*;
use std::cmp::max;

#[derive(Debug, Copy, Clone, Eq, PartialEq, PartialOrd, Ord)]
pub struct D6(pub u8);

impl D6 {
    // pub fn new(raw: i8) -> Self {
    //     Self(max(1, min(6, raw)) as u8)
    // }

    pub fn roll() -> Self {
        let range = rand::distributions::Uniform::from(1..=6);
        let mut rng = rand::thread_rng();

        Self(rng.sample(range))
    }

    // pub fn modify(self, modifier: i8) -> Self {
    //     Self::new(self.0 as i8 + modifier)
    // }
}

#[derive(Debug, Clone)]
pub struct Roll {
    dice: Vec<D6>,
    modifier: i8,
    result: u8,

    // extra: Option<D6>,
    // normal_successes: u8,
    // extra_successes: u8,
    // fails: u8,
}

impl Roll {
    // pub fn new(num_dice: u8, d: Difficulty) -> Self {
    pub fn new(num_dice: u8, modifier: i8) -> Self {
        let dice: Vec<D6> = (1..=num_dice).map(|_| D6::roll()).collect();
        Self::from_dice(dice, modifier)
    }

    pub fn from_dice(dice: Vec<D6>, modifier: i8) -> Self {
        let m = dice.len() as i8 * modifier;
        let roll_sum: u8 = dice.iter().map(|d| d.0).sum();
        let result = max(0, roll_sum as i8 + m) as u8;


        Self {
            dice,
            result,
            modifier,
        }
    }
    // pub fn normal_successes(&self) -> u8 {
    //     self.normal_successes
    // }

    pub fn result(&self) -> u8 {
        self.result
    }

    // pub fn successes(&self) -> u8 {
    //     self.normal_successes + self.extra_successes
    // }

    // pub fn fails(&self) -> u8 {
    //     self.fails
    // }
}

// #[derive(Debug, Clone, Copy)]
// pub struct Difficulty(u8);

// impl Difficulty {
//     pub fn new(d: u8) -> Self {
//         // Self(i8::signum(skill - difficulty))
//         Self(max(1, d))
//     }
// }

// #[derive(Debug, Clone, Copy)]
// pub struct RollAdvantage(i8);

// impl RollAdvantage {
//     pub fn new(skill: i8, difficulty: i8) -> Self {
//         // Self(i8::signum(skill - difficulty))
//         Self(min(3, max(-3, skill - difficulty)))
//     }
// }
