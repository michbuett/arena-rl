use crate::core::dice::*;
use crate::core::{Action, DisplayStr, Tile, WorldPos};
use std::cmp::max;

const STAT_AVERAGE: u8 = 4;

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

    pub fn has_charged(&self) -> bool {
        !self.quick_action_available
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
        let wounds_modifer = self
            .effects
            .iter()
            .map(|e| match e {
                Effect::AttributeModifier(Attribute::Wound, modifier) => *modifier,
                _ => 0,
            })
            .sum::<i8>();
        let default_wounds_num = 3;
        let min_wounds_num = 1;
        let max_wounds = max(min_wounds_num, default_wounds_num + wounds_modifer) as u8;
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

    pub fn defence(&self) -> i8 {
        STAT_AVERAGE as i8
    }

    pub fn protection(&self) -> i8 {
        0
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
}

impl AttackOption {
    fn into_attack(self, a: &Actor) -> Attack {
        let (mut to_hit, mut to_wound) = (STAT_AVERAGE, STAT_AVERAGE);

        if a.has_charged() {
            to_hit -= 1;
            to_wound += 1;
        }

        Attack {
            to_hit: to_hit as i8,
            to_wound: to_wound as i8,
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
    let attack = attack.into_attack(&attacker);
    let attack_difficulty = target.defence() - attack.to_hit;
    let to_hit_result = D6::roll().result(attack_difficulty);
    let mut log = vec![format!(
        "{} attacks {} (difficulty: {})",
        attacker.name, target.name, attack_difficulty
    )];

    match to_hit_result {
        RR::CritFail | RR::Fail => {
            log.push(format!("{} misses", attacker.name));

            (CombatResult::Miss(target), log)
        }

        RR::SuccessBut | RR::Success | RR::CritSuccess => {
            let to_wound_result = D6::roll().result(target.protection() - attack.to_wound);

            log.push(format!("{} hits", attacker.name));

            match to_wound_result {
                RR::CritFail | RR::Fail => (CombatResult::Block(), log),

                RR::SuccessBut | RR::Success | RR::CritSuccess => (
                    CombatResult::Hit(target.wound(Wound::from_roll(&to_wound_result))),
                    log,
                ),
            }
        }
    }
}
