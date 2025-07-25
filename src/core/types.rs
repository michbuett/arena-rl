use bevy::{asset::Asset, prelude::*, reflect::TypePath};
use serde::Deserialize;
use std::cmp::max;

use super::Card;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize)]
pub enum AttributeType {
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

#[derive(Debug, Clone, Copy, Component, Deserialize)]
pub struct Attributes {
    pub physical_strength: i8,
    pub physical_agility: i8,
    pub mental_strength: i8,
    pub methal_agility: i8,
}

impl Attributes {
    pub fn get(&self, attr: AttributeType) -> i8 {
        use AttributeType::*;
        match attr {
            PhysicalStr => self.physical_strength,
            PhysicalAg => self.physical_agility,
            MentalStr => self.mental_strength,
            MentalAg => self.methal_agility,
            Strength => max(self.physical_strength, self.mental_strength),
            Agility => max(self.physical_agility, self.methal_agility),
            Physical => max(self.physical_strength, self.physical_agility),
            Mental => max(self.mental_strength, self.methal_agility),
            Any => max(self.get(Physical), self.get(Mental)),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Asset, TypePath)]
pub struct ActionTemplates(pub Vec<(String, AttackOption)>);

#[derive(Debug, Clone, Deserialize, Asset, TypePath)]
pub struct ActorTemplates(pub Vec<(String, ActorTemplate)>);

#[derive(Debug, Clone, Deserialize)]
pub enum AttackOption {
    MeleeAttack {
        name: String,
        attribute: AttributeType,
        damage: u8,
        penetration: u8,
        difficulty: u8,
    },
}

impl AttackOption {
    pub fn can_attack(&self, distance: i32) -> bool {
        match self {
            AttackOption::MeleeAttack { .. } => distance == 1,
        }
    }
}

#[derive(Debug, Clone, Component)]
pub struct Attacks(pub Vec<AttackOption>);

#[derive(Debug, Clone, Deserialize)]
pub struct ActorTemplate {
    pub attacks: Vec<String>,
    pub visual: Vec<String>,
    pub attributes: Attributes,
    pub feats: Option<Vec<String>>,
}

#[derive(Debug, Component, Clone, Deserialize, Asset, TypePath)]
pub struct Feats(pub Vec<(String, Feat)>);

#[derive(Debug, Clone, Deserialize)]
pub struct Feat {
    pub name: String,
    pub source: FeatSource,
    pub effects: Vec<Effect>,
}

// #[derive(Debug, Clone, Deserialize)]
// pub struct FeatKey(pub String);

#[derive(Debug, Clone, Copy, Deserialize)]
pub enum FeatSource {
    Intrinsic,
    Item,
    // Temporary(u8),
}

#[derive(Debug, Clone, Deserialize)]
pub enum Effect {
    Resistance(u8),
}

// #[derive(Debug, Clone, Copy, Deserialize)]
// pub struct Keyword(u8);

#[derive(Component, Debug, Clone)]
pub struct Protection(pub Vec<Resistance>);

impl Protection {
    pub fn total_resistance(&self) -> u8 {
        self.0.iter().map(|r| r.resistance).sum()
    }
}

#[derive(Debug, Clone)]
pub struct Resistance {
    pub source: (String, FeatSource),
    pub resistance: u8,
}

impl Resistance {
    pub fn new(source: (String, FeatSource), resistance: u8) -> Self {
        Self { source, resistance }
    }
}
#[derive(Debug, Component)]
pub struct Items(pub Vec<Item>);

#[derive(Debug)]
pub struct Item {
    pub key: String,
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
    pub wounds: Vec<Card>,
}

impl Health {
    pub fn new(max_health: u8) -> Self {
        Self {
            max_health,
            wounds: vec![],
        }
    }

    pub fn damage_total(&self) -> u8 {
        self.wounds.iter().map(|c| c.value_high()).sum()
    }

    pub fn is_alive(&self) -> bool {
        self.max_health > self.damage_total()
    }

    pub fn current_attribute_values(&self, base_values: Attributes) -> Attributes {
        use crate::core::Suite;

        let Attributes {
            mut physical_strength,
            mut physical_agility,
            mut mental_strength,
            mut methal_agility,
        } = base_values;

        for card in self.wounds.iter() {
            match card.suite() {
                Suite::Clubs => {
                    physical_strength -= 1;
                }
                Suite::Spades => {
                    physical_agility -= 1;
                }
                Suite::Hearts => {
                    mental_strength -= 1;
                }
                Suite::Diamonds => {
                    methal_agility -= 1;
                }
            }
        }

        Attributes {
            physical_strength,
            physical_agility,
            mental_strength,
            methal_agility,
        }
    }
}
