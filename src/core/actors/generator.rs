use crate::core::{DisplayStr, WorldPos};
use std::collections::HashMap;

use super::actor::*;

pub enum ActorType {
    Tank,
    Saw,
    Spear,
    Healer,
    Gunner,
}

pub const VISUAL_BODY: u8 = 0;
pub const VISUAL_HEAD: u8 = 10;
pub const VISUAL_HAND_L: u8 = 20;
pub const VISUAL_HAND_R: u8 = 30;

pub fn generate_enemy_easy(pos: WorldPos, t: Team) -> Actor {
    ActorBuilder::new(generate_name(), pos, t)
        .look(vec![
            vbody("body-light", between(1, 3)),
            vhead("head", between(1, 4)),
            // ("claws", 1),
        ])
        .behaviour(AiBehaviour::Default)
        .traits(vec![(
            "intrinsic#Fragile".to_string(),
            Trait {
                name: DisplayStr::new("Fragile physiology"),
                effects: vec![Effect::AttrMod(Attr::Wound, -2)],
                source: TraitSource::IntrinsicProperty,
                visuals: None,
            },
        ), (
            "intrinsic#Claws".to_string(),
            Trait {
                name: DisplayStr::new("Rending claws"),
                effects: vec![Effect::MeleeAttack(DisplayStr::new("Rend"), 1, 3, 3)],
                source: TraitSource::IntrinsicProperty,
                visuals: Some((VISUAL_HAND_R, "claws_1".to_string())),
            },
        )])
        .build()
}

fn between(a: u16, b: u16) -> u16 {
    *one_of(&(a..=b).collect())
}

fn one_of<'a, T>(v: &'a Vec<T>) -> &'a T {
    use rand::seq::SliceRandom;
    v.choose(&mut rand::thread_rng()).unwrap()
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
    pub fn new() -> Self {
        Self {
            traits: init_traits(),
        }
    }

    fn get_trait(&self, key: &str) -> Option<(String, Trait)> {
        self.traits
            .get_key_value(key)
            .map(|(k, t)| (k.clone(), t.clone()))
        // self.traits.get_key_value(key).cloned()
    }

    pub fn generate_player_by_type(&self, pos: WorldPos, t: Team, actor_type: ActorType) -> Actor {
        match actor_type {
            ActorType::Tank => self.generate_player_heavy(pos, t),
            ActorType::Saw => self.generate_player_saw(pos, t),
            ActorType::Spear => self.generate_player_spear(pos, t),
            ActorType::Healer => self.generate_player_healer(pos, t),
            ActorType::Gunner => self.generate_player_gunner(pos, t),
        }
    }

    fn generate_player_heavy(&self, pos: WorldPos, t: Team) -> Actor {
        ActorBuilder::new(generate_name(), pos, t)
            .look(vec![
                vbody("body-heavy", 1),
                vhead("head-heavy", 1),
            ])
            .traits(vec![
                self.get_trait("item#Armor_PlateMail").unwrap(),
                self.get_trait("item#Shield_TowerShield").unwrap(),
                self.get_trait("item#Weapon_Flail").unwrap(),
            ])
            .build()
    }

    fn generate_player_saw(&self, pos: WorldPos, t: Team) -> Actor {
        ActorBuilder::new(generate_name(), pos, t)
            .look(vec![vbody("body-heavy", 1), vhead("head-heavy", 2)])
            .traits(vec![
                self.get_trait("item#Armor_PlateMail").unwrap(),
                self.get_trait("item#Weapon_PowerSaw").unwrap(),
            ])
            .build()
    }

    fn generate_player_spear(&self, pos: WorldPos, t: Team) -> Actor {
        ActorBuilder::new(generate_name(), pos, t)
            .look(vec![vbody("body-light", 2), vhead("head", between(1, 4)),])
            .traits(vec![
                self.get_trait("item#Armor_ChainMail").unwrap(),
                self.get_trait("item#Weapon_Spear").unwrap(),
            ])
            .build()
    }

    fn generate_player_healer(&self, pos: WorldPos, t: Team) -> Actor {
        ActorBuilder::new(generate_name(), pos, t)
            .look(vec![vbody("body-light", 1), vhead("head", 5)])
            .traits(vec![
                self.get_trait("item#Armor_ChainMail").unwrap(),
                self.get_trait("item#Weapon_Injector").unwrap(),
            ])
            .build()
    }

    fn generate_player_gunner(&self, pos: WorldPos, t: Team) -> Actor {
        ActorBuilder::new(generate_name(), pos, t)
            .look(vec![vbody("body-light", 4), vhead("head", 6), ])
            .traits(vec![
                self.get_trait("item#Armor_ChainMail").unwrap(),
                self.get_trait("item#Weapon_IonGun").unwrap(),
            ])
            .build()
    }
}

fn init_traits() -> HashMap<String, Trait> {
    let mut traits = HashMap::new();
    traits.insert(
        "item#Armor_ChainMail".to_string(),
        Trait {
            name: DisplayStr::new("Chain mail"),
            effects: vec![Effect::AttrMod(Attr::Protection, 4)],
            source: TraitSource::IntrinsicProperty,
            visuals: None,
        },
    );

    traits.insert(
        "item#Armor_PlateMail".to_string(),
        Trait {
            name: DisplayStr::new("Plate Mail"),
            effects: vec![Effect::AttrMod(Attr::Protection, 5)],
            source: TraitSource::IntrinsicProperty,
            visuals: None,
        },
    );

    traits.insert(
        "item#Weapon_PowerSaw".to_string(),
        Trait {
            name: DisplayStr::new("Power Saw"),
            effects: vec![Effect::MeleeAttack(DisplayStr::new("Swing"), 1, 2, 5)],
            source: TraitSource::IntrinsicProperty,
            visuals: Some((VISUAL_HAND_R, "melee-2h_1".to_string())),
        },
    );

    traits.insert(
        "item#Weapon_Injector".to_string(),
        Trait {
            name: DisplayStr::new("Injector"),
            effects: vec![Effect::MeleeAttack(DisplayStr::new("Stab"), 2, 3, 3)],
            source: TraitSource::IntrinsicProperty,
            visuals: Some((VISUAL_HAND_R, "staff_1".to_string())),
        },
    );

    traits.insert(
        "item#Weapon_Spear".to_string(),
        Trait {
            name: DisplayStr::new("Spear"),
            effects: vec![
                Effect::MeleeAttack(DisplayStr::new("Stab"), 2, 4, 3),
                Effect::GiveTrait(
                    "ability#Parry_Spear".to_string(),
                    Trait {
                        name: DisplayStr::new("Parry (spear)"),
                        effects: vec![Effect::Defence(3, DefenceType::Parry)],
                        source: TraitSource::Temporary(1),
                        visuals: None,
                    },
                    AbilityTarget::OnSelf,
                ),
            ],
            source: TraitSource::IntrinsicProperty,
            visuals: Some((VISUAL_HAND_R, "melee-2h_2".to_string())),
        },
    );

    traits.insert(
        "item#Weapon_Flail".to_string(),
        Trait {
            name: DisplayStr::new("Flail"),
            effects: vec![
                Effect::MeleeAttack(DisplayStr::new("Swing Flail"), 1, 3, 4),
                Effect::GiveTrait(
                    "ability#Parry_Flail".to_string(),
                    Trait {
                        name: DisplayStr::new("Parry (flail)"),
                        effects: vec![Effect::Defence(2, DefenceType::Parry,)],
                        source: TraitSource::Temporary(1),
                        visuals: None,
                    },
                    AbilityTarget::OnSelf,
                ),
            ],
            source: TraitSource::IntrinsicProperty,
            visuals: Some((VISUAL_HAND_R, "melee-1h_1".to_string())),
        },
    );

    traits.insert(
        "item#Weapon_IonGun".to_string(),
        Trait {
            name: DisplayStr::new("Ion Gun"),
            effects: vec![Effect::RangeAttack {
                name: DisplayStr::new("Shoot Ion Gun"),
                distance: (2, 8),
                to_hit: 4,
                to_wound: 4,
                fx: "fx-projectile-2".to_string(),
            }],
            source: TraitSource::IntrinsicProperty,
            visuals: Some((VISUAL_HAND_L, "gun-2h_1".to_string())),
        },
    );

    traits.insert(
        "item#Shield_TowerShield".to_string(),
        Trait {
            name: DisplayStr::new("Towershield"),
            effects: vec![
                Effect::AttrMod(Attr::MeleeDefence, 2),
                Effect::AttrMod(Attr::RangeDefence, 2),
                Effect::GiveTrait(
                    "ability#Block_Shield".to_string(),
                    Trait {
                        name: DisplayStr::new("Raise shield"),
                        effects: vec![Effect::Defence(1, DefenceType::Block,)],
                        source: TraitSource::Temporary(1),
                        visuals: None,
                    },
                    AbilityTarget::OnSelf,
                ),
            ],
            source: TraitSource::IntrinsicProperty,
            visuals: Some((VISUAL_HAND_L, "shild_1".to_string())),
        },
    );

    traits
}

fn vbody(name: impl std::fmt::Display, variant: u16) -> (u8, String) {
    (VISUAL_BODY, format!("{}_{}", name, variant))
}

fn vhead(name: impl std::fmt::Display, variant: u16) -> (u8, String) {
    (VISUAL_HEAD, format!("{}_{}", name, variant))
}
