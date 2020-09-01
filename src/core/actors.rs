use std::cmp::min;

use super::dice::*;
use crate::core::{DisplayStr, WorldPos};

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
#[derive(Debug, Clone)]
pub struct Attributes {
    /// cognitiv abilities, intelligence, wisdom
    mind: i8,
    /// agility, dextery, speed
    speed: i8,
    /// strength, endurance
    power: i8,
}

impl Attributes {
    pub fn new(mind: i8, speed: i8, power: i8) -> Self {
        Self { mind, speed, power }
    }
}

#[derive(Debug, Clone)]
pub enum AiBehaviour {
    Default,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Team(pub &'static str, pub u8);

pub struct ActorBuilder {
    attributes: Attributes,
    behaviour: Option<AiBehaviour>,
    pos: WorldPos,
    team: Team,
    armor: Armor,
    name: String,
}

impl ActorBuilder {
    pub fn new(pos: WorldPos, attributes: Attributes, team: Team) -> Self {
        Self {
            pos,
            attributes,
            team,
            name: generate_name(),
            behaviour: None,
            armor: Armor {
                // default to "no armor"-armor
                look: vec![],
                protection: 0,
            },
        }
    }

    pub fn build(self) -> Actor {
        let a = self.attributes;
        let energy = a.speed + a.power;

        Actor {
            name: self.name,
            active: false,
            pos: self.pos,
            energy: (energy, energy),
            attributes: (a.clone(), a.clone()),
            wounds: Vec::new(),
            effects: Vec::new(),
            behaviour: self.behaviour,
            team: self.team,
            armor: self.armor,
            turn: 0,
            attacks: vec![
                AttackOption {
                    name: DisplayStr("Swing"),
                    dice: 3,
                    distance: (0.0, 1.42),
                    damage: 4,
                    costs: 3,
                },
                AttackOption {
                    name: DisplayStr("Strong Blow"),
                    dice: 3,
                    distance: (0.0, 1.42),
                    damage: 5,
                    costs: 4,
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
        }
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
    attributes: (Attributes, Attributes),
    /// (current, max)
    energy: (i8, i8),
    wounds: Vec<Wound>,
    effects: Vec<Effect>,
    wields: Weapon,
    attacks: Vec<AttackOption>,
    defences: Vec<DefenceOption>,
    armor: Armor,

    pub turn: u64,
    pub active: bool,
    pub team: Team,
    pub pos: WorldPos,
    pub behaviour: Option<AiBehaviour>,
}

impl Actor {
    pub fn move_to(self, pos: WorldPos) -> Self {
        Self { pos, ..self }
    }

    pub fn is_pc(&self) -> bool {
        self.behaviour.is_none()
    }

    pub fn initiative(&self) -> u32 {
        if self.energy() <= 0
            || self.attributes().speed <= 0
            || self.has_effect(&Effect::Dead())
            || self.has_effect(&Effect::Dying())
        {
            return 0;
        }

        10_000 * self.energy() as u32
            + 100 * self.attributes().speed as u32
            + Roll::new(1, Dice::new(1)).total()
    }

    pub fn next_turn(self, turn: u64) -> Condition {
        let (e_current, e_max) = self.energy;
        let mut e_new = min(e_max, e_current + e_max);
        let power = self.attributes().power;
        let num_wounds = self.num_wounds();
        let mut effects = self.effects;

        if effects.contains(&Effect::Dying()) {
            let tries = save_roll(num_wounds) as i8;

            if tries <= power {
                // actor has recovered from its dying state
                effects = effects
                    .drain(..)
                    .filter(|e| *e != Effect::Dying())
                    .collect();
            } else {
                e_new = min(e_new, power + e_new - tries);

                if e_new < 0 {
                    // it's over now...
                    return Condition::Dead(self.pos, Item {
                        name: format!("Corpse of {}", self.name),
                        look: vec!(("corpses", 1))
                    });
                }
            }
        }

        Condition::Alive(Actor {
            turn,
            effects,
            energy: (e_new, e_max),
            ..self
        })
    }

    pub fn done(self, costs: u8) -> Self {
        Actor {
            active: false,
            energy: (self.energy.0 - costs as i8, self.energy.1),
            ..self
        }
    }

    pub fn energy(&self) -> i8 {
        self.energy.0
    }

    fn attributes(&self) -> &Attributes {
        &self.attributes.1
    }

    pub fn attacks(&self, target: &Actor) -> Vec<Attack> {
        let distance = WorldPos::distance(&self.pos, &target.pos);
        self.attacks
            .iter()
            .filter(|o| o.distance.0 <= distance && distance <= o.distance.1)
            .cloned()
            .map(|o| o.into_attack(&self.wields))
            .collect()
    }

    pub fn defences(&self, _: &Attack) -> Vec<Defence> {
        // TODO: consider attack and remaining energy
        self.defences
            .iter()
            .cloned()
            .map(|o| o.into_defence())
            .collect()
    }

    pub fn has_effect(&self, e: &Effect) -> bool {
        self.effects.contains(e)
    }

    pub fn num_wounds(&self) -> usize {
        self.wounds.len()
    }

    pub fn look(&self) -> &Look {
        &self.armor.look
    }

    pub fn protection(&self) -> i8 {
        self.armor.protection as i8 + self.attributes().power - 3
    }
}

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
    distance: (f32, f32), // (min, max)
}

impl AttackOption {
    pub fn into_attack(self, w: &Weapon) -> Attack {
        Attack {
            name: self.name,
            roll: Roll::new(self.dice, w.to_hit),
            damage: self.damage,
            costs: self.costs,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DefenceOption {
    name: DisplayStr,
    dice: u8,
    required_ability: Option<Vec<Tag>>,
    costs: u8,
}

impl DefenceOption {
    pub fn into_defence(self) -> Defence {
        Defence {
            name: self.name,
            roll: Roll::new(self.dice, Dice::new(4)),
            costs: self.costs,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Defence {
    pub name: DisplayStr,
    pub roll: Roll,
    pub costs: u8,
}

#[derive(Hash, PartialEq, Eq, Debug, Clone, Copy)]
pub struct Tag(pub &'static str);


#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Effect {
    Dying(),
    Dead(),
}

#[derive(Debug, Clone)]
pub struct Wound {}

/// The result of a combat
#[derive(Debug, Clone)]
pub enum CombatResult {
    Strike(Actor),
    Hit(Condition),
}

#[derive(Debug, Clone)]
pub enum Condition {
    Alive(Actor),
    Dead(WorldPos, Item),
}

pub fn combat(attack: Attack, defence: Defence, target: Actor) -> CombatResult {
    let hits = attack.roll.successes as i8 - defence.roll.successes as i8;
    let dmg = hits * attack.damage as i8 - target.protection();
    let target = target.done(defence.costs);

    if dmg <= 0 {
        // TODO bonus for very good defence
        CombatResult::Strike(target)
    } else {
        if target.has_effect(&Effect::Dying()) {
            CombatResult::Hit(Condition::Dead(target.pos, Item {
                name: format!("Corpse of {}", target.name),
                look: vec!(("corpses", 1))
            }))
        } else {
            let mut wounds = target.wounds.clone();
            let new_wound = Wound {}; // TODO consider attack (e.g. to cause bleeding)

            wounds.push(new_wound);

            if dmg <= 5 {
                // minor wound
                // => no save required
                CombatResult::Hit(Condition::Alive(Actor {
                    wounds,
                    ..target.clone()
                }))
            } else {
                // critical wound
                // => additional save roll required
                let tries = save_roll(wounds.len());
                let remaining_energy = min(
                    target.energy(),
                    target.attributes().power + target.energy() - tries as i8,
                );

                let mut effects = target.effects.clone();

                if remaining_energy < 0 {
                    effects.push(Effect::Dying());
                }

                CombatResult::Hit(Condition::Alive(Actor {
                    wounds,
                    effects,
                    energy: (remaining_energy, target.energy.1),
                    ..target.clone()
                }))
            }
        }
    }
}

fn save_roll(to_save: usize) -> usize {
    let mut tries = 0;
    let mut successes = 0;
    while successes < to_save {
        tries += 1;
        if Roll::new(1, Dice::new(4)).successes > 0 {
            successes += 1;
        }
    }
    tries
}

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

    ActorBuilder::new(pos, Attributes::new(4, 4, 4), t)
        .armor(Armor { look: vec!(("player", rng.sample(range))), protection: 2 })
        .build()
}

pub fn generate_enemy(pos: WorldPos, t: Team) -> Actor {
    extern crate rand;
    use rand::prelude::*;
    let range = rand::distributions::Uniform::from(1..=1216);
    let mut rng = rand::thread_rng();

    ActorBuilder::new(pos, Attributes::new(3, 3, 3), t)
        .armor(Armor { look: vec!(("enemy", rng.sample(range))), protection: 0 })
        .behaviour(AiBehaviour::Default)
        .build()
}

