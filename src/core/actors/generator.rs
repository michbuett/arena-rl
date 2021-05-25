use crate::core::{DisplayStr, WorldPos};

use super::actor::*;
// use super::traits::*;

pub enum ActorType {
    Tank,
    Saw,
    Spear,
    Healer,
}
    
pub fn generate_player_by_type(pos: WorldPos, t: Team, actor_type: ActorType) -> Actor {
    match actor_type {
        ActorType::Tank => generate_player_heavy(pos, t),
        ActorType::Saw => generate_player_saw(pos, t),
        ActorType::Spear => generate_player_spear(pos, t),
        ActorType::Healer => generate_player_healer(pos, t),
    }
}

fn generate_player_heavy(pos: WorldPos, t: Team) -> Actor {
    ActorBuilder::new(generate_name(), pos, t)
        .look(vec!(("body-heavy", 1), ("head-heavy", 1), ("melee-1h", 1), ("shild", 1)))
        .traits(vec!(
            Trait {
                name: DisplayStr::new("Plate mail"),
                effects: vec!(
                    Effect::AttrMod(Attr::Protection, 3),
                ),
                source: TraitSource::IntrinsicProperty,
            },

            Trait {
                name: DisplayStr::new("Towershield"),
                effects: vec!(
                    Effect::AttrMod(Attr::MeleeDefence, 2),
                    Effect::AttrMod(Attr::RangeDefence, 2),
                    Effect::GiveTrait(DisplayStr::new("Block with Shield"), AbilityTarget::OnSelf, Trait {
                        name: DisplayStr::new("Shield raised"),
                        effects: vec!(Effect::MeleeDefence(DisplayStr::new("Block with shield"), 1)),
                        source: TraitSource::Temporary(1),
                    })
                ),
                source: TraitSource::IntrinsicProperty,
            },

            Trait {
                name: DisplayStr::new("Flail"),
                effects: vec!(
                    Effect::MeleeAttack(DisplayStr::new("Swing Flail"), 1, 0, 2),
                ),
                source: TraitSource::IntrinsicProperty,
            }
        ))
        .build()
}

fn generate_player_saw(pos: WorldPos, t: Team) -> Actor {
    ActorBuilder::new(generate_name(), pos, t)
        .look(vec!(("body-heavy", 1), ("head-heavy", 2), ("melee-2h", 1)))
        .traits(vec!(
            Trait {
                name: DisplayStr::new("Plate Mail"),
                effects: vec!(
                    Effect::AttrMod(Attr::Protection, 3),
                ),
                source: TraitSource::IntrinsicProperty,
            },

            Trait {
                name: DisplayStr::new("Power Saw"),
                effects: vec!(
                    Effect::MeleeAttack(DisplayStr::new("Swing"), 1, 0, 3),
                ),
                source: TraitSource::IntrinsicProperty,
            }
        ))
        .build()
}

fn generate_player_spear(pos: WorldPos, t: Team) -> Actor {
    ActorBuilder::new(generate_name(), pos, t)
        .look(vec!(("body-light", 2), ("head", between(1, 4)), ("melee-2h", 2)))
        .traits(vec!(
            Trait {
                name: DisplayStr::new("Chain mail"),
                effects: vec!(
                    Effect::AttrMod(Attr::Protection, 2),
                ),
                source: TraitSource::IntrinsicProperty,
            },

            Trait {
                name: DisplayStr::new("Spear"),
                effects: vec!(
                    Effect::MeleeAttack(DisplayStr::new("Stab"), 2, 0, 1),
                ),
                source: TraitSource::IntrinsicProperty,
            }
        ))
        .build()
}

fn generate_player_healer(pos: WorldPos, t: Team) -> Actor {
    ActorBuilder::new(generate_name(), pos, t)
        .look(vec!(("body-light", 1), ("head", 5), ("melee-1h", 2)))
        .traits(vec!(
            Trait {
                name: DisplayStr::new("Chain mail"),
                effects: vec!(
                    Effect::AttrMod(Attr::Protection, 2),
                ),
                source: TraitSource::IntrinsicProperty,
            },

            Trait {
                name: DisplayStr::new("Injector"),
                effects: vec!(
                    Effect::MeleeAttack(DisplayStr::new("Stab"), 2, 0, 1),
                ),
                source: TraitSource::IntrinsicProperty,
            }
        ))
        .build()
}

pub fn generate_enemy_easy(pos: WorldPos, t: Team) -> Actor {
    ActorBuilder::new(generate_name(), pos, t)
        .look(vec![("body-light", between(1, 3)), ("head", between(1, 4))])
        .behaviour(AiBehaviour::Default)
        .traits(vec![Trait {
            name: DisplayStr::new("Fragile physiology"),
            effects: vec![Effect::AttrMod(Attr::Wound, -2)],
            source: TraitSource::IntrinsicProperty,
        }])
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
    ]).to_string()
}
