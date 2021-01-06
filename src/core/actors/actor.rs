use crate::core::dice::*;
use crate::core::{Action, DisplayStr, Tile, WorldPos};
use std::cmp::max;

const STAT_AVERAGE: i8 = 0;

#[derive(Debug, Clone)]
pub struct Item {
    pub name: String,
    pub look: Look,
}


#[derive(Debug, Clone)]
pub enum AiBehaviour {
    Default,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Team(pub &'static str, pub u8, pub bool);

pub struct ActorBuilder {
    behaviour: Option<AiBehaviour>,
    pos: WorldPos,
    team: Team,
    look: Look,
    name: String,
    traits: Vec<Trait>,
}

impl ActorBuilder {
    pub fn new(name: String, pos: WorldPos, team: Team) -> Self {
        Self {
            pos,
            team,
            name,
            behaviour: None,
            look: vec![],
            traits: Vec::new(),
        }
    }

    pub fn build(self) -> Actor {
        Actor {
            name: self.name,
            active: false,
            pos: self.pos,
            pain: 0,
            wounds: 0,
            effects: Vec::new(),
            traits: Vec::new(),
            pending_action: None,
            behaviour: self.behaviour,
            team: self.team,
            look: self.look,
            turn: 0,
            quick_action_available: true,
        }
        .set_traits(self.traits)
    }

    pub fn behaviour(self, b: AiBehaviour) -> Self {
        Self {
            behaviour: Some(b),
            ..self
        }
    }

    pub fn look(self, look: Look) -> Self {
        Self { look, ..self }
    }

    pub fn traits(self, traits: Vec<Trait>) -> Self {
        Self { traits, ..self }
    }
}

pub type Look = Vec<(&'static str, u16)>;

// #[derive(Debug, Clone)]
// pub struct Armor {
//     pub look: Look,
//     pub protection: u8,
// }

#[derive(Debug, Clone)]
pub struct Actor {
    pain: u8,
    wounds: u8,
    effects: Vec<Effect>,
    traits: Vec<Trait>,
    look: Look,
    quick_action_available: bool,

    pub name: String,
    pub turn: u64,
    pub active: bool,
    pub team: Team,
    pub pos: WorldPos,
    pub pending_action: Option<(Action, u8)>,
    pub behaviour: Option<AiBehaviour>,
}

impl Actor {
    pub fn move_to(self, to: Tile) -> Self {
        assert!(self.can_move(), "Actor cannot move: {:?}", self);

        Self {
            pos: to.to_world_pos(),
            quick_action_available: false,
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
            quick_action_available: false,
            active: false,
            ..self
        }
    }

    pub fn start_next_turn(self) -> (Actor, Option<(Action, u8)>) {
        let pending_action = self.pending_action;
        let next_turn_actor = Self {
            pending_action: None,
            quick_action_available: true,
            ..self
        };

        (next_turn_actor, pending_action)
    }

    pub fn melee_attack(&self) -> AttackOption {
        AttackOption {
            name: DisplayStr("Melee Attack"),
            reach: 1,
            to_hit: STAT_AVERAGE,
            to_wound: STAT_AVERAGE,
        }
    }

    fn set_traits(self, traits: Vec<Trait>) -> Self {
        let effects = traits.iter().flat_map(|t| t.effects.to_vec()).collect();

        Self {
            traits,
            effects,
            ..self
        }
    }

    pub fn is_dying(&self) -> bool {
        self.effects.contains(&Effect::Dying())
    }

    fn wound(self, w: Wound) -> Condition {
        let default_wounds_num = 3 + self.stat(Stat::Wound);
        let max_wounds = max(1, default_wounds_num) as u8;
        let wounds = self.wounds + w.wound;

        if wounds < max_wounds {
            Condition::Alive(Self { wounds, ..self })
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
        &self.look
    }

    /// -3 => None
    /// -2 => Puny
    /// -1 => Low — rusty
    ///  0 => Average
    ///  1 => Good — trained (decent)
    ///  2 => Elite (only the best have elite stats)
    ///  3 => Exceptional (once per generagion; the best of the best)
    ///  4 => Legendary (once per era)
    ///  5 => Supernatural
    ///  6 => ? (Ultra, Marvelous)
    ///  7 => Godlike (unlimited power)
    pub fn stat(&self, stat: Stat) -> i8 {
        stat_mod(&self.effects, stat)
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord)]
pub struct Activation {
    pub val: D6,
    pub is_used: bool,
}

#[derive(Debug, Clone)]
pub struct AttackOption {
    pub name: DisplayStr,
    pub reach: u8,
    pub to_hit: i8,
    pub to_wound: i8,
}

impl AttackOption {
    fn into_attack(self, a: &Actor) -> Attack {
        Attack {
            to_hit: self.to_hit + a.stat(Stat::ToHit),
            to_wound: self.to_wound + a.stat(Stat::ToWound),
        }
    }
}

pub struct Attack {
    to_hit: i8,
    to_wound: i8,
}

#[derive(Debug, Clone)]
pub struct Trait {
    pub name: DisplayStr,
    pub effects: Vec<Effect>,
    pub source: TraitSource,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum TraitSource {
    IntrinsicProperty,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Effect {
    StatModifier(Stat, i8),
    Dying(),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Stat {
    Wound,
    ToHit,
    ToWound,
    Defence,
    Protection,
}

#[derive(Debug, Clone)]
struct Wound {
    pain: u8,
    wound: u8,
}

impl Wound {
    fn from_roll(rr: &RR) -> Self {
        match rr {
            RR::F_ | RR::FF => Self { pain: 0, wound: 0 },
            RR::SF => Self { pain: 1, wound: 0 },
            RR::S_ => Self { pain: 1, wound: 1 },
            RR::SS => Self { pain: 2, wound: 2 },
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
    let attack = attack.into_attack(&attacker);
    let attack_difficulty = attack.to_hit - target.stat(Stat::Defence);
    let to_hit_result = D6::roll().result(attack_difficulty);
    let mut log = vec![format!(
        "{} attacks {} (difficulty: {:?})",
        attacker.name, target.name, attack_difficulty
    )];

    match to_hit_result {
        RR::FF | RR::F_ => {
            log.push(format!("{} misses", attacker.name));
            (CombatResult::Miss(target), log)
        }

        RR::SF | RR::S_ | RR::SS => {
            let to_wound_difficulty = attack.to_wound - target.stat(Stat::Protection);
            let to_wound_result = D6::roll().result(to_wound_difficulty);

            log.push(format!("{} hits", attacker.name));

            match to_wound_result {
                RR::FF | RR::F_ => (CombatResult::Block(), log),

                RR::SF | RR::S_ | RR::SS => (
                    CombatResult::Hit(target.wound(Wound::from_roll(&to_wound_result))),
                    log,
                ),
            }
        }
    }
}

fn stat_mod(effects: &Vec<Effect>, stat: Stat) -> i8 {
    effects
        .iter()
        .map(|e| match e {
            Effect::StatModifier(s, modifier) => {
                if *s == stat {
                    *modifier
                } else {
                    0
                }
            }
            _ => 0,
        })
        .sum::<i8>()
}
