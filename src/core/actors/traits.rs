use std::cmp::max;

use crate::core::DisplayStr;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct ProtoTrait {
    name: String,
    effects: Vec<Effect>,
    source: TraitSource,
    visuals: Option<(u8, String)>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Trait {
    pub name: DisplayStr,
    pub effects: Vec<Effect>,
    pub source: TraitSource,
    pub visuals: Option<(u8, String)>,
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize)]
pub enum TraitSource {
    IntrinsicProperty,
    Temporary(u8),
}

#[derive(Debug, Clone, Deserialize)]
pub enum Effect {
    /// (attribute, bonus/malus)
    AttrMod(Attr, i8),

    /// (name, reach, to-hit, to-wound)
    MeleeAttack {
        name: DisplayStr,
        distance: u8,
        to_hit: i8,
        to_wound: i8,
        fx: String,
    },

    /// (name, min-distance, max-distance, to-hit, to-wound)
    RangeAttack {
        name: DisplayStr,
        distance: (u8, u8),
        to_hit: i8,
        to_wound: i8,
        fx: String,
    },

    /// (modifier, type)
    Defence(i8, DefenceType),

    /// (key, trait, target)
    GiveTrait(String, Trait, AbilityTarget),

    GatherStrength,
}

#[derive(Debug, Clone, Deserialize)]
pub enum DefenceType {
    Dodge(u32, u32),
    Block,
    Parry,
    TakeCover,
}

#[derive(Debug, Clone, Deserialize)]
pub enum AbilityTarget {
    OnSelf,
    // OnOther,
    // OnTile,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Deserialize)]
pub enum Attr {
    MeleeDefence,
    RangeDefence,
    ToHit,
    ToWound,
    Protection,
    Physical,
    Movement,
}

const ATTR_BASE_VALUE: i8 = 3;

#[derive(Debug, Clone)]
pub struct AttrVal(Vec<(DisplayStr, i8)>, u8);

impl AttrVal {
    pub fn new(attr: Attr, effects: &Vec<(DisplayStr, Effect)>) -> AttrVal {
        let modifier_effects: Vec<(DisplayStr, i8)> = effects
            .iter()
            .filter(|(_, e)| match e {
                Effect::AttrMod(a, _) => *a == attr,
                _ => false,
            })
            .map(|(n, e)| match e {
                Effect::AttrMod(_, m) => (n.clone(), *m),
                _ => panic!("Unexpected effect {:?} while creating AttrVal", e),
            })
            .collect();

        let sum: i8 = modifier_effects.iter().map(|(_, m)| m).sum();
        let value = max(0, ATTR_BASE_VALUE + sum) as u8;

        Self(modifier_effects, value)
    }

    /// The absolute attribute value (i.e. base value and all modifier)
    pub fn abs_val(&self) -> u8 {
        self.1
    }

    pub fn val(&self) -> i8 {
        self.0.iter().map(|(_, m)| m).sum()
    }

    pub fn modify(mut self, name: DisplayStr, modifier: i8) -> Self {
        self.0.push((name, modifier));
        self.1 = max(0, self.1 as i8 + modifier) as u8;
        self
    }
}
