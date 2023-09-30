extern crate rand;

use rand::prelude::*;

#[derive(Clone, Copy, Debug)]
pub enum Suite {
    Clubs,
    Spades,
    Hearts,
    Diamonds,
}

#[derive(Clone, Copy, Debug)]
pub struct Card {
    pub value: u8,
    pub suite: Suite,
}

pub trait Deck {
    fn deal(&mut self) -> Card;
}

#[derive(Debug, Clone)]
pub struct FixDeck {
    current_idx: usize,
    cards: Vec<Card>,
}

impl FixDeck {
    pub fn new(cards: Vec<Card>) -> Self {
        assert!(!cards.is_empty(), "Initial set of cards must not be empty!");

        Self {
            current_idx: 0,
            cards,
        }
    }
}

impl Deck for FixDeck {
    fn deal(&mut self) -> Card {
        let result = self.cards[self.current_idx].clone();
        if self.current_idx < self.cards.len() {
            self.current_idx += 1;
        } else {
            self.current_idx = 0;
        }
        result
    }
}

/// A randomly generated standard poker deck with 52 cards
#[derive(Debug, Clone)]
pub struct RndDeck {
    cards: Vec<Card>,
}

impl RndDeck {
    pub fn new() -> Self {
        Self {
            cards: Self::shuffel(),
        }
    }

    fn shuffel() -> Vec<Card> {
        let suites = vec![Suite::Clubs, Suite::Spades, Suite::Hearts, Suite::Diamonds];
        let mut cards = Vec::new();

        for suite in suites {
            for value in 1..=13 {
                cards.push(Card { value, suite })
            }
        }

        let mut rng = thread_rng();
        cards.shuffle(&mut rng);
        cards
    }
}

impl Deck for RndDeck {
    fn deal(&mut self) -> Card {
        if self.cards.is_empty() {
            self.cards = Self::shuffel();
        }

        self.cards.pop().unwrap() // unwrapping is safe because the deck is shuffelled when empty
    }
}
