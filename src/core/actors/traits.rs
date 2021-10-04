use crate::core::{Tile, DisplayStr};
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
    RangeAttack{
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

#[derive(Debug, Clone)]
pub struct AttrVal(Vec<(DisplayStr, i8)>);

impl AttrVal {
    pub fn new(attr: Attr, effects: &Vec<(DisplayStr, Effect)>) -> AttrVal {
        Self(effects.iter().filter(|(_, e)| {
            match e {
                Effect::AttrMod(a, _) => *a == attr,
                _ => false
            }
        }).map(|(n, e)| {
            match e {
                Effect::AttrMod(_, m) => (n.clone(), *m),
                _ => panic!("Unexpected effect {:?} while creating AttrVal", e),
            }
        }).collect())
    }
    
    pub fn val(&self) -> i8 {
       self.0.iter().map(|(_, m)| m).sum()
    }

    pub fn modify(mut self, name: DisplayStr, modifier: i8) -> Self {
        self.0.push((name, modifier));
        self
    }
}
