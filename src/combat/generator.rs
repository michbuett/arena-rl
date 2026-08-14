use std::collections::HashMap;

use bevy::prelude::*;

use crate::{
    assets::Visual,
    combat::{
        actor::Activations,
        commands::{ManeuverTemplate, ManeuverTemplates},
    },
    core::{
        ActiveEffects, ActorTemplate, ActorTemplates, Deck, Effect, FeatStore, Feats, Health, Item,
        ItemState, Items, PassiveDefence, Resistance, ResistanceSource,
    },
};

use super::commands::ActorManeuvers;

pub fn setup_generators(
    maneuver_templates_assets: Res<Assets<ManeuverTemplates>>,
    actor_templates_assets: Res<Assets<ActorTemplates>>,
    feats_assets: Res<Assets<Feats>>,
    mut commands: Commands,
) {
    commands.insert_resource(ActorGenerator::new(
        &maneuver_templates_assets,
        &actor_templates_assets,
        &feats_assets,
    ));
}

#[derive(Resource)]
pub struct ActorGenerator {
    maneuver_templates: HashMap<String, ManeuverTemplate>,
    actor_templates: HashMap<String, ActorTemplate>,
    feat_store: FeatStore,
}

impl ActorGenerator {
    pub fn new(
        maneuver_templates_assets: &Res<Assets<ManeuverTemplates>>,
        actor_templates_assets: &Res<Assets<ActorTemplates>>,
        feats_assets: &Res<Assets<Feats>>,
    ) -> Self {
        let maneuver_templates: HashMap<String, ManeuverTemplate> = HashMap::from_iter(
            maneuver_templates_assets
                .iter()
                .flat_map(|(_, fl)| fl.0.iter())
                .map(|(key, raw_template)| (key.into(), raw_template.clone().into())),
        );

        let actor_templates: HashMap<String, ActorTemplate> = HashMap::from_iter(
            actor_templates_assets
                .iter()
                .flat_map(|(_, fl)| fl.0.iter())
                .cloned(),
        );

        let feat_store = FeatStore::new(feats_assets);

        Self {
            maneuver_templates,
            actor_templates,
            feat_store,
        }
    }

    pub fn generate_actor(&self, template_name: &str, deck: &mut Deck) -> impl Bundle {
        let Some(actor_template) = self.actor_templates.get(template_name) else {
            panic!("Unknown actor template '{template_name}'");
        };

        let maneuver_templates = actor_template
            .attacks
            .iter()
            .filter_map(|name| self.maneuver_templates.get(name).cloned())
            .collect::<Vec<_>>();

        let attrubute_values = actor_template.attributes;
        let max_health = (attrubute_values.physical_strength + attrubute_values.mental_strength)
            .clamp(3, i8::MAX) as u8;

        let mut resistances: Vec<Resistance> = vec![];
        let mut items: Vec<Item> = vec![];
        let mut active_effects = vec![];

        if let Some(feat_list) = actor_template.feats.as_ref() {
            for feat_ref in feat_list.iter() {
                let Some(key) = self.feat_store.find_key(feat_ref) else {
                    warn!("Unknown feat '{}'", feat_ref);
                    continue;
                };

                let descr = self.feat_store.description(key);
                if matches!(descr.feat_type, crate::core::FeatType::Item) {
                    items.push(Item {
                        key,
                        // feat_ref: feat_ref.to_string(),
                        name: descr.name.to_string(),
                        state: ItemState::New,
                    });
                }

                let feat_eff = self.feat_store().effect(key);
                for eff in feat_eff.effects.iter() {
                    if let (Effect::Resistance(resistance), _) = eff {
                        let source = ResistanceSource::Feat(key, descr.feat_type);
                        resistances.push(Resistance::new(source, *resistance));
                    }
                }

                active_effects.push(feat_eff.clone());
            }
        }

        (
            Activations::new(deck.deal()),
            Visual::from(&actor_template.visual),
            ActorManeuvers(maneuver_templates),
            ActiveEffects::new(&active_effects),
            PassiveDefence(resistances),
            Items(items),
            actor_template.attributes,
            Health::new(10 + max_health),
        )
    }

    pub fn feat_store(&self) -> &FeatStore {
        &self.feat_store
    }
}

#[test]
fn test_feats_syntax() {
    let raw_string = std::fs::read_to_string("assets/data/main.feats.ron").unwrap();
    let data: Feats = ron::from_str(&raw_string).unwrap();
    assert!(data.0.len() > 0);
}

#[test]
fn test_actor_templates_syntax() {
    let raw_string = std::fs::read_to_string("assets/data/main.actors.ron").unwrap();
    let data: ActorTemplates = ron::from_str(&raw_string).unwrap();
    assert!(data.0.len() > 0);
}

#[test]
fn test_maneuver_templates_syntax() {
    let raw_string = std::fs::read_to_string("assets/data/main.maneuvers.ron").unwrap();
    let data: ManeuverTemplates = ron::from_str(&raw_string).unwrap();
    assert!(data.0.len() > 0);
}
