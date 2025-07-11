use std::collections::HashMap;

use bevy::prelude::*;

use crate::{
    assets::{ActionTemplates, ActorTemplate, ActorTemplates, AttackOption, Attacks, Visual},
    core::{Deck, ProgressCheck},
};

use super::actor::Health;

pub fn setup_generators(
    action_templates_assets: Res<Assets<ActionTemplates>>,
    actor_templates_assets: Res<Assets<ActorTemplates>>,
    mut commands: Commands,
) {
    commands.insert_resource(ActorGenerator::new(
        &action_templates_assets,
        &actor_templates_assets,
    ));
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

        let attrubute_values = actor_template.attribute_values.clone();
        let mut deck = Deck::new_rnd();
        let max_health_mod = ProgressCheck::base()
            .modify_magnitude(attrubute_values.physical_strength)
            .perform_check(&mut deck)
            .sum();

        (
            Visual::from(&actor_template.visual),
            Attacks(attack_options),
            actor_template.attribute_values.clone(),
            Health::new(10 + max_health_mod),
        )
    }
}
