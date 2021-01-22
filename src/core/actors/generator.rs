use crate::core::{DisplayStr, WorldPos};

use super::actor::*;
// use super::traits::*;

pub fn generate_player(pos: WorldPos, t: Team) -> Actor {
    extern crate rand;
    use rand::prelude::*;
    let range = rand::distributions::Uniform::from(1..=100);
    let mut rng = rand::thread_rng();

    ActorBuilder::new(generate_name(), pos, t)
        .look(vec![("player", rng.sample(range))])
        .traits(vec!(
            Trait {
                name: DisplayStr("Defensiv combat training"),
                effects: vec!(Effect::GiveTrait(DisplayStr("Defensiv stance"), AbilityTarget::OnSelf, Trait {
                // effects: vec!(Effect::Ability(DisplayStr("Defensiv stance"), AbilityTarget::OnSelf, Ability::GiveTrait(Trait {
                    name: DisplayStr("Defensiv stance"),
                    effects: vec!(Effect::AttrMod(Attr::Defence, 1)),
                    source: TraitSource::Temporary(1),
                })),
                source: TraitSource::IntrinsicProperty,
            }
        ))
        .build()
}

pub fn generate_enemy_easy(pos: WorldPos, t: Team) -> Actor {
    ActorBuilder::new(generate_name(), pos, t)
        .look(vec![("tile", 3965), ("tile", *one_of(&vec![5747, 5748, 5749]))],)
        .behaviour(AiBehaviour::Default)
        .traits(vec![Trait {
            name: DisplayStr("Fragile physiology"),
            effects: vec![Effect::AttrMod(Attr::Wound, -2)],
            source: TraitSource::IntrinsicProperty,
        }])
        .build()
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
