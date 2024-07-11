use std::collections::HashMap;
use std::fs::File;
use std::iter::FromIterator;
use std::path::Path;

use ron::de::from_reader;

use crate::core::{DisplayStr, Suite};
use serde::Deserialize;

pub const NUM_VISUAL_STATES: usize = 4;
pub const NUM_VISUAL_LAYERS: usize = 4;
pub const NUM_ATTRIBUTE_MODIFIER: usize = 7;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Deserialize)]
pub enum VisualState {
    Idle = 0,
    Prone = 1,
    Hidden = 2,
    Dead = 3,
}

#[derive(Debug, Copy, Clone, Deserialize)]
pub enum VLayers {
    Body = 0,
    Head = 1,
    Weapon1 = 2,
    Weapon2 = 3,
}

#[derive(Debug, Copy, Clone, Deserialize)]
pub enum AttributeModifier {
    PhysicalStrength,
    PhysicalAgility,
    PhysicalResistence,
    MentalStrength,
    MentalAgility,
    MentalResitence,
    MoveDistance,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Trait {
    pub name: DisplayStr,
    pub effects: Vec<Effect>,
    pub source: TraitSource,
    pub visuals: Option<Vec<(VisualState, Vec<(VLayers, String)>)>>,
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize)]
pub enum TraitSource {
    IntrinsicProperty,
    Temporary(u8),
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub enum Keyword {
    Flying,
    Underground,
    Quick,
    Slow,
}

impl Keyword {
    pub fn as_bit(&self) -> u64 {
        let num = self.clone() as u32;
        2_u64.pow(num + 1)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub enum AttackFx {
    MeleeSingleTarget { name: String },
    Projectile { name: String },
}

#[derive(Debug, Clone, Deserialize)]
pub enum Effect {
    /// (attribute, bonus/malus)
    AttrMod(Attr, i8),

    /// (attribute, bonus/malus)
    Mod(AttributeModifier, i8),

    AttackSingleTarget {
        to_hit: (Suite, i8),
        challenge_value: u8,
        to_wound: (Suite, i8),
        defence: Suite,
        distance_max: Option<u8>,
        distance_min: Option<u8>,
        effects: Option<Vec<(HitEffectCondition, HitEffect)>>,
        fx: AttackFx,
        name: DisplayStr,
        rend: Option<u8>,
    },

    /// (name, reach, to-hit, to-wound)
    MeleeAttack {
        name: DisplayStr,

        /// The amount of effort this kind of attack requires
        required_effort: u8,

        /// The reach of the attack (defaults to 1)
        /// Values > 1 allows to attack across tiles in a straight line
        distance: Option<u8>,

        /// How far an actor will push forward during its attack (defaults to 0)
        advance: Option<u8>,

        /// A modifier of the hit roll (e.g. for a precice but less penetrating attack)
        /// Defaults to zero
        to_hit: Option<i8>,

        /// Armor Penetration. A modifier of the wound roll (e.g. for an more
        /// brutal but less precice attack)
        /// Defaults to zero
        ap: Option<i8>,

        /// A modifier of the wound roll quality (so even a slight hit may cause
        /// a terrible wound)
        /// Defaults to zero
        rend: Option<i8>,

        /// The name of the animation which is played upon attacking
        fx: String,

        /// A set of effects which a apply if the hit roll was successfull
        effects: Option<Vec<(HitEffectCondition, HitEffect)>>,
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
    GiveTrait(String, AbilityTarget),
    // GiveTrait(String, Trait, AbilityTarget),
    Ability {
        key: String,
        target: AbilityTarget,
    },

    GatherStrength,

    Keyword(Keyword),
}

#[derive(Default)]
pub struct TraitStorage {
    traits: HashMap<String, Trait>,
}

impl TraitStorage {
    pub fn new(path: &Path) -> Self {
        let p = path.join("traits.ron");
        let f = match File::open(p) {
            Ok(result) => result,
            Err(e) => {
                panic!("Error opening proto sprite config file: {:?}", e);
            }
        };

        let traits: Vec<(String, Trait)> = match from_reader(f) {
            Ok(result) => result,
            Err(e) => {
                panic!("Error parsing proto sprite config: {:?}", e);
            }
        };

        Self {
            traits: HashMap::from_iter(traits),
        }
    }

    pub fn get(&self, key: &str) -> &Trait {
        if !self.traits.contains_key(key) {
            panic!("Unknown trait: {}", key);
        }

        self.traits.get(key).unwrap()
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub enum HitEffectCondition {
    OnHit,
}

#[derive(Debug, Clone, Deserialize)]
pub enum HitEffect {
    PushBack(u8),
    PullCloser(u8),
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
    Physical,
    Movement,

    /// Offensiv and defensiv stat; compared to the MeleeSkill of the opponent
    /// to determine the chance of hitting an enemy with a melee attack
    MeleeSkill,

    /// Offensiv stat; compared to the level of obscurity (cover) to determine
    /// the chance of hitting an enemy with a ranged attack
    RangedSkill,

    /// Defensiv stat; used during melee and ranged combat to determine the
    /// chance of hitting the opponent
    Evasion,

    /// Offensiv stat, compared to amor (Protection) to determine the chance of
    /// wounding an enemy
    ArmorPenetration,

    /// Defensiv stat, compared to amor penetration to determine the
    /// chance of wounding an enemy
    Protection,

    /// Defensiv stat, increases/decreases the quality of the wound roll
    Resilience,

    /// Defensiv stat, increases/decreases the quality of a hit roll in melee combat
    MeleeBlock,
    /// Defensiv stat, increases/decreases the quality of a hit roll in ranged combat
    RangedBlock,
}
