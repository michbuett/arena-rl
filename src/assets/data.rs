use bevy::{asset::Asset, prelude::*, reflect::TypePath};
use serde::Deserialize;

use crate::core::{Attribute, AttributeValues};

#[derive(Debug, Clone, Deserialize, Asset, TypePath)]
pub struct ActionTemplates(pub Vec<(String, AttackOption)>);

#[derive(Debug, Clone, Deserialize, Asset, TypePath)]
pub struct ActorTemplates(pub Vec<(String, ActorTemplate)>);

#[derive(Debug, Clone, Deserialize)]
pub enum AttackOption {
    MeleeAttack {
        name: String,
        attribute: Attribute,
        damage: i16,
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
    pub attribute_values: AttributeValues,
}
