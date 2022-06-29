use std::collections::HashMap;
use std::fs::File;
use std::iter::FromIterator;
use std::path::Path;

use ron::de::from_reader;

use crate::core::WorldPos;

use super::{
    actor::{Actor, ActorBuilder, AiBehaviour, Team, Trait},
    Visual, VisualElements,
    VisualState::Hidden,
    VisualState::Prone,
};

pub enum ActorType {
    Tank,
    Saw,
    Spear,
    Healer,
    Gunner,
    MonsterSucker,
    MonsterWorm,
    MonsterZombi,
}

fn generate_name() -> String {
    one_of(&vec![
        "Avrak The Gruesome",
        "Bhak Toe Burster",
        "Bhog Horror Dagger",
        "Brumvur The Gargantuan",
        "Cukgilug",
        "Dhukk The Brutal",
        "Drurzod The Rotten",
        "Duvrull Iron Splitter",
        "Eagungad",
        "Ghakk The Fearless",
        "Gruvukk Anger Dagger",
        "Guvrok Beast Saber",
        "Hrolkug",
        "Jag Skull Queller",
        "Jal The Merciless",
        "Klughig",
        "Kogan",
        "Komarod",
        "Lugrub",
        "Magdud",
        "Meakgu",
        "Ohulhug",
        "Oogorim",
        "Rhuruk The Wretched",
        "Rob Muscle Splitter",
        "Robruk The Feisty",
        "Shortakk The Crazy",
        "Shovog The Fierce",
        "Taugh",
        "Wegub",
        "Xagok",
        "Xoruk",
        "Xuag",
        "Yegoth",
        "Yokgu",
        "Zog",
        "Zogugbu",
        "Zubzor Tooth Clobberer",
        "Zug The Ugly",
        "Zuvrog Sorrow Gouger",
    ])
    .to_string()
}

#[derive(Default)]
pub struct ObjectGenerator {
    traits: HashMap<String, Trait>,
}

impl ObjectGenerator {
    pub fn new(path: &Path) -> Self {
        let traits = load_traits_from_file(path);

        Self {
            traits: HashMap::from_iter(traits),
        }
    }

    fn get_trait(&self, key: &str) -> Option<(String, Trait)> {
        self.traits
            .get_key_value(key)
            .map(|(k, t)| (k.clone(), t.clone()))
    }

    pub fn generate_actor_by_type(
        &self,
        pos: WorldPos,
        t: Team,
        actor_type: ActorType,
    ) -> ActorBuilder {
        match actor_type {
            ActorType::Tank => self.generate_actor_heavy(pos, t),
            ActorType::Saw => self.generate_actor_saw(pos, t),
            ActorType::Spear => self.generate_actor_spear(pos, t),
            ActorType::Healer => self.generate_actor_healer(pos, t),
            ActorType::Gunner => self.generate_actor_gunner(pos, t),
            ActorType::MonsterSucker => self.generate_monster_sucker(pos, t),
            ActorType::MonsterWorm => self.generate_monster_worm(pos, t),
            ActorType::MonsterZombi => self.generate_monster_zombi(pos, t),
        }
    }

    pub fn generate_player_by_type(&self, pos: WorldPos, t: Team, actor_type: ActorType) -> Actor {
        self.generate_actor_by_type(pos, t, actor_type).build()
    }

    pub fn generate_enemy_by_type(&self, pos: WorldPos, t: Team, actor_type: ActorType) -> Actor {
        self.generate_actor_by_type(pos, t, actor_type)
            .behaviour(AiBehaviour::Default)
            .build()
    }

    pub fn generate_enemy_wave(&self, wave: u64) -> Vec<(u8, ActorType)> {
        match wave {
            0 => vec![(1, ActorType::MonsterSucker), (4, ActorType::MonsterSucker)],

            1 => vec![
                (0, ActorType::MonsterSucker),
                (3, ActorType::MonsterSucker),
                (1, ActorType::MonsterSucker),
                (4, ActorType::MonsterSucker),
            ],

            2 => vec![(1, ActorType::MonsterWorm), (4, ActorType::MonsterWorm)],

            3 => vec![(1, ActorType::MonsterZombi), (4, ActorType::MonsterZombi)],

            4 => vec![
                (0, ActorType::MonsterZombi),
                (1, ActorType::MonsterZombi),
                (2, ActorType::MonsterZombi),
                (3, ActorType::MonsterZombi),
                (4, ActorType::MonsterZombi),
                (5, ActorType::MonsterZombi),
            ],

            5 => vec![
                (0, ActorType::MonsterZombi),
                (1, ActorType::MonsterZombi),
                (2, ActorType::MonsterZombi),
                (3, ActorType::MonsterZombi),
                (4, ActorType::MonsterZombi),
                (5, ActorType::MonsterZombi),
                (6, ActorType::MonsterZombi),
                (7, ActorType::MonsterZombi),
                (8, ActorType::MonsterZombi),
                (9, ActorType::MonsterZombi),
                (10, ActorType::Healer),
                (11, ActorType::MonsterZombi),
            ],
            _ => vec![],
        }
    }

    fn generate_actor_heavy(&self, pos: WorldPos, t: Team) -> ActorBuilder {
        ActorBuilder::new(generate_name(), pos, t)
            .visual(Visual::new(
                VisualElements::new()
                    .body("body-heavy_1")
                    .head("head-heavy_1"),
            ))
            .traits(vec![
                self.get_trait("item#Armor_PlateMail").unwrap(),
                self.get_trait("item#Shield_TowerShield").unwrap(),
                self.get_trait("item#Weapon_Flail").unwrap(),
            ])
    }

    fn generate_actor_saw(&self, pos: WorldPos, t: Team) ->
        ActorBuilder { ActorBuilder::new(generate_name(), pos, t)
        .visual(Visual::new( VisualElements::new()
        .body("body-heavy_1") .head("head-heavy_2"), )) .traits(vec![
        self.get_trait("item#Armor_PlateMail").unwrap(),
        self.get_trait("item#Weapon_PowerSaw").unwrap(), ]) }

    fn generate_actor_spear(&self, pos: WorldPos, t: Team) -> ActorBuilder {
        ActorBuilder::new(generate_name(), pos, t)
            .visual(Visual::new(
                VisualElements::new()
                    .body("body-light_2")
                    .head(format!("head_{}", between(1, 4))),
            ))
            .traits(vec![
                self.get_trait("item#Armor_ChainMail").unwrap(),
                self.get_trait("item#Weapon_Spear").unwrap(),
            ])
    }

    fn generate_actor_healer(&self, pos: WorldPos, t: Team) -> ActorBuilder {
        ActorBuilder::new(generate_name(), pos, t)
            .visual(Visual::new(
                VisualElements::new()
                    .body("body-light_1")
                    .head("head_5")
            ))
            .traits(vec![
                self.get_trait("item#Armor_ChainMail").unwrap(),
                self.get_trait("item#Weapon_Injector").unwrap(),
            ])
    }

    fn generate_actor_gunner(&self, pos: WorldPos, t: Team) -> ActorBuilder {
        ActorBuilder::new(generate_name(), pos, t)
            .visual(Visual::new(
                VisualElements::new().body("body-light_4").head("head_6"),
            ))
            .traits(vec![
                self.get_trait("item#Armor_ChainMail").unwrap(),
                self.get_trait("item#Weapon_IonGun").unwrap(),
            ])
    }

    fn generate_monster_sucker(&self, pos: WorldPos, t: Team) -> ActorBuilder {
        ActorBuilder::new(generate_name(), pos, t)
            .visual(
                Visual::new(VisualElements::new().body("monster-sucker_1"))
                    .add_state(Prone, VisualElements::new().body("monster-sucker-prone_1")),
            )
            .traits(vec![
                self.get_trait("intrinsic#Weapon_SharpTeeth").unwrap(),
                self.get_trait("intrinsic#Trait_Weak").unwrap(),
                self.get_trait("intrinsic#Trait_Quick").unwrap(),
                self.get_trait("intrinsic#Trait_Flyer").unwrap(),
            ])
    }

    fn generate_monster_worm(&self, pos: WorldPos, t: Team) -> ActorBuilder {
        ActorBuilder::new(generate_name(), pos, t)
            .visual(
                Visual::new(
                    VisualElements::new()
                        .body("monster-worm_1")
                        .head("monster-worm-dust_1"),
                )
                .add_state(Hidden, VisualElements::new().body("monster-worm-hidden_1")),
            )
            .traits(vec![
                self.get_trait("intrinsic#Trait_Underground").unwrap(),
                self.get_trait("intrinsic#Weapon_CrushingJaw").unwrap(),
            ])
    }

    fn generate_monster_zombi(&self, pos: WorldPos, t: Team) -> ActorBuilder {
        ActorBuilder::new(generate_name(), pos, t)
            .visual(Visual::new(
                VisualElements::new()
                    .body(format!("body-zombi_{}", between(1, 2)))
                    .head(format!("head-zombi_{}", between(1, 7))),
            ))
            .traits(vec![
                self.get_trait("intrinsic#Weapon_Claws").unwrap(),
                self.get_trait("intrinsic#Trait_Slow").unwrap(),
            ])
    }
}

// fn vbody(name: impl std::fmt::Display, variant: u16) -> (u8, String) {
//     (0, format!("{}_{}", name, variant))
// }

// fn vhead(name: impl std::fmt::Display, variant: u16) -> (u8, String) {
//     (10, format!("{}_{}", name, variant))
// }

fn between(a: u16, b: u16) -> u16 {
    *one_of(&(a..=b).collect())
}

// fn vname(name: impl std::fmt::Display, variant_from: u16, variant_to: u16) -> String {
//     let variant = *one_of(&(variant_from..=variant_to).collect());
//     format!("{}_{}", name, variant)
// }

fn one_of<'a, T>(v: &'a Vec<T>) -> &'a T {
    use rand::seq::SliceRandom;
    v.choose(&mut rand::thread_rng()).unwrap()
}

fn load_traits_from_file(path: &Path) -> Vec<(String, Trait)> {
    let p = path.join("traits.ron");
    let f = match File::open(p) {
        Ok(result) => result,
        Err(e) => {
            panic!("Error opening proto sprite config file: {:?}", e);
        }
    };

    match from_reader(f) {
        Ok(result) => result,
        Err(e) => {
            panic!("Error parsing proto sprite config: {:?}", e);
        }
    }
}
