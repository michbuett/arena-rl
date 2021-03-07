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
    // let r_idx = (dice.0 - 1) as usize;
    // let d_idx = max(0, min(SUCCESS_TABLE.len() as i8, d + 3)) as usize;
    // SUCCESS_TABLE[d_idx][r_idx]
    let roll = dice.0 as i8 - max(-3, min(3, d));
    if roll <= 1 {
        RR::FF
    } else if roll >= 6 {
        RR::SS
    } else {
        match roll {
            2 => RR::F_,
            5 => RR::S_,
            _ => RR::SF,
        }
    }
}

// const SUCCESS_TABLE: [[RR; 6]; 7] = [
//     [RR::FF, RR::FF, RR::FF, RR::F_, RR::F_, RR::SF], // -3 -> impossible
//     [RR::FF, RR::FF, RR::F_, RR::F_, RR::SF, RR::S_], // -2 -> very hard
//     [RR::FF, RR::F_, RR::SF, RR::SF, RR::S_, RR::S_], // -1 -> hard
//     [RR::FF, RR::F_, RR::SF, RR::SF, RR::S_, RR::SS], //  0 -> mediocre
//     [RR::F_, RR::SF, RR::SF, RR::S_, RR::S_, RR::SS], //  1 -> easy
//     [RR::F_, RR::SF, RR::S_, RR::S_, RR::SS, RR::SS], //  2 -> very easy
//     [RR::SF, RR::S_, RR::S_, RR::SS, RR::SS, RR::SS], //  3 -> trivial
// ];

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
    /// a fatal failure
    FF, 
    /// simple failure
    F_, 
    /// a success, but ...
    SF, 
    /// a simple success - no strings attached
    S_, 
    /// a super success
    SS, 
}

impl RR {
    pub fn from_roll(roll: D6, d: i8) -> Self {
        let result = roll_to_result(roll, d);

        match result {
            RR::FF => 
                if let RR::FF = roll_to_result(D6::roll(), d) { RR::FF } else { RR::F_ },
           
            RR::SS => 
                if let RR::SS = roll_to_result(D6::roll(), d) { RR::SS } else { RR::S_ },
            
            _ => result
        }
    }
}

#[test]
fn it_matches_rolls_correctly() {
    assert_eq!(RR::from_roll(D6::new(2), -1), RR::SF);
    assert_eq!(RR::from_roll(D6::new(2), -2), RR::SF);
    assert_eq!(RR::from_roll(D6::new(2), -3), RR::S_);
    assert_eq!(RR::from_roll(D6::new(2), -4), RR::S_);

    assert_eq!(RR::from_roll(D6::new(2), 0), RR::F_);
    assert_eq!(RR::from_roll(D6::new(3), 0), RR::SF);
    assert_eq!(RR::from_roll(D6::new(4), 0), RR::SF);
    assert_eq!(RR::from_roll(D6::new(5), 0), RR::S_);

    assert_eq!(RR::from_roll(D6::new(5), 1), RR::SF);
    assert_eq!(RR::from_roll(D6::new(5), 2), RR::SF);
    assert_eq!(RR::from_roll(D6::new(5), 3), RR::F_);
    assert_eq!(RR::from_roll(D6::new(5), 4), RR::F_);
}


// pub struct Roll {
//     dices: Vec<D6>,
//     difficulty: i8,
// }

// impl Roll {
//     pub fn new(dices: u8, difficulty: i8) -> Self {
//         Self {
//             dices: (1..=dices).map(|_| D6::roll()).collect::<Vec<_>>(),
//             difficulty,
//         }
//     }
// }
