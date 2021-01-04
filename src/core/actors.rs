use std::cmp::max;
use super::dice::*;
use crate::core::{Action, DisplayStr, WorldPos, Tile};

// const STAT_AVERAGE: u8 = 4;

/// Anything that exists in the world
#[derive(Debug, Clone)]
pub enum GameObject {
    Actor(Actor),
    Item(WorldPos, Item),
}

#[derive(Debug, Clone)]
pub struct Item {
    pub name: String,
    pub look: Look,
}

///  0 => -3 => None
///  1 => -2 => Puny
///  2 => -1 => Low — rusty
///  3 =>  0 => Average
///  4 =>  1 => Good — trained
///  5 =>  2 => Elite (only the best have elite stats)
///  6 =>  3 => Exceptional (once per generagion; the best of the best)
///  7 =>  4 => Legendary (once per era)
///  8 =>  5 => Supernatural
///  9 =>  6 => ? (Ultra, Marvelous)
/// 10 =>  7 => Godlike (unlimited power)
// #[derive(Debug, Clone)]
// pub struct Attributes {
//     /// cognitiv abilities, intelligence, wisdom
//     mind: i8,
//     /// agility, dextery, speed
//     speed: i8,
//     /// strength, endurance
//     power: i8,
// }

// impl Attributes {
//     pub fn new(mind: i8, speed: i8, power: i8) -> Self {
//         Self { mind, speed, power }
//     }
// }

#[derive(Debug, Clone)]
pub enum AiBehaviour {
    Default,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Team(pub &'static str, pub u8, pub bool);

pub struct ActorBuilder {
    // attributes: Attributes,
    behaviour: Option<AiBehaviour>,
    pos: WorldPos,
    team: Team,
    armor: Armor,
    name: String,
    traits: Vec<Trait>,
}

impl ActorBuilder {
    pub fn new(pos: WorldPos, team: Team) -> Self {
    // pub fn new(pos: WorldPos, attributes: Attributes, team: Team) -> Self {
        Self {
            pos,
            // attributes,
            team,
            name: generate_name(),
            behaviour: None,
            armor: Armor {
                // default to "no armor"-armor
                look: vec![],
                protection: 0,
            },
            traits: Vec::new(),
        }
    }

    pub fn build(self) -> Actor {
        // let a = self.attributes;
        // let energy = a.speed + a.power;

        Actor {
            name: self.name,
            active: false,
            pos: self.pos,
            // energy: (energy, energy),
            // attributes: (a.clone(), a.clone()),
            pain: 0,
            wounds: 0,
            // wounds: Vec::new(),
            effects: Vec::new(),
            traits: Vec::new(),
            pending_action: None,
            behaviour: self.behaviour,
            team: self.team,
            armor: self.armor,
            turn: 0,
            quick_action_available: true,
            // activations: roll_activations(3),
            attacks: vec![
                AttackOption {
                    name: DisplayStr("Swing"),
                    dice: 3,
                    distance: (0.0, 1.42),
                    damage: 4,
                    costs: 3,
                },
            ],
            defences: vec![DefenceOption {
                name: DisplayStr("Dodge"),
                dice: 3,
                required_ability: None,
                costs: 2,
            }],
            wields: Weapon {
                name: DisplayStr("Sword"),
                tags: Vec::new(),
                to_hit: Dice::new(4),
            },
        }.set_traits(self.traits)
    }

    pub fn behaviour(self, b: AiBehaviour) -> Self {
        Self {
            behaviour: Some(b),
            ..self
        }
    }

    pub fn armor(self, armor: Armor) -> Self {
        Self { armor, ..self }
    }

    pub fn traits(self, traits: Vec<Trait>) -> Self {
        Self { traits, ..self }
    }
}

// #[derive(Debug, Clone)]
// pub struct Look (Vec<&'static str>);
pub type Look = Vec<(&'static str, u16)>;

#[derive(Debug, Clone)]
pub struct Armor {
    pub look: Look,
    pub protection: u8,
}

#[derive(Debug, Clone)]
pub struct Actor {
    pub name: String,
    /// base | effectiv
    // attributes: (Attributes, Attributes),
    /// (current, max)
    // energy: (i8, i8),
    pain: u8,
    wounds: u8,
    // wounds: Vec<Wound>,
    effects: Vec<Effect>,
    traits: Vec<Trait>,
    wields: Weapon,
    pub attacks: Vec<AttackOption>,
    defences: Vec<DefenceOption>,
    armor: Armor,
    // activations: Vec<Activation>,
    quick_action_available: bool,

    pub turn: u64,
    pub active: bool,
    pub team: Team,
    pub pos: WorldPos,
    pub pending_action: Option<(Action, u8)>,
    // pub pending_action: Option<Box<(Action, u8)>>,
    pub behaviour: Option<AiBehaviour>,
}

impl Actor {
    pub fn move_to(self, to: Tile) -> Self {
        assert!(self.can_move(), "Actor cannot move: {:?}", self);
        
        Self {
            pos: to.to_world_pos(),
            quick_action_available: false,
            // activations: use_activation(D6::new(1), self.activations),
            ..self
        }
    }

    pub fn can_move(&self) -> bool {
        self.quick_action_available
    }

    pub fn move_distance(&self) -> u8 {
        2
    }

    pub fn is_pc(&self) -> bool {
        self.behaviour.is_none()
    }

    pub fn activate(self) -> Self {
        Self {
            active: true,
            ..self
        }
    }

    pub fn deactivate(self) -> Self {
        Self {
            active: false,
            ..self
        }
    }

    pub fn prepare(self, action: (Action, u8)) -> Actor {
        Actor {
            pending_action: Some(action),
            // pending_action: Some(Box::new(action)),
            quick_action_available: false,
            active: false,
            ..self
        }
    }

    pub fn start_next_turn(self) -> (Actor, Option<(Action, u8)>) {
    // pub fn start_next_turn(self) -> (Actor, Option<Box<(Action, u8)>>) {
        let pending_action = self.pending_action;
        let next_turn_actor = Self {
            pending_action: None,
            quick_action_available: true,
            ..self
        };
        
        (next_turn_actor, pending_action)
    }
    
    // pub fn next_turn(self, turn: u64) -> Condition {
    //     let (e_current, e_max) = self.energy;
    //     let mut e_new = min(e_max, e_current + e_max);
    //     let power = self.attributes().power;
    //     let num_wounds = self.num_wounds();
    //     let mut effects = self.effects;

    //     if effects.contains(&Effect::Dying()) {
    //         let tries = save_roll(num_wounds) as i8;

    //         if tries <= power {
    //             // actor has recovered from its dying state
    //             effects = effects
    //                 .drain(..)
    //                 .filter(|e| *e != Effect::Dying())
    //                 .collect();
    //         } else {
    //             e_new = min(e_new, power + e_new - tries);

    //             if e_new < 0 {
    //                 // it's over now...
    //                 return Condition::Dead(
    //                     self.pos,
    //                     Item {
    //                         name: format!("Corpse of {}", self.name),
    //                         look: vec![("corpses", 1)],
    //                     },
    //                 );
    //             }
    //         }
    //     }

    //     // TODO: calc number of activations from current stats
    //     // let max_activations: usize = 3;

    //     Condition::Alive(Actor {
    //         turn,
    //         effects,
    //         energy: (e_new, e_max),
    //         quick_action_available: true,
    //         // activations: roll_activations(max_activations.checked_sub(num_wounds).unwrap_or(0)),
    //         ..self
    //     })
    // }

    // pub fn done(self, costs: u8) -> Self {
    //     Actor {
    //         active: false,
    //         energy: (0, self.energy.1),
    //         // energy: (self.energy.0 - costs as i8, self.energy.1),
    //         ..self
    //     }
    // }

    // pub fn energy(&self) -> i8 {
    //     self.energy.0
    // }

    // fn attributes(&self) -> &Attributes {
    //     &self.attributes.1
    // }

    pub fn attacks(&self, target: &Actor) -> Vec<AttackOption> {
        let distance = WorldPos::distance(&self.pos, &target.pos);
        self.attacks
            .iter()
            .filter(|o| o.distance.0 <= distance && distance <= o.distance.1)
            .cloned()
            // .map(|o| o.into_attack(&self.wields))
            .collect()
    }

    // pub fn defences(&self, _: &Attack) -> Vec<Defence> {
    //     // TODO: consider attack and remaining energy
    //     self.defences
    //         .iter()
    //         .cloned()
    //         .map(|o| o.into_defence())
    //         .collect()
    // }

    // pub fn has_effect(&self, e: &Effect) -> bool {
    //     self.effects.contains(e)
    // }

    fn set_traits(self, traits: Vec<Trait>) -> Self {
        let effects = traits.iter().flat_map(|t| t.effects.to_vec()).collect();

        Self { traits, effects, ..self }
    }

    pub fn is_dying(&self) -> bool {
        self.effects.contains(&Effect::Dying())
    }

    fn wound(self, w: Wound) -> Condition {
        let wounds_modifer = self.effects.iter().map(|e| {
            match e {
                Effect::AttributeModifier(Attribute::Wound, modifier) => *modifier,
                _ => 0,
            }
        }).sum::<i8>();
        let default_wounds_num = 3;
        let min_wounds_num = 1;
        let max_wounds = max(min_wounds_num, default_wounds_num + wounds_modifer) as u8;
        let wounds = self.wounds + w.wound;
        
        if wounds < max_wounds {
            Condition::Alive(Self{ wounds, ..self })
        } else {
            Condition::Dead(
                self.pos,
                Item {
                    name: format!("Corpse of {}", self.name),
                    look: vec![("corpses", 1)],
                },
            )
        }
    }
        
    pub fn num_wounds(&self) -> usize {
        self.wounds as usize
    }

    pub fn look(&self) -> &Look {
        &self.armor.look
    }

    pub fn defence(&self) -> i8 {
        3
    }

    pub fn protection(&self) -> i8 {
        self.armor.protection as i8
    }

    // pub fn activations(&self) -> std::slice::Iter<Activation> {
    //     self.activations.iter()
    // }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord)]
pub struct Activation {
    pub val: D6,
    pub is_used: bool,
}

// fn roll_activations(num: usize) -> Vec<Activation> {
//     let mut result: Vec<Activation> = (1..=num)
//         .map(|_| Activation {
//             val: D6::roll(),
//             is_used: false,
//         })
//         .collect();
//     result.sort();
//     result
// }

// fn use_activation(d6: D6, mut activations: Vec<Activation>) -> Vec<Activation> {
//     // let mut i: Option<Activation> = None;
//     // let mut val =

//     for a in activations.iter_mut() {
//         if !a.is_used && a.val >= d6 {
//             a.is_used = true;
//             return activations;
//         }
//     }

//     panic!("Cannot finde {:?} in {:?}", d6, activations);
// }

#[derive(Debug, Clone)]
pub struct Weapon {
    pub name: DisplayStr,
    tags: Vec<Tag>,
    to_hit: Dice,
}

#[derive(Debug, Clone)]
pub struct Attack {
    pub name: DisplayStr,
    pub roll: Roll,
    pub damage: u8,
    pub costs: u8,
}

#[derive(Debug, Clone)]
pub struct AttackOption {
    pub name: DisplayStr,
    pub dice: u8,
    pub damage: u8,
    pub costs: u8,
    pub distance: (f32, f32), // (min, max)
}

impl AttackOption {
    // fn into_attack(self, w: &Weapon) -> Attack {
    //     Attack {
    //         name: self.name,
    //         roll: Roll::new(self.dice, w.to_hit),
    //         damage: self.damage,
    //         costs: self.costs,
    //     }
    // }

    fn into_attack2(self, _a: &Actor) -> Attack2 {
        Attack2 {
            to_hit: 3,
            to_wound: 3,
        }
    }
}

pub struct Attack2 {
    to_hit: i8,
    to_wound: i8,
}

#[derive(Debug, Clone)]
pub struct DefenceOption {
    name: DisplayStr,
    dice: u8,
    required_ability: Option<Vec<Tag>>,
    costs: u8,
}

// impl DefenceOption {
//     pub fn into_defence(self) -> Defence {
//         Defence {
//             name: self.name,
//             roll: Roll::new(self.dice, Dice::new(4)),
//             costs: self.costs,
//         }
//     }
// }

#[derive(Debug, Clone)]
pub struct Defence {
    pub name: DisplayStr,
    pub roll: Roll,
    pub costs: u8,
}

#[derive(Hash, PartialEq, Eq, Debug, Clone, Copy)]
pub struct Tag(pub &'static str);

#[derive(Debug, Clone)]
pub struct Trait {
    name: DisplayStr,
    effects: Vec<Effect>,
    source: TraitSource,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum TraitSource {
    IntrinsicProperty,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Effect {
    AttributeModifier(Attribute, i8),
    Dying(),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Attribute {
    Wound,
    // ToHit,
    // ToWound,
    // Defence,
    // Protection,
}

#[derive(Debug, Clone)]
struct Wound {
    pain: u8,
    wound: u8,
}

impl Wound {
    fn from_roll(rr: &RR) -> Self {
        match rr {
            RR::Fail | RR::CritFail => Self { pain: 0, wound: 0 },
            RR::SuccessBut => Self { pain: 1, wound: 0 },
            RR::Success => Self { pain: 1, wound: 1 },
            RR::CritSuccess => Self { pain: 2, wound: 2 },
        }
    }
}

/// The result of a combat
#[derive(Debug, Clone)]
pub enum CombatResult {
    Miss(Actor),
    Block(),
    Hit(Condition),
}

#[derive(Debug, Clone)]
pub enum Condition {
    Alive(Actor),
    Dead(WorldPos, Item),
}

pub fn combat(attack: AttackOption, attacker: Actor, target: Actor) -> (CombatResult, Vec<String>) {
    let attack = attack.into_attack2(&attacker);
    let to_hit_result = D6::roll().result(target.defence() - attack.to_hit);
    let mut log = vec!(format!("{} attacks {}", attacker.name, target.name));

    match to_hit_result {
        RR::CritFail | RR::Fail => {
            log.push(format!("{} misses", attacker.name));
                
            (CombatResult::Miss(target), log)
        }

        RR::SuccessBut | RR::Success | RR::CritSuccess => {
            let to_wound_result = D6::roll().result(target.protection() - attack.to_wound);

            log.push(format!("{} hits", attacker.name));

            match to_wound_result {
                RR::CritFail | RR::Fail =>
                    (CombatResult::Block(), log),

                RR::SuccessBut | RR::Success | RR::CritSuccess =>
                    (CombatResult::Hit(target.wound(Wound::from_roll(&to_wound_result))), log),
            }
        }
    }
}

// pub fn combat(attack: Attack, defence: Defence, target: Actor) -> CombatResult {
//     let hits = attack.roll.successes as i8 - defence.roll.successes as i8;
//     let dmg = hits * attack.damage as i8 - target.protection();
//     let target = target.done(defence.costs);

//     if dmg <= 0 {
//         // TODO bonus for very good defence
//         CombatResult::Strike(target)
//     } else {
//         if target.has_effect(&Effect::Dying()) {
//             CombatResult::Hit(Condition::Dead(
//                 target.pos,
//                 Item {
//                     name: format!("Corpse of {}", target.name),
//                     look: vec![("corpses", 1)],
//                 },
//             ))
//         } else {
//             let mut wounds = target.wounds.clone();
//             let new_wound = Wound {}; // TODO consider attack (e.g. to cause bleeding)

//             wounds.push(new_wound);

//             if dmg <= 5 {
//                 // minor wound
//                 // => no save required
//                 CombatResult::Hit(Condition::Alive(Actor {
//                     wounds,
//                     ..target.clone()
//                 }))
//             } else {
//                 // critical wound
//                 // => additional save roll required
//                 let tries = save_roll(wounds.len());
//                 let remaining_energy = min(
//                     target.energy(),
//                     target.attributes().power + target.energy() - tries as i8,
//                 );

//                 let mut effects = target.effects.clone();

//                 if remaining_energy < 0 {
//                     effects.push(Effect::Dying());
//                 }

//                 CombatResult::Hit(Condition::Alive(Actor {
//                     wounds,
//                     effects,
//                     energy: (remaining_energy, target.energy.1),
//                     ..target.clone()
//                 }))
//             }
//         }
//     }
// }

// fn save_roll(to_save: usize) -> usize {
//     let mut tries = 0;
//     let mut successes = 0;
//     while successes < to_save {
//         tries += 1;
//         if Roll::new(1, Dice::new(4)).successes > 0 {
//             successes += 1;
//         }
//     }
//     tries
// }

fn generate_name() -> String {
    extern crate rand;
    use rand::prelude::*;
    let mut rng = thread_rng();
    [
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
    ]
    .choose(&mut rng)
    .unwrap()
    .to_string()
}

pub fn generate_player(pos: WorldPos, t: Team) -> Actor {
    extern crate rand;
    use rand::prelude::*;
    let range = rand::distributions::Uniform::from(1..=100);
    let mut rng = rand::thread_rng();

    ActorBuilder::new(pos, t)
    // ActorBuilder::new(pos, Attributes::new(4, 4, 4), t)
        .armor(Armor {
            look: vec![("player", rng.sample(range))],
            protection: 2,
        })
        .build()
}

// pub fn generate_enemy(pos: WorldPos, t: Team) -> Actor {
//     extern crate rand;
//     use rand::prelude::*;
//     let range = rand::distributions::Uniform::from(1..=1216);
//     let mut rng = rand::thread_rng();

//     ActorBuilder::new(pos, Attributes::new(3, 3, 3), t)
//         .armor(Armor {
//             look: vec![("enemy", rng.sample(range))],
//             protection: 0,
//         })
//         .behaviour(AiBehaviour::Default)
//         .build()
// }

fn one_of<'a, T>(v: &'a Vec<T>) -> &'a T {
    use rand::seq::SliceRandom;
    v.choose(&mut rand::thread_rng()).unwrap()
}

pub fn generate_enemy_easy(pos: WorldPos, t: Team) -> Actor {
    ActorBuilder::new(pos, t)
        .armor(Armor {
            look: vec![("tile", 3965), ("tile", *one_of(&vec!(5747, 5748, 5749)))],
            protection: 0,
        })
        .behaviour(AiBehaviour::Default)
        .traits(vec!(Trait {
            name: DisplayStr("Fragile physiology"),
            effects: vec!(Effect::AttributeModifier(Attribute::Wound, -2)),
            source: TraitSource::IntrinsicProperty,
        }))
        .build()
}
