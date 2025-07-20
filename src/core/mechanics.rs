use super::{AttributeType, Attributes, Card, Deck};

pub const MAX_HAND_SIZE: usize = 5;

#[derive(Clone, Debug)]
pub struct SkillCheck {
    pub target_number: u8,
    pub attribute: AttributeType,
}

impl SkillCheck {
    pub fn perform_check(self, deck: &mut Deck, attr_values: &Attributes) -> SkillCheckResult {
        let av = attr_values.get(self.attribute);
        let flip = deck.deal();
        let result = (flip.value_low() as i8 + av).try_into().unwrap_or(0);

        SkillCheckResult {
            // flip,
            result,
            // attr_values: *attr_values,
            check: self,
        }
    }
}

#[derive(Clone, Debug)]
pub struct SkillCheckResult {
    check: SkillCheck,
    // attr_values: AttributeValues,
    result: u8,
    // flip: Card,
}

impl SkillCheckResult {
    pub fn is_success(&self) -> bool {
        self.result >= self.check.target_number
    }

    // pub fn magnitude(&self) -> u8 {
    //     (self.result as i16 - self.check.target_number as i16).unsigned_abs() as u8
    // }

    // pub fn result(&self) -> u8 {
    //     self.result
    // }
}

#[derive(Debug)]
pub struct ProgressCheckNew {
    pub num_cards: u8,
    pub resistence: u8,
}

impl ProgressCheckNew {
    pub fn perform_check(self, deck: &mut Deck) -> ProgressCheckResult {
        let resistance = self.resistence.clamp(0, self.num_cards);
        let mut draw = (1..=self.num_cards)
            .map(|_| deck.deal())
            .collect::<Vec<_>>();

        draw.sort_by(|a, b| b.value_high().cmp(&a.value_high()));

        let split = draw.split_at(resistance as usize);
        let (discard, draw) = (split.0.to_vec(), split.1.to_vec());

        ProgressCheckResult { discard, draw }
    }
}

#[derive(Debug)]
pub struct ProgressCheck {
    pub magnitude: u8,
}

impl ProgressCheck {
    pub fn base() -> Self {
        Self { magnitude: 3 }
    }

    pub fn modify_magnitude(mut self, delta: i8) -> Self {
        self.magnitude = (self.magnitude as i16 + delta as i16).clamp(1, 10) as u8;
        self
    }

    pub fn perform_check(self, deck: &mut Deck) -> ProgressCheckResult {
        let (base_amount, advantage) = match self.magnitude {
            0 => (0, 0), // draw nothing
            1 => (1, -2),
            2 => (1, -1),
            other @ _ => {
                let (div, rest) = (other / 3, other % 3);
                if rest == 2 {
                    (div + 1, -1)
                } else {
                    (div, rest as i8)
                }
            }
        };

        let draw_amount = base_amount + advantage.unsigned_abs();
        let mut draw = (1..=draw_amount).map(|_| deck.deal()).collect::<Vec<_>>();
        draw.sort_by(|a, b| a.value_low().cmp(&b.value_low())); // now cards are sorted in ascending order

        let (draw, discard) = if advantage < 0 {
            // draw with disadvantage
            // => keep the first (lowest), discard the last (highest)
            let split = draw.split_at(draw.len() - advantage.abs() as usize);
            (split.0.to_vec(), split.1.to_vec())
        } else if advantage > 0 {
            // draw with advantage
            // => keep the last (highes), discard the first (lowest)
            let split = draw.split_at(advantage.abs() as usize);
            (split.1.to_vec(), split.0.to_vec())
        } else {
            (draw, vec![])
        };

        ProgressCheckResult {
            discard,
            draw,
            // check: self,
        }
    }
}

#[derive(Debug)]
pub struct ProgressCheckResult {
    pub discard: Vec<Card>,
    pub draw: Vec<Card>,
    // pub check: ProgressCheck,
}

impl ProgressCheckResult {
    pub fn sum(&self) -> u8 {
        self.draw.iter().map(|c| c.value_low()).sum()
    }
}

#[test]
fn test_can_perform_a_basic_progress_check() {
    use crate::core::fixed_deck;

    let mut deck = Deck::new(&fixed_deck);
    let check = ProgressCheck { magnitude: 6 };
    let result = check.perform_check(&mut deck);

    assert_eq!(result.draw.len(), 2);
    assert_eq!(result.discard.len(), 0);
}

#[test]
fn test_can_perform_a_progress_check_with_advantage() {
    use crate::core::fixed_deck;

    let mut deck = Deck::new(&fixed_deck);
    let check = ProgressCheck { magnitude: 16 };
    let result = check.perform_check(&mut deck);

    assert_eq!(result.draw.len(), 5);
    assert_eq!(result.discard.len(), 1);

    // verify that the discarded card is indeed the lowest card
    let discarded_card = result.discard.first().unwrap();
    for card in result.draw.iter() {
        assert!(card.value_low() >= discarded_card.value_low());
    }
}
