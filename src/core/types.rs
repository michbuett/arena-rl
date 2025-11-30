use bevy::prelude::*;
use serde::Deserialize;

use super::{ActionKeyword, CheckResult, Effect, FeatKey, FeatType, KeywordSet, Magnitude};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize)]
pub enum AttributeType {
    PhysicalStr,
    PhysicalAg,
    MentalStr,
    MentalAg,
}

#[derive(Debug, Clone, Copy, Component, Deserialize)]
pub struct Attributes {
    pub physical_strength: i8,
    pub physical_agility: i8,
    pub mental_strength: i8,
    pub mental_agility: i8,
}

impl Attributes {
    pub fn get(&self, attribute: AttributeType) -> i8 {
        match attribute {
            AttributeType::PhysicalStr => self.physical_strength,
            AttributeType::PhysicalAg => self.physical_agility,
            AttributeType::MentalStr => self.mental_strength,
            AttributeType::MentalAg => self.mental_agility,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Asset, TypePath)]
pub struct ActorTemplates(pub Vec<(String, ActorTemplate)>);

#[derive(Debug, Clone, Deserialize)]
pub struct ActionEffects(Vec<(ActionEffectTrigger, Magnitude, ActionEffect)>);

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
pub enum ActionEffectTrigger {
    // Always,
    Success,
    Complication,
}

impl ActionEffects {
    pub fn effects_for_result(
        &self,
        check_result: &CheckResult,
    ) -> Vec<(ActionEffectTrigger, ActionEffect)> {
        // let mut result = self.effects_for_trigger(ActionEffectTrigger::Always, Magnitude::None);
        let mut result = vec![];

        if check_result.is_success() {
            result.append(
                &mut self.effects_for_trigger(ActionEffectTrigger::Success, check_result.success),
            );
        }

        if check_result.has_complication() {
            result.append(
                &mut self.effects_for_trigger(
                    ActionEffectTrigger::Complication,
                    check_result.complication,
                ),
            );
        }

        result
    }

    fn effects_for_trigger(
        &self,
        trigger: ActionEffectTrigger,
        magnitude: Magnitude,
    ) -> Vec<(ActionEffectTrigger, ActionEffect)> {
        let result = self
            .0
            .iter()
            .filter_map(|(tr, mg, eff)| {
                if *tr == trigger && *mg <= magnitude {
                    return Some((*tr, eff.clone()));
                }
                None
            })
            .fold(vec![], fold_effects);
        // .collect();

        result
    }
}

fn fold_effects(
    mut list_so_far: Vec<(ActionEffectTrigger, ActionEffect)>,
    (trigger, eff): (ActionEffectTrigger, ActionEffect),
) -> Vec<(ActionEffectTrigger, ActionEffect)> {
    if let Some(idx) = list_so_far
        .iter()
        .position(|(t, e)| trigger == *t && e.can_combine(&eff))
    {
        let (t, e) = list_so_far.get(idx).unwrap();
        list_so_far.insert(idx, (*t, eff.try_combine(e).unwrap()));
    } else {
        list_so_far.push((trigger, eff));
    }
    list_so_far
}
// fn filter_action_effect(
//     eff: &ActionEffect,
//     trigger: &ActionEffectTrigger,
//     check_result: &CheckResult,
// ) -> Option<ActionEffect> {
//     // if check_result.complication
//     match (trigger, &check_result.success, &check_result.complication) {
//         (ActionEffectTrigger::Always, _, _) => Some(eff.clone()),

//         (ActionEffectTrigger::Success, Magnitude::Minor, _)
//         | (ActionEffectTrigger::SuccessNormal, Magnitude::Normal, _)
//         | (ActionEffectTrigger::SuccessMajor, Magnitude::Major, _) => Some(eff.clone()),

//         (ActionEffectTrigger::ComplicationMajor, _, Magnitude::Minor)
//         | (ActionEffectTrigger::ComplicationNormal, _, Magnitude::Normal)
//         | (ActionEffectTrigger::Complication, _, Magnitude::Major) => Some(eff.clone()),

//         _ => None,
//     }
// }

impl From<RawActionEffects> for ActionEffects {
    fn from(value: RawActionEffects) -> Self {
        use ActionEffectTrigger::*;
        use Magnitude::*;
        match value {
            RawActionEffects::Always(eff) => Self(vec![(Success, Normal, eff)]),
            RawActionEffects::LowNormalHigh(e1, e2, e3) => Self(vec![
                (Success, Minor, e1),
                (Success, Normal, e2),
                (Success, Major, e3),
            ]),
        }
    }
}
// impl From<Vec<(ActionEffectTrigger, Magnitude, ActionEffect)>> for ActionEffects {
//     fn from(value: Vec<(ActionEffectTrigger, Magnitude, ActionEffect)>) -> Self {
//         Self(value)
//     }
// }

// impl From<ActionCheck> for ActionEffects {
//     fn from(value: ActionCheck) -> Self {
//         use ActionEffectTrigger::*;
//         use Magnitude::*;
//         match value {
//             ActionCheck::NoCheck(eff) => Self(vec![(Always, None, eff)]),
//             ActionCheck::Check { effects, .. } => Self(effects.clone()),
//         }
//     }
// }

#[derive(Debug, Clone, Deserialize)]
pub enum ActionEffect {
    NoEffect,
    DamageTarget(u8),
    Protection(u8),
    TempEffect {
        effect: Effect,
        descr: String,
        keywords: Vec<ActionKeyword>,
        turns: u8,
    },
    // Multi(Vec<ActionEffect>),
}

impl ActionEffect {
    fn try_combine(&self, other: &ActionEffect) -> Option<ActionEffect> {
        match (self, other) {
            (Self::Protection(p1), Self::Protection(p2)) => Some(Self::Protection(p1 + p2)),
            (Self::DamageTarget(p1), Self::DamageTarget(p2)) => Some(Self::DamageTarget(p1 + p2)),
            _ => None,
        }
    }

    fn can_combine(&self, other: &ActionEffect) -> bool {
        self.try_combine(other).is_some()
    }
}

#[derive(Debug, Clone, Deserialize)]
pub enum ActionFx {
    SelfTxt(String),
    SingleTargetMeleeAttack(String),
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawActionTemplate {
    // pub attribute: AttributeType,
    // pub effect: RawActionEffects,
    pub fx: ActionFx,
    // pub keywords: Vec<ActionKeyword>,
    pub name: String,
    pub check: ActionCheck,
}

#[derive(Debug, Clone, Deserialize)]
pub enum ActionCheck {
    NoCheck(ActionEffects),
    Check {
        effects: ActionEffects,
        // effects: Vec<(ActionEffectTrigger, Magnitude, ActionEffect)>,
        // effect: (ActionEffect, ActionEffect, ActionEffect),
        // complication: (ActionEffect, ActionEffect, ActionEffect),
        risky: bool,
        attribute: AttributeType,
        keywords: KeywordSet<ActionKeyword>,
    },
}

/// An abstraction to make defining the effects for an action more convinient
#[derive(Debug, Clone, Deserialize)]
pub enum RawActionEffects {
    Always(ActionEffect),
    LowNormalHigh(ActionEffect, ActionEffect, ActionEffect),
}

#[derive(Debug, Clone, Deserialize)]
pub struct ActorTemplate {
    pub attacks: Vec<String>,
    pub visual: Vec<String>,
    pub attributes: Attributes,
    pub feats: Option<Vec<String>>,
}

#[derive(Component, Debug, Clone)]
pub struct ActiveDefence(pub Resistance);

#[derive(Component, Debug, Clone)]
pub struct PassiveDefence(pub Vec<Resistance>);

impl PassiveDefence {
    pub fn total_resistance(&self) -> u8 {
        self.0.iter().map(|r| r.resistance).sum()
    }
}

#[derive(Debug, Clone)]
pub struct Resistance {
    pub source: (String, FeatType),
    pub resistance: u8,
}

impl Resistance {
    pub fn new(source: (String, FeatType), resistance: u8) -> Self {
        Self { source, resistance }
    }
}
#[derive(Debug, Component)]
pub struct Items(pub Vec<Item>);

#[derive(Debug)]
pub struct Item {
    pub feat_ref: String,
    pub key: FeatKey,
    pub name: String,
    pub state: ItemState,
}

#[derive(Debug)]
pub enum ItemState {
    New,
    Damaged,
    Broken,
}

#[derive(Component, Debug, Clone)]
pub struct Health {
    pub max_health: u8,
    pub damage_taken: u8,
}

impl Health {
    pub fn new(max_health: u8) -> Self {
        Self {
            max_health,
            damage_taken: 0,
        }
    }

    pub fn damage(&mut self, amount: u8) {
        self.damage_taken += amount;
    }

    pub fn damage_total(&self) -> u8 {
        self.damage_taken
    }

    pub fn is_alive(&self) -> bool {
        self.max_health > self.damage_total()
    }
}
