extern crate rand;

use rand::prelude::*;
use std::cmp::{min, max};

#[derive(Debug, Copy, Clone, Eq, PartialEq, PartialOrd, Ord)]
pub struct D6(pub u8);

impl D6 {
    // pub fn new(x: u8) -> Self {
    //     assert!(1 <= x && x <= 6);
    //     Self(x)
    // }

    pub fn roll() -> Self {
        let range = rand::distributions::Uniform::from(1..=6);
        let mut rng = rand::thread_rng();

        Self(rng.sample(range))
    }

    pub fn result(self, d: i8) -> RR {
        let r_idx = (self.0 - 1) as usize;
        let d_idx = max(0, min(SUCCESS_TABLE.len() as i8, d + 3)) as usize;
        let result = SUCCESS_TABLE[d_idx][r_idx];
        // println!("Roll {}, difficulty: {}, result: {:?}", self.0, difficulty, result);
        result
    }
}

// #[derive(Debug, Copy, Clone, Eq, PartialEq)]
// pub enum Difficulty {
//     VeryEasy,
//     Easy,
//     Mediocre,
//     Hard,
//     VeryHard,
// }

// impl Difficulty {
//     pub fn from_skills(skill: i8, challenge: i8) -> Difficulty {
//         if skill == challenge {
//             Difficulty::Mediocre
//         } else if skill > challenge {
//             if skill >= 2 * challenge {
//                 Difficulty::VeryEasy
//             } else {
//                 Difficulty::Easy
//             }
//         } else {
//             if challenge >= 2 * skill {
//                 Difficulty::VeryHard
//             } else {
//                 Difficulty::Hard
//             }
//         }
//     }

//     fn to_index(self) -> usize {
//         match self {
//             Difficulty::VeryEasy => 0,
//             Difficulty::Easy => 1,
//             Difficulty::Mediocre => 2,
//             Difficulty::Hard => 3,
//             Difficulty::VeryHard => 4,
//         }
//     }
// }

const SUCCESS_TABLE: [[RR; 6]; 7] = [
    [RR::SF, RR::S_, RR::S_, RR::SS, RR::SS, RR::SS], // very easy
    [RR::F_, RR::SF, RR::S_, RR::S_, RR::SS, RR::SS], // very easy
    [RR::F_, RR::SF, RR::SF, RR::S_, RR::S_, RR::SS], // easy
    [RR::FF, RR::F_, RR::SF, RR::SF, RR::S_, RR::SS], // mediocre
    [RR::FF, RR::F_, RR::SF, RR::SF, RR::S_, RR::S_], // hard
    [RR::FF, RR::FF, RR::F_, RR::F_, RR::SF, RR::S_], // very hard
    [RR::FF, RR::FF, RR::FF, RR::F_, RR::F_, RR::SF], // impossible
];

// const SUCCESS_TABLE: [[RR; 6]; 5] = [
//     [RR::SF, RR::S_, RR::S_, RR::S_, RR::S_, RR::S_], // very easy
//     [RR::SF, RR::SF, RR::S_, RR::S_, RR::S_, RR::S_], // easy
//     [RR::F_, RR::SF, RR::SF, RR::S_, RR::S_, RR::S_], // mediocre
//     [RR::F_, RR::F_, RR::SF, RR::SF, RR::S_, RR::S_], // hard
//     [RR::F_, RR::F_, RR::F_, RR::SF, RR::SF, RR::S_], // very hard
// ];

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum RR {
    FF,
    F_,
    SF,
    S_,
    SS,
}
