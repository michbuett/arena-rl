extern crate rand;

use rand::prelude::*;
use std::cmp::{min, max};

#[derive(Debug, Copy, Clone, Eq, PartialEq, PartialOrd, Ord)]
pub struct D6(pub u8);

impl D6 {
    pub fn roll() -> Self {
        let range = rand::distributions::Uniform::from(1..=6);
        let mut rng = rand::thread_rng();

        Self(rng.sample(range))
    }

    pub fn result(self, d: i8) -> RR {
        let result = roll_to_result(self, d);

        match result {
            RR::FF => 
                if let RR::FF = roll_to_result(D6::roll(), d) { RR::FF } else { RR::F_ },
           
            RR::SS => 
                if let RR::SS = roll_to_result(D6::roll(), d) { RR::SS } else { RR::S_ },
            
            _ => result
        }
    }
}

fn roll_to_result(dice: D6, d: i8) -> RR {
    let r_idx = (dice.0 - 1) as usize;
    let d_idx = max(0, min(SUCCESS_TABLE.len() as i8, d + 3)) as usize;
    SUCCESS_TABLE[d_idx][r_idx]
}

const SUCCESS_TABLE: [[RR; 6]; 7] = [
    [RR::FF, RR::FF, RR::FF, RR::F_, RR::F_, RR::SF], // -3 -> impossible
    [RR::FF, RR::FF, RR::F_, RR::F_, RR::SF, RR::S_], // -2 -> very hard
    [RR::FF, RR::F_, RR::SF, RR::SF, RR::S_, RR::S_], // -1 -> hard
    [RR::FF, RR::F_, RR::SF, RR::SF, RR::S_, RR::SS], //  0 -> mediocre
    [RR::F_, RR::SF, RR::SF, RR::S_, RR::S_, RR::SS], //  1 -> easy
    [RR::F_, RR::SF, RR::S_, RR::S_, RR::SS, RR::SS], //  2 -> very easy
    [RR::SF, RR::S_, RR::S_, RR::SS, RR::SS, RR::SS], //  3 -> trivial
];

// const SUCCESS_TABLE: [[RR; 6]; 5] = [
//     [RR::SF, RR::S_, RR::S_, RR::S_, RR::S_, RR::S_], // very easy
//     [RR::SF, RR::SF, RR::S_, RR::S_, RR::S_, RR::S_], // easy
//     [RR::F_, RR::SF, RR::SF, RR::S_, RR::S_, RR::S_], // mediocre
//     [RR::F_, RR::F_, RR::SF, RR::SF, RR::S_, RR::S_], // hard
//     [RR::F_, RR::F_, RR::F_, RR::SF, RR::SF, RR::S_], // very hard
// ];

/// A roll result
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum RR {
    /// a critical failure
    FF, 
    /// just a failure
    F_,
    SF,
    S_,
    SS,
}
