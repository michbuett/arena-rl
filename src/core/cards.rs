extern crate rand;

use rand::prelude::*;
use std::cmp::max;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Suite {
    Clubs,
    Spades,
    Hearts,
    Diamonds,
}

impl Suite {
    pub fn affinity(&self) -> SuiteAffinity {
        match self {
            Suite::Clubs | Suite::Hearts => SuiteAffinity::Strength,
            Suite::Spades | Suite::Diamonds => SuiteAffinity::Agilty,
        }
    }

    pub fn substantiality(&self) -> SuiteSubstantiality {
        match self {
            Suite::Clubs | Suite::Spades => SuiteSubstantiality::Physical,
            Suite::Hearts | Suite::Diamonds => SuiteSubstantiality::Mental,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SuiteAffinity {
    Strength,
    Agilty,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SuiteSubstantiality {
    Physical,
    Mental,
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

    pub fn value(&self, target_suite: Suite) -> u8 {
        if self.suite == target_suite {
            self.value
        } else if self.suite.affinity() == target_suite.affinity()
            || self.suite.substantiality() == target_suite.substantiality()
        {
            (self.value + 1) / 2
        } else {
            0
        }
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
    use Suite::*;
    vec![
        Card::new(10, Clubs),
        Card::new(9, Spades),
        Card::new(8, Hearts),
        Card::new(7, Diamonds),
    ]
}

#[test]
fn test_allow_deterministic_cards() {
    use Suite::*;

    // let mut deck = Deck::new(|| vec![Card::spades(1), Card::hearts(2)]);
    let mut deck = Deck::new(&fixed_deck);

    // draw all cards from deck
    // should be in the same order we put them in
    assert_eq!(deck.deal(), Card::new(10, Clubs));
    assert_eq!(deck.deal(), Card::new(9, Spades));
    assert_eq!(deck.deal(), Card::new(8, Hearts));
    assert_eq!(deck.deal(), Card::new(7, Diamonds));

    // draw again all cards
    // should still be the same order
    assert_eq!(deck.deal(), Card::new(10, Clubs));
    assert_eq!(deck.deal(), Card::new(9, Spades));
    assert_eq!(deck.deal(), Card::new(8, Hearts));
    assert_eq!(deck.deal(), Card::new(7, Diamonds));
}
#[derive(Debug)]
pub struct Challenge {
    pub advantage: i8,
    pub challenge_type: Suite,
    pub skill_val: u8,
    pub target_num: u8,
}

#[derive(Debug, Clone)]
pub struct ChallengeResult {
    pub draw: (Card, Vec<Card>),
    pub success_lvl: i8,
}

pub fn resolve_challenge(c: Challenge, deck: &mut Deck) -> ChallengeResult {
    let draw = draw(deck, c.advantage, c.challenge_type);
    let val = c.skill_val + &draw.0.value(c.challenge_type);
    let success_lvl = if val >= c.target_num {
        (val / c.target_num) as i8
    } else {
        -1 * (c.target_num / max(1, val)) as i8
    };

    ChallengeResult { draw, success_lvl }
}

#[test]
fn test_can_resolve_simple_challenge() {
    use Suite::*;

    let mut deck = Deck::new(&fixed_deck);
    let challenge = Challenge {
        advantage: 0,
        challenge_type: Suite::Spades,
        skill_val: 5,
        target_num: 10,
    };

    let result = resolve_challenge(challenge, &mut deck);

    assert_eq!(
        result.draw,
        (Card::new(10, Clubs), vec![Card::new(10, Clubs)])
    );
    assert_eq!(result.success_lvl, 1); // 5 (skill) + 5 (half value for 10oC) VS 10 (TN)

    let mut deck = Deck::new(&fixed_deck);
    let challenge = Challenge {
        advantage: 0,
        challenge_type: Suite::Diamonds,
        skill_val: 5,
        target_num: 10,
    };

    let result = resolve_challenge(challenge, &mut deck);

    assert_eq!(result.success_lvl, -2); // 5 (skill) + 0 (zero for 10oC) VS 10
}

fn draw(deck: &mut Deck, advantage: i8, s: Suite) -> (Card, Vec<Card>) {
    if advantage == 0 {
        let card = deck.deal();
        (card, vec![card])
    } else {
        let draw: Vec<Card> = (0..advantage.abs() + 1).map(|_| deck.deal()).collect();
        let sign = i8::signum(advantage) as i32;
        let card = draw
            .iter()
            .max_by_key(|c| (100 * c.value(s) as i32 + c.value as i32) * sign)
            .unwrap()
            .to_owned();

        (card, draw)
    }
}

#[test]
fn test_draw_without_advantage() {
    use Suite::*;

    let mut deck = Deck::new(&fixed_deck);
    let (c, d) = draw(&mut deck, 0, Suite::Spades);
    assert_eq!(c, Card::new(10, Clubs));
    assert_eq!(d, vec![Card::new(10, Clubs)]);
}

#[test]
fn test_draw_with_advantage() {
    use Suite::*;

    let mut deck = Deck::new(&fixed_deck);
    let (c, d) = draw(&mut deck, 1, Suite::Spades);
    assert_eq!(c, Card::new(9, Spades));
    assert_eq!(d, vec![Card::new(10, Clubs), Card::new(9, Spades)]);

    let mut deck = Deck::new(&fixed_deck);
    let (c, d) = draw(&mut deck, 2, Suite::Diamonds);
    assert_eq!(c, Card::new(9, Spades));
    assert_eq!(
        d,
        vec![
            Card::new(10, Clubs),
            Card::new(9, Spades),
            Card::new(8, Hearts)
        ]
    );

    let mut deck = Deck::new(&fixed_deck);
    let (c, d) = draw(&mut deck, 2, Suite::Hearts);
    assert_eq!(c, Card::new(8, Hearts));
    assert_eq!(
        d,
        vec![
            Card::new(10, Clubs),
            Card::new(9, Spades),
            Card::new(8, Hearts)
        ]
    );
}

#[test]
fn test_draw_with_disadvantage() {
    use Suite::*;

    let mut deck = Deck::new(&fixed_deck);
    let (c, d) = draw(&mut deck, -1, Suite::Spades);
    assert_eq!(c, Card::new(10, Clubs));
    assert_eq!(d, vec![Card::new(10, Clubs), Card::new(9, Spades)]);

    let mut deck = Deck::new(&fixed_deck);
    let (c, d) = draw(&mut deck, -2, Suite::Diamonds);
    assert_eq!(c, Card::new(10, Clubs));
    assert_eq!(
        d,
        vec![
            Card::new(10, Clubs),
            Card::new(9, Spades),
            Card::new(8, Hearts)
        ]
    );

    let mut deck = Deck::new(&fixed_deck);
    let (c, d) = draw(&mut deck, -2, Suite::Clubs);
    assert_eq!(c, Card::new(8, Hearts));
    assert_eq!(
        d,
        vec![
            Card::new(10, Clubs),
            Card::new(9, Spades),
            Card::new(8, Hearts)
        ]
    );
}
