extern crate rand;

use rand::prelude::*;
use serde::Deserialize;
use std::cmp::max;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize)]
pub enum Suite {
    PhysicalStr,
    PhysicalAg,
    MentalStr,
    MentalAg,
    Physical,
    Mental,
    Strength,
    Agility,
    Any,
}

impl Suite {
    fn matches(&self, other: Suite) -> SuiteMatch {
        let one_way_match = self.match_one_way(other);
        if let SuiteMatch::No = one_way_match {
            other.match_one_way(*self)
        } else {
            one_way_match
        }
    }

    fn match_one_way(&self, other: Suite) -> SuiteMatch {
        use Suite::*;
        match (self, other) {
            (PhysicalStr, PhysicalStr)
            | (PhysicalAg, PhysicalAg)
            | (MentalStr, MentalStr)
            | (MentalAg, MentalAg)
            | (PhysicalStr, Physical)
            | (PhysicalAg, Physical)
            | (MentalStr, Mental)
            | (MentalAg, Mental)
            | (PhysicalStr, Strength)
            | (PhysicalAg, Agility)
            | (MentalStr, Strength)
            | (MentalAg, Agility)
            | (_, Any) => SuiteMatch::Full,

            (PhysicalStr, PhysicalAg)
            | (PhysicalStr, MentalStr)
            | (MentalStr, MentalAg)
            | (PhysicalAg, MentalAg) => SuiteMatch::Partial,

            _ => SuiteMatch::No,
        }
    }
}

pub enum SuiteMatch {
    Full,
    Partial,
    No,
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
        match self.suite.matches(target_suite) {
            SuiteMatch::Full => self.value,
            SuiteMatch::Partial => (self.value + 1) / 2,
            SuiteMatch::No => 0,
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
        let suites = vec![
            Suite::PhysicalStr,
            Suite::PhysicalAg,
            Suite::MentalStr,
            Suite::MentalAg,
        ];
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
        Card::new(10, PhysicalStr),
        Card::new(9, PhysicalAg),
        Card::new(8, MentalStr),
        Card::new(7, MentalAg),
    ]
}

#[test]
fn test_allow_deterministic_cards() {
    use Suite::*;

    // let mut deck = Deck::new(|| vec![Card::spades(1), Card::hearts(2)]);
    let mut deck = Deck::new(&fixed_deck);

    // draw all cards from deck
    // should be in the same order we put them in
    assert_eq!(deck.deal(), Card::new(10, PhysicalStr));
    assert_eq!(deck.deal(), Card::new(9, PhysicalAg));
    assert_eq!(deck.deal(), Card::new(8, MentalStr));
    assert_eq!(deck.deal(), Card::new(7, MentalAg));

    // draw again all cards
    // should still be the same order
    assert_eq!(deck.deal(), Card::new(10, PhysicalStr));
    assert_eq!(deck.deal(), Card::new(9, PhysicalAg));
    assert_eq!(deck.deal(), Card::new(8, MentalStr));
    assert_eq!(deck.deal(), Card::new(7, MentalAg));
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
        (val / max(1, c.target_num)) as i8
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
        challenge_type: Suite::PhysicalAg,
        skill_val: 5,
        target_num: 10,
    };

    let result = resolve_challenge(challenge, &mut deck);

    assert_eq!(
        result.draw,
        (Card::new(10, PhysicalStr), vec![Card::new(10, PhysicalStr)])
    );
    assert_eq!(result.success_lvl, 1); // 5 (skill) + 5 (half value for 10oC) VS 10 (TN)

    let mut deck = Deck::new(&fixed_deck);
    let challenge = Challenge {
        advantage: 0,
        challenge_type: Suite::MentalAg,
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
    let (c, d) = draw(&mut deck, 0, Suite::PhysicalAg);
    assert_eq!(c, Card::new(10, PhysicalStr));
    assert_eq!(d, vec![Card::new(10, PhysicalStr)]);
}

#[test]
fn test_draw_with_advantage() {
    use Suite::*;

    let mut deck = Deck::new(&fixed_deck);
    let (c, d) = draw(&mut deck, 1, Suite::PhysicalAg);
    assert_eq!(c, Card::new(9, PhysicalAg));
    assert_eq!(
        d,
        vec![Card::new(10, PhysicalStr), Card::new(9, PhysicalAg)]
    );

    let mut deck = Deck::new(&fixed_deck);
    let (c, d) = draw(&mut deck, 2, Suite::MentalAg);
    assert_eq!(c, Card::new(9, PhysicalAg));
    assert_eq!(
        d,
        vec![
            Card::new(10, PhysicalStr),
            Card::new(9, PhysicalAg),
            Card::new(8, MentalStr)
        ]
    );

    let mut deck = Deck::new(&fixed_deck);
    let (c, d) = draw(&mut deck, 2, Suite::MentalStr);
    assert_eq!(c, Card::new(8, MentalStr));
    assert_eq!(
        d,
        vec![
            Card::new(10, PhysicalStr),
            Card::new(9, PhysicalAg),
            Card::new(8, MentalStr)
        ]
    );
}

#[test]
fn test_draw_with_disadvantage() {
    use Suite::*;

    let mut deck = Deck::new(&fixed_deck);
    let (c, d) = draw(&mut deck, -1, Suite::PhysicalAg);
    assert_eq!(c, Card::new(10, PhysicalStr));
    assert_eq!(
        d,
        vec![Card::new(10, PhysicalStr), Card::new(9, PhysicalAg)]
    );

    let mut deck = Deck::new(&fixed_deck);
    let (c, d) = draw(&mut deck, -2, Suite::MentalAg);
    assert_eq!(c, Card::new(10, PhysicalStr));
    assert_eq!(
        d,
        vec![
            Card::new(10, PhysicalStr),
            Card::new(9, PhysicalAg),
            Card::new(8, MentalStr)
        ]
    );

    let mut deck = Deck::new(&fixed_deck);
    let (c, d) = draw(&mut deck, -2, Suite::PhysicalStr);
    assert_eq!(c, Card::new(8, MentalStr));
    assert_eq!(
        d,
        vec![
            Card::new(10, PhysicalStr),
            Card::new(9, PhysicalAg),
            Card::new(8, MentalStr)
        ]
    );
}
