use crate::core::{DisplayStr, WorldPos};
use std::collections::HashMap;

use super::actor::*;

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

pub const VISUAL_BODY: u8 = 0;
pub const VISUAL_HEAD: u8 = 10;
pub const VISUAL_HAND_L: u8 = 20;
pub const VISUAL_HAND_R: u8 = 30;

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
            .look(vec![vbody("body-heavy", 1), vhead("head-heavy", 1)])
            .traits(vec![
                self.get_trait("item#Armor_PlateMail").unwrap(),
                self.get_trait("item#Shield_TowerShield").unwrap(),
                self.get_trait("item#Weapon_Flail").unwrap(),
            ])
    }

    fn generate_actor_saw(&self, pos: WorldPos, t: Team) -> ActorBuilder {
        ActorBuilder::new(generate_name(), pos, t)
            .look(vec![vbody("body-heavy", 1), vhead("head-heavy", 2)])
            .traits(vec![
                self.get_trait("item#Armor_PlateMail").unwrap(),
                self.get_trait("item#Weapon_PowerSaw").unwrap(),
            ])
    }

    fn generate_actor_spear(&self, pos: WorldPos, t: Team) -> ActorBuilder {
        ActorBuilder::new(generate_name(), pos, t)
            .look(vec![vbody("body-light", 2), vhead("head", between(1, 4))])
            .traits(vec![
                self.get_trait("item#Armor_ChainMail").unwrap(),
                self.get_trait("item#Weapon_Spear").unwrap(),
            ])
    }

    fn generate_actor_healer(&self, pos: WorldPos, t: Team) -> ActorBuilder {
        ActorBuilder::new(generate_name(), pos, t)
            .look(vec![vbody("body-light", 1), vhead("head", 5)])
            .traits(vec![
                self.get_trait("item#Armor_ChainMail").unwrap(),
                self.get_trait("item#Weapon_Injector").unwrap(),
            ])
    }

    fn generate_actor_gunner(&self, pos: WorldPos, t: Team) -> ActorBuilder {
        ActorBuilder::new(generate_name(), pos, t)
            .look(vec![vbody("body-light", 4), vhead("head", 6)])
            .traits(vec![
                self.get_trait("item#Armor_ChainMail").unwrap(),
                self.get_trait("item#Weapon_IonGun").unwrap(),
            ])
    }

    fn generate_monster_sucker(&self, pos: WorldPos, t: Team) -> ActorBuilder {
        ActorBuilder::new(generate_name(), pos, t)
            .look(vec![(1, "monster-sucker".to_string())])
            .traits(vec![
                self.get_trait("intrinsic#Weapon_SharpTeeth").unwrap(),
                self.get_trait("intrinsic#Trait_Weak").unwrap(),
                self.get_trait("intrinsic#Trait_Quick").unwrap(),
            ])
    }

    fn generate_monster_worm(&self, pos: WorldPos, t: Team) -> ActorBuilder {
        ActorBuilder::new(generate_name(), pos, t)
            .look(vec![
                (1, "monster-worm".to_string()),
                (2, "monster-worm-dust".to_string()),
            ])
            .traits(vec![self
                .get_trait("intrinsic#Weapon_CrushingJaw")
                .unwrap()])
    }

    fn generate_monster_zombi(&self, pos: WorldPos, t: Team) -> ActorBuilder {
        ActorBuilder::new(generate_name(), pos, t)
            .look(vec![
                vbody("body-zombi", between(1, 2)),
                vhead("head-zombi", between(1, 7)),
            ])
            .traits(vec![
                self.get_trait("intrinsic#Weapon_Claws").unwrap(),
                self.get_trait("intrinsic#Trait_Slow").unwrap(),
            ])
    }
}

fn init_traits() -> HashMap<String, Trait> {
    let mut traits = HashMap::new();
    traits.insert(
        "item#Armor_ChainMail".to_string(),
        Trait {
            name: DisplayStr::new("Chain mail"),
            effects: vec![Effect::AttrMod(Attr::Protection, 1)],
            source: TraitSource::IntrinsicProperty,
            visuals: None,
        },
    );

    traits.insert(
        "item#Armor_PlateMail".to_string(),
        Trait {
            name: DisplayStr::new("Plate Mail"),
            effects: vec![
                Effect::AttrMod(Attr::Protection, 2),
                Effect::AttrMod(Attr::MeleeDefence, -1),
                Effect::AttrMod(Attr::RangeDefence, -1),
            ],
            source: TraitSource::IntrinsicProperty,
            visuals: None,
        },
    );

    traits.insert(
        "item#Weapon_PowerSaw".to_string(),
        Trait {
            name: DisplayStr::new("Power Saw"),
            effects: vec![Effect::MeleeAttack{
                name: DisplayStr::new("Swing saw"),
                distance: 1,
                to_hit: 0,
                to_wound: 2,
                fx: "fx-hit-1".to_string(),
            }],
            source: TraitSource::IntrinsicProperty,
            visuals: Some((VISUAL_HAND_R, "melee-2h_1".to_string())),
        },
    );

    traits.insert(
        "item#Weapon_Injector".to_string(),
        Trait {
            name: DisplayStr::new("Injector"),
            effects: vec![Effect::MeleeAttack{
                name: DisplayStr::new("Stab"),
                distance: 2,
                to_hit: 0,
                to_wound: 0,
                fx: "fx-hit-1".to_string(),
            }],
            source: TraitSource::IntrinsicProperty,
            visuals: Some((VISUAL_HAND_R, "staff_1".to_string())),
        },
    );

    traits.insert(
        "item#Weapon_Spear".to_string(),
        Trait {
            name: DisplayStr::new("Spear"),
            effects: vec![Effect::MeleeAttack {
                name: DisplayStr::new("Stab"),
                distance: 2,
                to_hit: 1,
                to_wound: 0,
                fx: "fx-hit-1".to_string(),
            },
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
            effects: vec![Effect::MeleeAttack{
                name: DisplayStr::new("Swing Flail"),
                distance: 1,
                to_hit: 0,
                to_wound: 1,
                fx: "fx-hit-1".to_string(),
            },
                Effect::GiveTrait(
                    "ability#Parry_Flail".to_string(),
                    Trait {
                        name: DisplayStr::new("Parry (flail)"),
                        effects: vec![Effect::Defence(2, DefenceType::Parry)],
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
                to_hit: 1,
                to_wound: 1,
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
                Effect::AttrMod(Attr::MeleeDefence, 1),
                Effect::AttrMod(Attr::RangeDefence, 1),
                Effect::GiveTrait(
                    "ability#Block_Shield".to_string(),
                    Trait {
                        name: DisplayStr::new("Raise shield"),
                        effects: vec![Effect::Defence(1, DefenceType::Block)],
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

    traits.insert(
        "intrinsic#Weapon_SharpTeeth".to_string(),
        Trait {
            name: DisplayStr::new("Sharp teeth"),
            effects: vec![Effect::MeleeAttack {
                name: DisplayStr::new("Bite"),
                distance: 1,
                to_hit: 0,
                to_wound: 0,
                fx: "fx-hit-3".to_string(),
            }],
            source: TraitSource::IntrinsicProperty,
            visuals: None,
        },
    );

    traits.insert(
        "intrinsic#Weapon_CrushingJaw".to_string(),
        Trait {
            name: DisplayStr::new("Crushing jaw"),
            effects: vec![Effect::MeleeAttack {
                name: DisplayStr::new("Bite"),
                distance: 1,
                to_hit: 0,
                to_wound: 1,
                fx: "fx-hit-3".to_string(),
            }],
            source: TraitSource::IntrinsicProperty,
            visuals: None,
        },
    );

    traits.insert(
        "intrinsic#Weapon_Claws".to_string(),
        Trait {
            name: DisplayStr::new("Rending claws"),
            effects: vec![Effect::MeleeAttack {
                name: DisplayStr::new("Rend"),
                distance: 1,
                to_hit: 1,
                to_wound: 0,
                fx: "fx-hit-2".to_string(),
            }],
            source: TraitSource::IntrinsicProperty,
            visuals: Some((VISUAL_HAND_R, "claws_1".to_string())),
        },
    );

    traits.insert(
        "intrinsic#Trait_Weak".to_string(),
        Trait {
            name: DisplayStr::new("Weak"),
            effects: vec![Effect::AttrMod(Attr::Physical, -1)],
            source: TraitSource::IntrinsicProperty,
            visuals: None,
        },
    );

    traits.insert(
        "intrinsic#Trait_Quick".to_string(),
        Trait {
            name: DisplayStr::new("Quick"),
            effects: vec![Effect::AttrMod(Attr::Movement, 1)],
            source: TraitSource::IntrinsicProperty,
            visuals: None,
        },
    );

    traits.insert(
        "intrinsic#Trait_Slow".to_string(),
        Trait {
            name: DisplayStr::new("Slow"),
            effects: vec![Effect::AttrMod(Attr::Movement, -1)],
            source: TraitSource::IntrinsicProperty,
            visuals: None,
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

fn between(a: u16, b: u16) -> u16 {
    *one_of(&(a..=b).collect())
}

fn one_of<'a, T>(v: &'a Vec<T>) -> &'a T {
    use rand::seq::SliceRandom;
    v.choose(&mut rand::thread_rng()).unwrap()
}
