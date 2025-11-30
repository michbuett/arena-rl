use serde::Deserialize;

use crate::core::Effect;

use super::{
    ActiveEffect, ActiveEffectSource, AttributeType, Attributes, Card, Deck, FeatKey, Suite,
};

#[derive(Debug, Clone)]
pub struct CheckResult {
    pub cards: Vec<Card>,
    pub success: Magnitude,
    pub complication: Magnitude,
    pub check: Option<(i8, AttributeType, CheckModifier)>,
}

impl CheckResult {
    pub fn no_check() -> Self {
        Self {
            cards: vec![],
            success: Magnitude::Normal,
            complication: Magnitude::None,
            check: None,
        }
    }
    pub fn is_success(&self) -> bool {
        !matches!(self.success, Magnitude::None)
    }

    pub fn has_complication(&self) -> bool {
        !matches!(self.complication, Magnitude::None)
    }

    // pub fn success_level(&self) -> u8 {
    //     match self.success {
    //         Magnitude::None => 0,
    //         Magnitude::Minor => 1,
    //         Magnitude::Normal => 2,
    //         Magnitude::Major => 3,
    //     }
    // }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Deserialize)]
pub enum Magnitude {
    None,
    Minor,
    Normal,
    Major,
}

#[derive(Debug, Clone)]
pub enum FlipModifierSource {
    Feat(FeatKey),
    Circumstance(String),
    Temporary(u8, String),
}

impl From<&ActiveEffectSource> for FlipModifierSource {
    fn from(value: &ActiveEffectSource) -> Self {
        match value {
            ActiveEffectSource::Feat(key) => FlipModifierSource::Feat(*key),
            ActiveEffectSource::Temporary(turns, descr) => {
                FlipModifierSource::Temporary(*turns, descr.clone())
            }
        }
    }
}

impl FlipModifierSource {
    fn situational(descr: impl Into<String>) -> Self {
        Self::Circumstance(descr.into())
    }
}

#[derive(Debug, Clone)]
pub struct CheckModifier(Vec<(i32, FlipModifierSource)>);

impl CheckModifier {
    pub fn new() -> Self {
        Self(vec![])
    }

    pub fn iter(&self) -> impl Iterator<Item = &(i32, FlipModifierSource)> {
        self.0.iter()
    }

    fn add(&mut self, boon_or_bane: impl Into<i32>, src: FlipModifierSource) {
        self.0.push((boon_or_bane.into(), src));
    }

    pub fn tn(&self, base_tn: impl Into<i32>) -> u8 {
        let chance_modifier: i32 = self.0.iter().map(|(d, _)| *d).sum();
        (base_tn.into() + chance_modifier).clamp(1, 9) as u8
    }

    fn test(&self, flip: Card, base_tn: impl Into<i32>) -> bool {
        flip.value() <= self.tn(base_tn)
    }
}

fn suite_match(attribute: AttributeType, card: &Card) -> bool {
    matches!(
        (attribute, card.suite()),
        (AttributeType::PhysicalStr, Suite::Clubs)
            | (AttributeType::PhysicalAg, Suite::Spades)
            | (AttributeType::MentalStr, Suite::Hearts)
            | (AttributeType::MentalAg, Suite::Diamonds)
    )
}

// #[derive(Debug)]
// pub struct ProgressCheck {
//     pub num_cards: u8,
//     pub resistence: u8,
// }

// impl ProgressCheck {
//     pub fn perform_check(self, deck: &mut Deck) -> ProgressCheckResult {
//         let resistance = self.resistence.clamp(0, self.num_cards);
//         let mut draw = (1..=self.num_cards)
//             .map(|_| deck.deal())
//             .collect::<Vec<_>>();

//         draw.sort_by(|a, b| b.value().cmp(&a.value()));

//         let split = draw.split_at(resistance as usize);
//         let (discard, draw) = (split.0.to_vec(), split.1.to_vec());

//         ProgressCheckResult { discard, draw }
//     }
// }

// #[derive(Debug)]
// pub struct ProgressCheckResult {
//     pub discard: Vec<Card>,
//     pub draw: Vec<Card>,
//     // pub check: ProgressCheck,
// }

// impl ProgressCheckResult {
//     pub fn sum(&self) -> u8 {
//         self.draw.iter().map(|c| c.value()).sum()
//     }
// }

// #[test]
// fn test_can_perform_a_basic_progress_check() {
//     use crate::core::fixed_deck;

//     let mut deck = Deck::new(&fixed_deck);
//     let check = ProgressCheck { magnitude: 6 };
//     let result = check.perform_check(&mut deck);

//     assert_eq!(result.draw.len(), 2);
//     assert_eq!(result.discard.len(), 0);
// }

// #[test]
// fn test_can_perform_a_progress_check_with_advantage() {
//     use crate::core::fixed_deck;

//     let mut deck = Deck::new(&fixed_deck);
//     let check = ProgressCheck { magnitude: 16 };
//     let result = check.perform_check(&mut deck);

//     assert_eq!(result.draw.len(), 5);
//     assert_eq!(result.discard.len(), 1);

//     // verify that the discarded card is indeed the lowest card
//     let discarded_card = result.discard.first().unwrap();
//     for card in result.draw.iter() {
//         assert!(card.value_low() >= discarded_card.value_low());
//     }
// }
//
//

// pub struct SimpleAction<'a> {
//     pub difficulty_modifier: i8,
//     pub risk_complication: bool,
//     pub attribute: AttributeType,
//     pub keywords: KeywordSet<ActionKeyword>,
//     pub attr_actor: (&'a EffectiveAttributes, &'a ActiveEffects),
// }

// impl<'a> SimpleAction<'a> {
//     pub fn into_check(self) -> Check {
//         let attribute = self.attribute;
//         let risk_complication = self.risk_complication;
//         let mut flip_modifier = FlipModifier::new();

//         if self.difficulty_modifier != 0 {
//             flip_modifier.add(
//                 -self.difficulty_modifier,
//                 FlipModifierSource::situational("Action difficulty"),
//             );
//         }

//         for eff in self.attr_actor.1.for_action(self.keywords) {
//             match eff.effect {
//                 Effect::BoonOrBane(delta) => {
//                     flip_modifier.add(delta, FlipModifierSource::from(&eff.source))
//                 }
//                 _ => {}
//             }
//         }

//         let attribute_value = self.attr_actor.0.0.get(attribute);
//         if attribute_value != 0 {
//             flip_modifier.add(
//                 attribute_value,
//                 FlipModifierSource::situational("Attribute bonus"),
//             );
//         }

//         Check {
//             attribute,
//             flip_modifier,
//             risk_complication,
//         }
//     }
// }

// pub struct ApplicableEffects<'a> {
//     keywords: KeywordSet<ActionKeyword>,
//     effects: Vec<(KeywordSet<ActionKeyword>, &'a ActiveEffects)>,
// }

// impl<'a> ApplicableEffects<'a> {
//     pub fn new(keywords: KeywordSet<ActionKeyword>) -> Self {
//         Self {
//             keywords,
//             effects: vec![],
//         }
//     }

//     pub fn action(mut self, approach: AttributeType, actor_effects: &'a ActiveEffects) -> Self {
//         let approach_kw = match approach {
//             AttributeType::PhysicalStr => ActionKeyword::ActWithPhysicalStr,
//             AttributeType::PhysicalAg => ActionKeyword::ActWithPhysicalAg,
//             AttributeType::MentalStr => ActionKeyword::ActWithMentalStr,
//             AttributeType::MentalAg => ActionKeyword::ActWithMentalAg,
//         };

//         self.effects.push((
//             self.keywords
//                 .add(ActionKeyword::WhenActing)
//                 .add(approach_kw),
//             actor_effects,
//         ));
//         self
//     }

//     pub fn target(mut self, target_effects: &'a ActiveEffects) -> Self {
//         self.effects
//             .push((self.keywords.add(ActionKeyword::WhenTarget), target_effects));
//         self
//     }
// }

pub struct SimpleAction {
    pub risk_complication: bool,
    pub attribute: AttributeType,
    pub attributes: Attributes,
}

impl<'a> SimpleAction {
    pub fn into_check(self, active_effects: impl IntoIterator<Item = &'a ActiveEffect>) -> Check {
        let attribute = self.attribute;
        let risk_complication = self.risk_complication;
        let mut skill_modifier = CheckModifier::new();

        for eff in active_effects {
            match eff.effect {
                Effect::BoonOrBane(delta) => {
                    skill_modifier.add(delta, FlipModifierSource::from(&eff.source))
                }
                _ => {}
            }
        }

        Check {
            skill: self.attributes.get(attribute),
            attribute,
            check_modifier: skill_modifier,
            risks: if risk_complication { 1 } else { 0 },
        }
    }
}

pub struct Check {
    attribute: AttributeType, // TODO: use attribute to apply bonus if suite matches
    skill: i8,
    check_modifier: CheckModifier,
    risks: u8,
}

impl Check {
    pub fn effort(mut self, effort: Card) -> Self {
        if suite_match(self.attribute, &effort) {
            self.check_modifier
                .add(1, FlipModifierSource::situational("Effort attribute match"));
        }

        if effort.value() < 5 {
            self.check_modifier
                .add(-1, FlipModifierSource::situational("Low effort"));
        } else if effort.value() >= 10 {
            self.check_modifier
                .add(1, FlipModifierSource::situational("High effort"));
        }

        self
    }

    pub fn perform_check(self, deck: &mut Deck) -> CheckResult {
        let num_flips = self.risks + 1;
        let mut cards = vec![];
        let mut magnitude_so_far = 0;
        let mut lowest_fail = 11;

        for flip in (1..=num_flips).map(|_| deck.deal()) {
            let success = self.check_modifier.test(flip, self.skill);
            if success && flip.value() > magnitude_so_far {
                magnitude_so_far = flip.value();
            } else if !success && flip.value() < lowest_fail {
                lowest_fail = flip.value();
            }
            cards.push(flip);
        }

        let success = match magnitude_so_far {
            1..=4 => Magnitude::Minor,
            5..=7 => Magnitude::Normal,
            8..=10 => Magnitude::Major,
            _ => Magnitude::None,
        };

        let complication = match (self.risks > 0, lowest_fail) {
            (false, _) => Magnitude::None,
            (true, 1..=3) => Magnitude::Major,
            (true, 4..=6) => Magnitude::Normal,
            (true, _) => Magnitude::Minor,
        };

        CheckResult {
            check: Some((self.skill, self.attribute, self.check_modifier)),
            cards,
            complication,
            success,
        }
    }
}
