use bevy::{asset::Asset, prelude::Resource, prelude::*, reflect::TypePath};
use core::panic;
use serde::Deserialize;
use std::collections::HashMap;

use super::Visual;

#[derive(Debug, Clone, Deserialize, Asset, TypePath)]
pub struct ActionTemplates(Vec<(String, AttackOption)>);

#[derive(Debug, Clone, Deserialize, Asset, TypePath)]
pub struct ActorTemplates(Vec<(String, ActorTemplate)>);

#[derive(Debug, Clone, Deserialize)]
pub enum AttackOption {
    MeleeAttack { name: String },
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
struct ActorTemplate {
    attacks: Vec<String>,
    visual: Vec<String>,
}

#[derive(Resource)]
pub struct ActorGenerator {
    action_templates: HashMap<String, AttackOption>,
    actor_templates: HashMap<String, ActorTemplate>,
}

impl ActorGenerator {
    pub fn new(
        action_templates_assets: &Res<Assets<ActionTemplates>>,
        actor_templates_assets: &Res<Assets<ActorTemplates>>,
    ) -> Self {
        let mut action_templates: HashMap<String, AttackOption> = HashMap::new();
        for (_, at) in action_templates_assets.iter() {
            for (key, action_template) in at.0.iter() {
                action_templates.insert(key.clone(), action_template.clone());
            }
        }

        let mut actor_templates: HashMap<String, ActorTemplate> = HashMap::new();
        for (_, at) in actor_templates_assets.iter() {
            for (key, actor_template) in at.0.iter() {
                actor_templates.insert(key.clone(), actor_template.clone());
            }
        }

        Self {
            action_templates,
            actor_templates,
        }
    }

    pub fn generate_actor(&self, template_name: &str) -> impl Bundle {
        let Some(actor_template) = self.actor_templates.get(template_name) else {
            panic!("Unknown actor template '{}'", template_name);
        };

        let attack_options = actor_template
            .attacks
            .iter()
            .map(|attack_name| self.action_templates.get(attack_name).cloned())
            .flatten()
            .collect::<Vec<_>>();

        let visual = if actor_template.visual.len() == 1 {
            Visual::Single(actor_template.visual.first().unwrap().clone())
        } else {
            Visual::Multi(actor_template.visual.clone())
        };

        (visual, Attacks(attack_options))
    }
}
