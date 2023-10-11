extern crate rand;

use rand::prelude::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Suite {
    Clubs,
    Spades,
    Hearts,
    Diamonds,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Card {
    pub value: u8,
    pub suite: Suite,
}

impl Card {
    /// Creates a card with a given value and a given suite (for tests)
    #[allow(dead_code)]
    pub fn new(value: u8, suite: Suite) -> Self {
        Self { value, suite }
    }
}

#[derive(Clone)]
pub struct Deck {
    cards: Vec<Card>,
    shuffle: &'static (dyn Fn() -> Vec<Card> + Send + Sync),
}

use std::fmt;
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

#[allow(dead_code)]
fn fixed_deck() -> Vec<Card> {
    vec![Card::new(1, Suite::Spades), Card::new(2, Suite::Hearts)]
}

#[test]
fn test_allow_deterministic_cards() {
    use Suite::*;

    // let mut deck = Deck::new(|| vec![Card::spades(1), Card::hearts(2)]);
    let mut deck = Deck::new(&fixed_deck);

    // draw all cards from deck
    // should be in the same order we put them in
    assert_eq!(deck.deal(), Card::new(1, Spades));
    assert_eq!(deck.deal(), Card::new(2, Hearts));

    // draw again all cards
    // should still be the same order
    assert_eq!(deck.deal(), Card::new(1, Spades));
    assert_eq!(deck.deal(), Card::new(2, Hearts));
}
