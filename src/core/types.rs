use bevy::{asset::Asset, prelude::*, reflect::TypePath};
use serde::Deserialize;

use super::{Card, DC, Suite};

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

#[derive(Debug, Clone, Copy)]
pub struct EffectiveAttributes(pub Attributes);

impl Attributes {
    pub fn action_mod(mut self, card: &Card, modifier: i8) -> Self {
        match card.suite() {
            Suite::Clubs => {
                self.physical_strength += modifier;
            }
            Suite::Spades => {
                self.physical_agility += modifier;
            }
            Suite::Hearts => {
                self.mental_strength += modifier;
            }
            Suite::Diamonds => {
                self.mental_agility += modifier;
            }
        }
        self
    }

    pub fn effectiv_attributes(&self, health: &Health) -> EffectiveAttributes {
        let mut result = *self;
        for card in health.wounds.iter() {
            result = result.action_mod(card, -1);
        }
        EffectiveAttributes(result)
    }
}

#[derive(Debug, Clone, Deserialize, Asset, TypePath)]
pub struct ActorTemplates(pub Vec<(String, ActorTemplate)>);

#[derive(Debug, Clone, Deserialize, Asset, TypePath)]
pub struct AttackTemplates(pub Vec<(String, AttackOptionTemplate)>);

#[derive(Debug, Clone, Copy)]
pub struct AttributeRequirements([i8; 4]);
impl AttributeRequirements {
    fn dc_mod(&self, attributes: &Attributes) -> i32 {
        let y = [
            attributes.physical_strength,
            attributes.physical_agility,
            attributes.mental_strength,
            attributes.mental_agility,
        ];

        self.0
            .iter()
            .zip(y.iter())
            .map(|(req_attr_val, eff_val)| {
                if eff_val < req_attr_val {
                    1
                } else if *req_attr_val > 0 && eff_val > req_attr_val {
                    -1
                } else {
                    0
                }
            })
            .sum::<i32>()
            .clamp(i32::MIN, 1)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct EffortRequirements(i8, i8);
impl EffortRequirements {
    fn dc_mod(&self, effort: &Card) -> i32 {
        let EffortRequirements(req_effort_min, req_effort_max) = self;
        let effort_val = effort.value_high() as i8;

        if effort_val < *req_effort_min {
            1
        } else if effort_val >= *req_effort_max {
            -1
        } else {
            0
        }
    }
}

#[derive(Debug, Clone)]
pub struct ActionCheck {
    pub req_attributes: AttributeRequirements,
    pub req_effort: EffortRequirements,
    pub risk_complication: bool,
}

impl ActionCheck {
    pub fn difficulty(&self, effort: &Card, attributes: &Attributes) -> DC {
        let result = 8 + self.req_effort.dc_mod(effort) + self.req_attributes.dc_mod(attributes);
        DC(result.clamp(0, 20) as u8)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub enum AttackOptionTemplate {
    MeleeAttack {
        name: String,
        damage: u8,
        penetration: u8,
        req_attributes: Vec<(AttributeType, i8)>,
        req_effort: (Option<i8>, Option<i8>),
    },
}

impl AttackOptionTemplate {
    pub fn into_attack_option(self) -> AttackOption {
        match self {
            AttackOptionTemplate::MeleeAttack {
                name,
                damage,
                penetration,
                req_attributes,
                req_effort,
            } => {
                let mut ra = [i8::MIN; 4];

                for (attr, min) in req_attributes.iter() {
                    match attr {
                        AttributeType::PhysicalStr => ra[0] = *min,
                        AttributeType::PhysicalAg => ra[1] = *min,
                        AttributeType::MentalStr => ra[2] = *min,
                        AttributeType::MentalAg => ra[3] = *min,
                    }
                }

                AttackOption {
                    name,
                    target_type: AttackTargetType::MeleeSingle,
                    data: AttackData {
                        damage,
                        penetration,
                        req_attributes: AttributeRequirements(ra),
                        req_effort: EffortRequirements(
                            req_effort.0.unwrap_or(i8::MIN),
                            req_effort.1.unwrap_or(i8::MAX),
                        ),
                    },
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct AttackData {
    pub damage: u8,
    pub penetration: u8,
    pub req_attributes: AttributeRequirements,
    pub req_effort: EffortRequirements,
}

#[derive(Debug, Clone)]
pub enum AttackTargetType {
    MeleeSingle,
}

#[derive(Debug, Clone)]
pub struct AttackOption {
    pub name: String,
    pub data: AttackData,
    pub target_type: AttackTargetType,
}

impl AttackOption {
    pub fn can_attack(&self, distance: i32) -> bool {
        match self.target_type {
            AttackTargetType::MeleeSingle => distance == 1,
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
}
