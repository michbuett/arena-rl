use std::collections::HashMap;

use bevy::{asset::Asset, prelude::*, reflect::TypePath};
use serde::Deserialize;

use crate::core::StatusKeyword;

use super::{ActionKeyword, KeywordSet};

#[derive(Debug, Component, Clone, Deserialize, Asset, TypePath)]
pub struct Feats(pub Vec<(String, Feat)>);

#[derive(Debug, Clone, Deserialize)]
pub struct Feat {
    pub name: String,
    pub feat_type: FeatType,
    pub effects: Vec<(Effect, KeywordSet<ActionKeyword>)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FeatKey(pub usize);

#[derive(Debug, Clone)]
pub struct FeatEffect {
    pub key: FeatKey,
    pub effects: Vec<(Effect, KeywordSet<ActionKeyword>)>,
}

#[derive(Debug, Clone)]
pub struct FeatDescription {
    // pub key: FeatKey,
    pub name: String,
    pub feat_type: FeatType,
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub enum FeatType {
    Intrinsic,
    Item,
    // Temporary(u8),
}

pub struct FeatStore {
    effects: Vec<FeatEffect>,
    descriptions: Vec<FeatDescription>,
    refs: HashMap<String, FeatKey>,
}

impl FeatStore {
    pub fn new(feats_assets: &Res<Assets<Feats>>) -> Self {
        let mut effects = vec![];
        let mut descriptions = vec![];
        let mut refs = HashMap::new();

        for (feat_ref, feat) in feats_assets.iter().flat_map(|(_, fl)| fl.0.iter()) {
            let key = FeatKey(effects.len());
            let eff = FeatEffect {
                key,
                effects: feat.effects.clone(),
            };

            let desc = FeatDescription {
                // key,
                name: feat.name.to_string(),
                feat_type: feat.feat_type,
            };

            refs.insert(feat_ref.to_string(), key);
            effects.insert(key.0, eff);
            descriptions.insert(key.0, desc);
        }

        Self {
            effects,
            descriptions,
            refs,
        }
    }

    pub fn find_key(&self, feat_ref: &str) -> Option<FeatKey> {
        self.refs.get(feat_ref).copied()
    }

    pub fn effect(&self, key: FeatKey) -> &FeatEffect {
        self.effects.get(key.0).unwrap()
    }

    pub fn description(&self, key: FeatKey) -> &FeatDescription {
        self.descriptions.get(key.0).unwrap()
    }
}

#[derive(Debug, Clone, Deserialize)]
pub enum Effect {
    Resistance(u8),
    BoonOrBane(i8),
    Status(KeywordSet<StatusKeyword>),
}

#[derive(Debug, Clone)]
pub struct ActiveEffect {
    pub effect: Effect,
    pub keywords: KeywordSet<ActionKeyword>,
    pub source: ActiveEffectSource,
}

#[derive(Debug, Clone)]
pub enum ActiveEffectSource {
    Temporary(u8, String),
    Feat(FeatKey),
}

#[derive(Debug, Clone, Component)]
pub struct ActiveEffects(Vec<ActiveEffect>);

impl ActiveEffects {
    pub fn new(feat_effects: &Vec<FeatEffect>) -> Self {
        let mut active_effects = vec![];
        for fe in feat_effects.iter() {
            for (e, keywords) in fe.effects.iter() {
                active_effects.push(ActiveEffect {
                    effect: e.clone(),
                    keywords: *keywords,
                    source: ActiveEffectSource::Feat(fe.key),
                })
            }
        }

        Self(active_effects)
    }

    pub fn add_temporary_eff(
        &mut self,
        effect: Effect,
        keywords: KeywordSet<ActionKeyword>,
        turns: u8,
        descr: String,
    ) {
        self.0.push(ActiveEffect {
            effect,
            keywords,
            source: ActiveEffectSource::Temporary(turns, descr),
        });
    }

    pub fn new_turn(&self) -> Self {
        Self(
            self.0
                .iter()
                .filter_map(|eff| match &eff.source {
                    ActiveEffectSource::Temporary(turns, descr) => {
                        if *turns > 1 {
                            Some(ActiveEffect {
                                effect: eff.effect.clone(),
                                keywords: eff.keywords,
                                source: ActiveEffectSource::Temporary(turns - 1, descr.clone()),
                            })
                        } else {
                            None
                        }
                    }
                    _ => Some(eff.clone()),
                })
                .collect(),
        )
    }

    pub fn for_action(
        &self,
        action_keywords: KeywordSet<ActionKeyword>,
    ) -> impl Iterator<Item = &ActiveEffect> {
        self.0
            .iter()
            .filter(move |ae| action_keywords.contains(ae.keywords))
    }
}
