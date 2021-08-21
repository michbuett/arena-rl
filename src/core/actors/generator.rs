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

pub fn generate_enemy_easy(pos: WorldPos, t: Team) -> Actor {
    ActorBuilder::new(generate_name(), pos, t)
        .look(vec![
            ("body-light", between(1, 3)),
            ("head", between(1, 4)),
            ("claw-L", 1),
            ("claw-R", 1),
        ])
        .behaviour(AiBehaviour::Default)
        .traits(vec![(
            "intrinsic#Fragile".to_string(),
            Trait {
                name: DisplayStr::new("Fragile physiology"),
                effects: vec![Effect::AttrMod(Attr::Wound, -2)],
                source: TraitSource::IntrinsicProperty,
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
                ("body-heavy", 1),
                ("head-heavy", 1),
                ("shild", 1),
                ("melee-1h", 1),
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
            .look(vec![("body-heavy", 1), ("head-heavy", 2), ("melee-2h", 1)])
            .traits(vec![
                self.get_trait("item#Armor_PlateMail").unwrap(),
                self.get_trait("item#Weapon_PowerSaw").unwrap(),
            ])
            .build()
    }

    fn generate_player_spear(&self, pos: WorldPos, t: Team) -> Actor {
        ActorBuilder::new(generate_name(), pos, t)
            .look(vec![
                ("body-light", 2),
                ("head", between(1, 4)),
                ("melee-2h", 2),
            ])
            .traits(vec![
                self.get_trait("item#Armor_ChainMail").unwrap(),
                self.get_trait("item#Weapon_Spear").unwrap(),
            ])
            .build()
    }

    fn generate_player_healer(&self, pos: WorldPos, t: Team) -> Actor {
        ActorBuilder::new(generate_name(), pos, t)
            .look(vec![("body-light", 1), ("head", 5), ("staff", 1)])
            .traits(vec![
                self.get_trait("item#Armor_ChainMail").unwrap(),
                self.get_trait("item#Weapon_Injector").unwrap(),
            ])
            .build()
    }

    fn generate_player_gunner(&self, pos: WorldPos, t: Team) -> Actor {
        ActorBuilder::new(generate_name(), pos, t)
            .look(vec![("body-light", 4), ("head", 6), ("gun-2h", 1)])
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
        },
    );

    traits.insert(
        "item#Armor_PlateMail".to_string(),
        Trait {
            name: DisplayStr::new("Plate Mail"),
            effects: vec![Effect::AttrMod(Attr::Protection, 5)],
            source: TraitSource::IntrinsicProperty,
        },
    );

    traits.insert(
        "item#Weapon_PowerSaw".to_string(),
        Trait {
            name: DisplayStr::new("Power Saw"),
            effects: vec![Effect::MeleeAttack(DisplayStr::new("Swing"), 1, 2, 5)],
            source: TraitSource::IntrinsicProperty,
        },
    );

    traits.insert(
        "item#Weapon_Injector".to_string(),
        Trait {
            name: DisplayStr::new("Injector"),
            effects: vec![Effect::MeleeAttack(DisplayStr::new("Stab"), 2, 3, 3)],
            source: TraitSource::IntrinsicProperty,
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
                    },
                    AbilityTarget::OnSelf,
                ),
            ],
            source: TraitSource::IntrinsicProperty,
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
                    },
                    AbilityTarget::OnSelf,
                ),
            ],
            source: TraitSource::IntrinsicProperty,
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
                    },
                    AbilityTarget::OnSelf,
                ),
            ],
            source: TraitSource::IntrinsicProperty,
        },
    );

    traits
}
