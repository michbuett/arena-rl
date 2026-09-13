use std::cmp::{max, min};

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

    pub fn tn(&self, base_tn: impl Into<i32>) -> (u8, u8) {
        let chance_modifier: i32 = self.0.iter().map(|(d, _)| *d).sum();
        let raw_tn = base_tn.into() + chance_modifier;
        let check_tn = raw_tn.clamp(3, 7);
        let ace_tn = max(0, raw_tn - check_tn + 1);

        (ace_tn as u8, check_tn as u8)
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
            risk_complication,
        }
    }
}

pub struct Check {
    attribute: AttributeType, // TODO: use attribute to apply bonus if suite matches
    skill: i8,
    check_modifier: CheckModifier,
    risk_complication: bool,
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
        let (cards, success, complication) = if self.risk_complication {
            self.perform_check_with_risk(deck)
        } else {
            self.perform_check_without_risk(deck)
        };

        CheckResult {
            cards,
            success,
            complication,
            check: Some((self.skill, self.attribute, self.check_modifier)),
        }
    }

    fn perform_check_with_risk(&self, deck: &mut Deck) -> (Vec<Card>, Magnitude, Magnitude) {
        let (ace_tn, check_tn) = self.check_modifier.tn(self.skill);
        let (flip1, flip2) = (deck.deal(), deck.deal());
        let high_val = max(flip1.value(), flip2.value());
        let low_val = min(flip1.value(), flip2.value());

        if low_val <= ace_tn || high_val <= check_tn {
            return (
                vec![flip1, flip2],
                success_magnitude(true, high_val),
                Magnitude::None,
            );
        }

        if low_val <= check_tn {
            return (
                vec![flip1, flip2],
                success_magnitude(true, low_val),
                complication_magnitude(high_val),
            );
        }

        return (
            vec![flip1, flip2],
            Magnitude::None,
            complication_magnitude(low_val),
        );
    }

    fn perform_check_without_risk(&self, deck: &mut Deck) -> (Vec<Card>, Magnitude, Magnitude) {
        let (ace_tn, check_tn) = self.check_modifier.tn(self.skill);
        let flip = deck.deal();
        if flip.value() <= ace_tn {
            let flip2 = deck.deal();
            let high_val = max(flip.value(), flip2.value());

            return (
                vec![flip, flip2],
                success_magnitude(true, high_val),
                Magnitude::None,
            );
        }

        if flip.value() <= check_tn {
            return (
                vec![flip],
                success_magnitude(true, flip.value()),
                Magnitude::None,
            );
        }

        // The check failed
        // => No success, but also no complication
        (vec![flip], Magnitude::None, Magnitude::None)
    }
}

fn success_magnitude(success: bool, card_value: u8) -> Magnitude {
    match (success, card_value) {
        (false, _) => Magnitude::None,
        (true, 1..=4) => Magnitude::Minor,
        (true, 5..=7) => Magnitude::Normal,
        (true, 8..=10) => Magnitude::Major,
        _ => unreachable!(),
    }
}

fn complication_magnitude(card_value: u8) -> Magnitude {
    match card_value {
        8..=10 => Magnitude::Minor,
        5..=7 => Magnitude::Normal,
        1..=4 => Magnitude::Major,
        _ => unreachable!(),
    }
}
