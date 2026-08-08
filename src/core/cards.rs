extern crate rand;

use bevy::ecs::component::Component;
use rand::prelude::*;
use serde::Deserialize;
use std::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize)]
pub enum Suite {
    Clubs,
    Spades,
    Hearts,
    Diamonds,
}

// impl Suite {
//     pub fn matches(&self, attr: &AttributeType) -> bool {
//         use AttributeType::*;
//         use Suite::*;

//         match attr {
//             PhysicalStr => matches!(self, Clubs),
//             PhysicalAg => matches!(self, Spades),
//             MentalStr => matches!(self, Hearts),
//             MentalAg => matches!(self, Diamonds),
//             Physical => matches!(self, Clubs) || matches!(self, Spades),
//             Mental => matches!(self, Hearts) || matches!(self, Diamonds),
//             Strength => matches!(self, Clubs) || matches!(self, Hearts),
//             Agility => matches!(self, Spades) || matches!(self, Diamonds),
//             Any => true,
//         }
//     }
// }

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CardValue {
    Ace,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Card {
    value: CardValue,
    suite: Suite,
}

impl Card {
    /// Creates a card with a given value and a given suite (for tests)
    /// Panics if value > 10 or value < 1
    fn new(value: u8, suite: Suite) -> Self {
        let value = match value {
            1 => CardValue::Ace,
            2 => CardValue::Two,
            3 => CardValue::Three,
            4 => CardValue::Four,
            5 => CardValue::Five,
            6 => CardValue::Six,
            7 => CardValue::Seven,
            8 => CardValue::Eight,
            9 => CardValue::Nine,
            10 => CardValue::Ten,
            v => panic!("Invalid card value {v}"),
        };

        Self { value, suite }
    }

    pub fn value(&self) -> u8 {
        self.value as u8 + 1
    }

    pub fn suite(&self) -> Suite {
        self.suite
    }
}

const NUM_DECKS_PER_GAME: u8 = 2;

#[derive(Clone)]
pub struct Deck {
    cards: Vec<Card>,
    shuffle: &'static (dyn Fn() -> Vec<Card> + Send + Sync),
}

impl fmt::Debug for Deck {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Deck").field("cards", &self.cards).finish()
    }
}

impl Deck {
    pub fn new<F>(shuffle: &'static F) -> Self
    where
        F: Fn() -> Vec<Card> + Sync + Send,
    {
        Self {
            cards: vec![],
            shuffle,
        }
    }

    pub fn new_rnd() -> Self {
        Self::new(&Self::rnd_shuffle)
    }

    pub fn deal(&mut self) -> Card {
        if self.cards.is_empty() {
            let mut cards: Vec<Card> = (*(self.shuffle)()).to_vec();
            cards.reverse();
            self.cards = cards;
        }

        self.cards.pop().unwrap() // unwrapping is safe because the deck is shuffelled when empty
    }

    fn rnd_shuffle() -> Vec<Card> {
        let suites = [Suite::Clubs, Suite::Spades, Suite::Hearts, Suite::Diamonds];
        let mut cards = Vec::new();

        for _ in 0..NUM_DECKS_PER_GAME {
            for suite in suites.iter() {
                for value in 1..=10 {
                    cards.push(Card::new(value, *suite))
                }
            }
        }

        let mut rng = thread_rng();
        cards.shuffle(&mut rng);
        cards
    }
}

// #[cfg(test)]
// pub fn fixed_deck() -> Vec<Card> {
//     use Suite::*;
//     vec![
//         Card::new(10, Clubs),
//         Card::new(9, Spades),
//         Card::new(8, Hearts),
//         Card::new(7, Diamonds),
//         Card::new(10, Spades),
//         Card::new(9, Hearts),
//         Card::new(8, Diamonds),
//         Card::new(7, Clubs),
//     ]
// }

const MAX_HAND_SIZE: usize = 5;

#[derive(Component)]
pub struct Hand {
    selected_idx: Option<usize>,
    cards: Vec<Card>,
}

impl Hand {
    pub fn new() -> Self {
        let mut deck = Deck::new_rnd();
        let cards = (0..3).map(|_| deck.deal()).collect();

        Self {
            cards,
            selected_idx: None,
        }
    }

    pub fn cards(&self) -> impl Iterator<Item = &Card> {
        self.cards.iter()
    }

    pub fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }

    pub fn is_full(&self) -> bool {
        self.cards.len() >= MAX_HAND_SIZE
    }

    pub fn add_card(&mut self, card: Card) {
        self.cards.push(card);
    }

    pub fn remove(&mut self, index: usize) -> Card {
        if self.selected_idx.is_some_and(|i| i == index) {
            self.selected_idx = None;
        }
        self.cards.remove(index)
    }

    pub fn toggle_selection(&mut self, index: usize) {
        assert!(
            index < self.cards.len(),
            "The hand has not enough cards (index: {index})"
        );

        if self.is_selected(index) {
            self.reset_selection();
        } else {
            self.selected_idx = Some(index);
        }
    }

    pub fn reset_selection(&mut self) {
        self.selected_idx = None;
    }

    pub fn is_selected(&self, index: usize) -> bool {
        self.selected_idx.is_some_and(|selected| selected == index)
    }
}
