use std::collections::HashMap;

use bevy::prelude::*;

use crate::{
    assets::Visual,
    core::{
        ActorTemplate, ActorTemplates, AttackOption, AttackTemplates, Attacks, Deck, Effect, Feat,
        Feats, Health, Item, ItemState, Items, ProgressCheck, Protection, Resistance,
    },
};

pub fn setup_generators(
    action_templates_assets: Res<Assets<AttackTemplates>>,
    actor_templates_assets: Res<Assets<ActorTemplates>>,
    feats_assets: Res<Assets<Feats>>,
    mut commands: Commands,
) {
    commands.insert_resource(ActorGenerator::new(
        &action_templates_assets,
        &actor_templates_assets,
        &feats_assets,
    ));
}

#[derive(Resource)]
pub struct ActorGenerator {
    action_templates: HashMap<String, AttackOption>,
    actor_templates: HashMap<String, ActorTemplate>,
    feat_templates: HashMap<String, Feat>,
}

impl ActorGenerator {
    pub fn new(
        action_templates_assets: &Res<Assets<AttackTemplates>>,
        actor_templates_assets: &Res<Assets<ActorTemplates>>,
        feats_assets: &Res<Assets<Feats>>,
    ) -> Self {
        let action_templates: HashMap<String, AttackOption> = HashMap::from_iter(
            action_templates_assets
                .iter()
                .flat_map(|(_, fl)| fl.0.iter())
                .cloned()
                .map(|(key, aot)| (key.to_string(), aot.into_attack_option())),
        );

        let actor_templates: HashMap<String, ActorTemplate> = HashMap::from_iter(
            actor_templates_assets
                .iter()
                .flat_map(|(_, fl)| fl.0.iter())
                .cloned(),
        );

        let feat_templates: HashMap<String, Feat> =
            HashMap::from_iter(feats_assets.iter().flat_map(|(_, fl)| fl.0.iter()).cloned());

        Self {
            action_templates,
            actor_templates,
            feat_templates,
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

        let attrubute_values = actor_template.attributes.clone();
        let mut deck = Deck::new_rnd();
        let max_health_mod = ProgressCheck::base()
            .modify_magnitude(attrubute_values.physical_strength)
            .perform_check(&mut deck)
            .sum();

        let mut resistances: Vec<Resistance> = vec![];
        let mut items: Vec<Item> = vec![];

        if let Some(feat_list) = actor_template.feats.as_ref() {
            for feat_name in feat_list.iter() {
                if let Some(feat) = self.feat_templates.get(feat_name) {
                    if matches!(feat.source, crate::core::FeatSource::Item) {
                        items.push(Item {
                            key: feat_name.to_string(),
                            name: feat.name.to_string(),
                            state: ItemState::New,
                        });
                    }

                    for eff in feat.effects.iter() {
                        match eff {
                            Effect::Resistance(resistance) => {
                                let source = (feat_name.to_string(), feat.source);
                                resistances.push(Resistance::new(source, *resistance));
                            }
                        }
                    }
                } else {
                    warn!("Unknown feat '{}'", feat_name);
                }
            }
        }

        (
            Visual::from(&actor_template.visual),
            Attacks(attack_options),
            Protection(resistances),
            Items(items),
            actor_template.attributes.clone(),
            Health::new(10 + max_health_mod),
        )
    }
}
