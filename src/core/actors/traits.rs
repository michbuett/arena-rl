use crate::core::DisplayStr;

#[derive(Debug, Clone)]
pub struct Trait {
    pub name: DisplayStr,
    pub effects: Vec<Effect>,
    pub source: TraitSource,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum TraitSource {
    IntrinsicProperty,
    Temporary(u8),
}

#[derive(Debug, Clone)]
pub enum Effect {
    /// (attribute, bonus/malus)
    AttrMod(Attr, i8),

    /// (name, reach, to-hit, to-wound)
    MeleeAttack(DisplayStr, u8, i8, i8),

    /// (name, min-distance, max-distance, to-hit, to-wound)
    RangeAttack(DisplayStr, u8, u8, i8, i8),

    MeleeDefence(DisplayStr, i8),
    GiveTrait(DisplayStr, AbilityTarget, Trait),
    // Ability(DisplayStr, AbilityTarget, Ability),
    Recovering,
    // Dying,
}

#[derive(Debug, Clone)]
pub enum AbilityTarget {
    OnSelf,
    // OnOther,
    // OnTile,
}

// #[derive(Debug, Clone)]
// pub enum Ability {
//     // MeleeAttack(DisplayStr, u8, i8, i8),
//     // RangedAttack(AttackOption),
//     GiveTrait(Trait),
//     Recover,
//     // BuffSelf(Box<Trait>),
//     // BuffOther(Box<Trait>),
//     // Aura(Box<Trait>),
// }

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Attr {
    Wound,
    // MeleeAttack,
    MeleeDefence,
    // RangeAttack,
    RangeDefence,
    ToHit,
    ToWound,
    Defence,
    Protection,
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
