use std::path::Path;

use crate::core::WorldPos;

use super::{
    actor::{Actor, ActorBuilder, ActorType, AiBehaviour, TeamId, Trait},
    TraitStorage, Visual, VisualElements,
    VisualState::Hidden,
    VisualState::Prone,
};

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
    traits: TraitStorage,
}

impl ObjectGenerator {
    pub fn new(path: &Path) -> Self {
        Self {
            traits: TraitStorage::new(path),
        }
    }

    pub fn traits(&self) -> &TraitStorage {
        &self.traits
    }

    fn get_trait(&self, key: &str) -> (String, Trait) {
        let t = self.traits.get(key);
        (key.to_string(), t.clone())
    }

    pub fn generate_actor_by_type(
        &self,
        pos: WorldPos,
        t: TeamId,
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

    pub fn generate_player_by_type(
        &self,
        pos: WorldPos,
        t: TeamId,
        actor_type: ActorType,
    ) -> Actor {
        self.generate_actor_by_type(pos, t, actor_type).build()
    }

    pub fn generate_enemy_by_type(&self, pos: WorldPos, t: TeamId, actor_type: ActorType) -> Actor {
        self.generate_actor_by_type(pos, t, actor_type)
            .behaviour(AiBehaviour::Default)
            .build()
    }

    fn generate_actor_heavy(&self, pos: WorldPos, t: TeamId) -> ActorBuilder {
        ActorBuilder::new(generate_name(), pos, t)
            .visual(Visual::new(
                VisualElements::new()
                    .body("body-heavy_1")
                    .head("head-heavy_1"),
            ))
            .traits(vec![
                self.get_trait("item#Armor_PlateMail"),
                self.get_trait("item#Shield_TowerShield"),
                self.get_trait("item#Weapon_Flail"),
            ])
    }

    fn generate_actor_saw(&self, pos: WorldPos, t: TeamId) -> ActorBuilder {
        ActorBuilder::new(generate_name(), pos, t)
            .visual(Visual::new(
                VisualElements::new()
                    .body("body-heavy_1")
                    .head("head-heavy_2"),
            ))
            .traits(vec![
                self.get_trait("item#Armor_PlateMail"),
                self.get_trait("item#Weapon_PowerSaw"),
            ])
    }

    fn generate_actor_spear(&self, pos: WorldPos, t: TeamId) -> ActorBuilder {
        ActorBuilder::new(generate_name(), pos, t)
            .visual(Visual::new(
                VisualElements::new()
                    .body("body-light_2")
                    .head(format!("head_{}", between(1, 4))),
            ))
            .traits(vec![
                self.get_trait("item#Armor_ChainMail"),
                self.get_trait("item#Weapon_Spear"),
            ])
    }

    fn generate_actor_healer(&self, pos: WorldPos, t: TeamId) -> ActorBuilder {
        ActorBuilder::new(generate_name(), pos, t)
            .visual(Visual::new(
                VisualElements::new().body("body-light_1").head("head_5"),
            ))
            .traits(vec![
                self.get_trait("item#Armor_ChainMail"),
                self.get_trait("item#Weapon_Injector"),
            ])
    }

    fn generate_actor_gunner(&self, pos: WorldPos, t: TeamId) -> ActorBuilder {
        ActorBuilder::new(generate_name(), pos, t)
            .visual(Visual::new(
                VisualElements::new().body("body-light_4").head("head_6"),
            ))
            .traits(vec![
                self.get_trait("item#Armor_ChainMail"),
                self.get_trait("item#Weapon_IonGun"),
            ])
    }

    fn generate_monster_sucker(&self, pos: WorldPos, t: TeamId) -> ActorBuilder {
        ActorBuilder::new(generate_name(), pos, t)
            .visual(
                Visual::new(VisualElements::new().body("monster-sucker_1"))
                    .add_state(Prone, VisualElements::new().body("monster-sucker-prone_1")),
            )
            .traits(vec![
                self.get_trait("intrinsic#Weapon_SharpTeeth"),
                self.get_trait("intrinsic#Trait_Weak"),
                self.get_trait("intrinsic#Trait_Quick"),
                self.get_trait("intrinsic#Trait_Flyer"),
            ])
    }

    fn generate_monster_worm(&self, pos: WorldPos, t: TeamId) -> ActorBuilder {
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
                self.get_trait("intrinsic#Trait_Underground"),
                self.get_trait("intrinsic#Weapon_CrushingJaw"),
            ])
    }

    fn generate_monster_zombi(&self, pos: WorldPos, t: TeamId) -> ActorBuilder {
        ActorBuilder::new(generate_name(), pos, t)
            .visual(Visual::new(
                VisualElements::new()
                    .body(format!("body-zombi_{}", between(1, 2)))
                    .head(format!("head-zombi_{}", between(1, 7))),
            ))
            .traits(vec![
                self.get_trait("intrinsic#Weapon_Claws"),
                self.get_trait("intrinsic#Trait_Slow"),
            ])
    }
}

fn between(a: u16, b: u16) -> u16 {
    *one_of(&(a..=b).collect())
}

fn one_of<'a, T>(v: &'a Vec<T>) -> &'a T {
    use rand::seq::SliceRandom;
    v.choose(&mut rand::thread_rng()).unwrap()
}
