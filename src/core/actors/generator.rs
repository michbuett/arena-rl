use std::{collections::HashMap, fs::File, iter::FromIterator, path::Path};

use crate::core::WorldPos;

use super::{
    actor::{Actor, ActorBuilder, AiBehaviour, TeamId, Trait},
    ActorAttriubes, TraitStorage, VLayers, Visual, VisualElements, VisualState,
};

use ron::de::from_reader;
use serde::Deserialize;

#[derive(Default)]
pub struct ObjectGenerator {
    traits: TraitStorage,
    actors: ActorTemplateStorage,
}

impl ObjectGenerator {
    pub fn new(path: &Path) -> Self {
        Self {
            traits: TraitStorage::new(path),
            actors: ActorTemplateStorage::new(path),
        }
    }

    pub fn traits(&self) -> &TraitStorage {
        &self.traits
    }

    fn get_trait(&self, key: &str) -> (String, Trait) {
        let t = self.traits.get(key);
        (key.to_string(), t.clone())
    }

    pub fn generate_actor(
        &self,
        pos: WorldPos,
        team_id: TeamId,
        template_name: ActorTemplateName,
    ) -> ActorBuilder {
        let template = self.actors.get(&template_name);
        let mut visual = Visual::new(VisualElements::new(
            template.visuals.0.iter().map(map_visual_config).collect(),
        ));

        for (state, el) in template.visuals.1.iter() {
            let el = VisualElements::new(el.iter().map(map_visual_config).collect());
            visual = visual.add_state(*state, el);
        }

        let traits = template
            .traits
            .iter()
            .map(|trait_name| self.get_trait(trait_name))
            .collect();

        let name = format!("{}-{}", template_name.0, between(0, 1000)); // TODO generate names
        ActorBuilder::new(
            name,
            pos,
            team_id,
            template.attributes.clone(),
            template.max_activations,
        )
        .traits(traits)
        .visual(visual)
    }

    pub fn generate_player(&self, pos: WorldPos, t: TeamId, template: ActorTemplateName) -> Actor {
        self.generate_actor(pos, t, template).build()
    }

    pub fn generate_enemy(&self, pos: WorldPos, t: TeamId, template: ActorTemplateName) -> Actor {
        self.generate_actor(pos, t, template)
            .behaviour(AiBehaviour::Default)
            .build()
    }
}
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct ActorTemplateName(String);

impl ActorTemplateName {
    pub fn new(n: impl ToString) -> Self {
        Self(n.to_string())
    }
}

type VisualConfig = (VLayers, String, Option<(u16, u16)>);

#[derive(Debug, Clone, Deserialize)]
struct ActorTemplate {
    max_activations: u8,
    attributes: ActorAttriubes,
    traits: Vec<String>,
    visuals: (Vec<VisualConfig>, Vec<(VisualState, Vec<VisualConfig>)>),
}

#[derive(Default)]
pub struct ActorTemplateStorage {
    templates: HashMap<String, ActorTemplate>,
}

impl ActorTemplateStorage {
    pub fn new(path: &Path) -> Self {
        let p = path.join("actors.ron");
        let f = match File::open(p) {
            Ok(result) => result,
            Err(e) => {
                panic!("Error opening proto sprite config file: {:?}", e);
            }
        };

        let traits: Vec<(String, ActorTemplate)> = match from_reader(f) {
            Ok(result) => result,
            Err(e) => {
                panic!("Error parsing proto sprite config: {:?}", e);
            }
        };

        Self {
            templates: HashMap::from_iter(traits),
        }
    }

    fn get(&self, template_name: &ActorTemplateName) -> &ActorTemplate {
        let ActorTemplateName(key) = template_name;
        if !self.templates.contains_key(key) {
            panic!("Unknown trait: {}", key);
        }

        self.templates.get(key).unwrap()
    }
}

/////////////////////////////////////////////////////////////////////
// little helper

fn between(a: u16, b: u16) -> u16 {
    *one_of(&(a..=b).collect())
}

fn one_of<'a, T>(v: &'a Vec<T>) -> &'a T {
    use rand::seq::SliceRandom;
    v.choose(&mut rand::thread_rng()).unwrap()
}

fn map_visual_config(vcfg: &VisualConfig) -> (VLayers, String) {
    let (vl, name, range) = vcfg;
    if let Some((a, b)) = range {
        (*vl, name.replace("{}", &format!("{}", between(*a, *b))))
    } else {
        (*vl, name.clone())
    }
}

/////////////////////////////////////////////////////////////////////
// tests

#[test]
fn test_can_create_all_actors() {
    let templates = vec![
        "actor#tank",
        "actor#saw",
        "actor#spear",
        "actor#gunner",
        "enemy#sucker",
        "enemy#worm",
        "enemy#zombi",
    ];

    let generator = ObjectGenerator::new(Path::new("assets/data/"));
    let p = WorldPos::new(0.0, 0.0, 0.0);
    let t = TeamId::new(0);

    for tn in templates {
        let atn = ActorTemplateName(tn.to_string());
        let a = generator.generate_actor(p, t, atn).build();
        assert_eq!(a.team, t); // the real test is loading and deserializing the data files
    }
}
