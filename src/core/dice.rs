extern crate rand;

use rand::prelude::*;

#[derive(Debug, Copy, Clone, Eq, PartialEq, PartialOrd, Ord)]
pub struct D6(pub u8);

impl D6 {
    pub fn new(x: u8) -> Self {
        assert!(1 <= x && x <= 6);
        Self(x)
    }

    pub fn roll() -> Self {
        let range = rand::distributions::Uniform::from(1..=6);
        let mut rng = rand::thread_rng();

        Self(rng.sample(range))
    }

    pub fn result(self, difficulty: i8) -> RR {
        use std::cmp::{min, max};
        let r_idx = (self.0 - 1) as usize;
        let d_idx = (2 + max(-2, min(3, difficulty))) as usize;
        let result = SUCCESS_TABLE[d_idx][r_idx];

        // println!("Roll {}, difficulty: {}, result: {:?}", self.0, difficulty, result);

        result
    }
}

const SUCCESS_TABLE: [[RR; 6]; 6] = [
    [RR::SuccessBut, RR::Success, RR::Success, RR::CritSuccess, RR::CritSuccess, RR::CritSuccess], // d(-2)
    [RR::SuccessBut, RR::SuccessBut, RR::Success, RR::Success, RR::CritSuccess, RR::CritSuccess], // d(-1)
    [RR::Fail, RR::SuccessBut, RR::SuccessBut, RR::Success, RR::Success, RR::CritSuccess], // d(0)
    [RR::Fail, RR::Fail, RR::SuccessBut, RR::SuccessBut, RR::Success, RR::Success], //d(+1)
    [RR::CritFail, RR::Fail, RR::Fail, RR::SuccessBut, RR::SuccessBut, RR::Success], //d(+2)
    [RR::CritFail, RR::CritFail, RR::Fail, RR::Fail, RR::SuccessBut, RR::SuccessBut], //d(+3)
];

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum RR {
    CritFail,
    Fail,
    SuccessBut,
    Success,
    CritSuccess,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, PartialOrd, Ord)]
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
