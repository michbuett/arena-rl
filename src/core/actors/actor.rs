use std::cmp::max;

pub use super::traits::*;

use crate::core::dice::*;
use crate::core::{Action, DisplayStr, Tile, WorldPos};

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

    pub fn build(mut self) -> Actor {
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
        .add_traits(&mut self.traits)
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
    pub effects: Vec<(DisplayStr, Effect)>,
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
            // active: false,
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

    pub fn can_activate(&self) -> bool {
        // self.pending_action.is_none() && self.quick_action_available
        self.pending_action.is_none()
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
        let mut new_traits = Vec::new();

        for t in self.traits.iter() {
            if let Trait {
                source: TraitSource::Temporary(time),
                ..
            } = t
            {
                if *time > 1 {
                    new_traits.push(t.clone());
                }
            } else {
                new_traits.push(t.clone());
            }
        }

        let pending_action = self.pending_action;
        let next_turn_actor = Self {
            pending_action: None,
            quick_action_available: true,
            traits: Vec::new(),
            ..self
        }
        .add_traits(&mut new_traits);

        (next_turn_actor, pending_action)
    }

    pub fn ability_self(&self) -> Vec<(DisplayStr, Trait, u8)> {
        let mut result = vec!();
        
        for e in self.effects.iter() {
            if let (_, Effect::GiveTrait(name, AbilityTarget::OnSelf, t)) = e {
                result.push((name.clone(), t.clone(), 0));
            }
        }

        result.push((DisplayStr::new("Recover"), Trait {
            name: DisplayStr::new("Recovering"),
            effects: vec!(Effect::Recovering),
            source: TraitSource::Temporary(1),
        }, 0));

        result
    }

    pub fn melee_attack(&self) -> AttackOption {
        AttackOption {
            name: DisplayStr::new("Unarmed attack"),
            reach: 1,
            to_hit: 0,
            to_wound: 0,
            // used_skill: Attr::ToHit,
            // counter_skill: Attr::Defence,
        }
    }

    pub fn melee_defence(&self) -> Option<Defence> {
    // pub fn melee_defence(&self) -> Option<DefenceOption> {
        for (_, eff) in self.effects.iter() {
            match eff {
                Effect::MeleeDefence(name, modifier) => {
                    return Some(Defence {
                        name: name.clone(),
                        defence: self.attr(Attr::MeleeDefence).modify(name.clone(), *modifier),
                    })
                    // return Some(DefenceOption {
                    //     name: name.clone(),
                    //     modifier: *modifier,
                    // })
                }
                _ => {}
            }
        }

        None
    }

    pub fn add_traits(self, new_traits: &mut Vec<Trait>) -> Self {
        let mut traits = self.traits;
        traits.append(new_traits);
        let effects = traits
            .iter()
            .flat_map(|t| {
                t.effects
                    .iter()
                    .map(|e| (t.name.clone(), e.clone()))
                    .collect::<Vec<_>>()
            })
            .collect();

        Self {
            traits,
            effects,
            ..self
        }
    }

    // fn set_traits(self, traits: Vec<Trait>) -> Self {
    //     let effects = traits.iter().flat_map(|t| t.effects.to_vec()).collect();

    //     Self {
    //         traits,
    //         effects,
    //         ..self
    //     }
    // }

    pub fn is_dying(&self) -> bool {
        self.effects.iter().any(|(_, e)| match e {
            Effect::Dying => true,
            _ => false,
        })
    }
 
    fn wound(self, w: Wound) -> Condition {
        let default_wounds_num = 3 + self.attr(Attr::Wound).val();
        let max_wounds = max(1, default_wounds_num) as u8;
        let wounds = self.wounds + w.wound;
        let pain = self.pain + w.pain;

        if wounds < max_wounds {
            Condition::Alive(Self { wounds, pain, ..self })
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

    // pub fn wounds(&self) -> (u8, u8) {
    //     let default_wounds_num = 3 + self.attr(Attr::Wound).val();
    //     let max_wounds = max(1, default_wounds_num) as u8;

    //     (self.wounds, max_wounds)
    // }

    #[deprecated]
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
    pub fn attr(&self, s: Attr) -> AttrVal {
        AttrVal::new(s, &self.effects)
    }

    pub fn active_traits(&self) -> ActiveTraitIter {
        ActiveTraitIter(self.traits.iter())
    }
}

pub struct ActiveTraitIter<'a> (std::slice::Iter<'a, Trait>);

impl<'a> Iterator for ActiveTraitIter<'a> {
    type Item = &'a Trait;

    fn next(&mut self) -> Option<&'a Trait> {
        self.0.next() // TODO consider conditions
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
    // pub used_skill: Attr,
    // pub counter_skill: Attr,
}

impl AttackOption {
    fn into_attack(self, a: &Actor) -> Attack {
        Attack {
            to_hit: a.attr(Attr::ToHit).modify(self.name.clone(), self.to_hit),
            to_wound: a.attr(Attr::ToWound).modify(self.name.clone(), self.to_wound),
            name: self.name,
            // used_skill: self.used_skill,
            // counter_skill: self.counter_skill,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Attack {
    name: DisplayStr,
    to_hit: AttrVal,
    to_wound: AttrVal,
    // used_skill: Attr,
    // counter_skill: Attr,
}

// #[derive(Debug, Clone)]
// pub struct DefenceOption {
//     pub name: DisplayStr,
//     pub modifier: i8 // TODO: support skill modifier like "Big Shield", ...
// }

// impl DefenceOption {
//     fn into_defence(self, a: &Actor) -> Defence {
//         Defence {
//             defence: a.attr(Attr::Defence).modify(self.name.clone(), self.modifier),
//             // name: self.name,
//         }
//     }
// }

#[derive(Debug, Clone)]
pub struct Defence {
    pub name: DisplayStr,
    pub defence: AttrVal,
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
pub struct CombatResult2 {
    attacker: Actor,
    defender: Actor,
    log: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum Condition {
    Alive(Actor),
    Dead(WorldPos, Item),
}

pub fn combat(attack: AttackOption, attacker: Actor, target: Actor) -> (CombatResult, DisplayStr) {
    let mut attack = attack.into_attack(&attacker);
    let attack_difficulty = attack.to_hit.val() - target.attr(Attr::Defence).val();
    let mut to_hit_roll = D6::roll();
    let mut log = vec![format!(
        "{} attacks {} (difficulty: {:?}, roll: {})",
        attacker.name, target.name, attack_difficulty, to_hit_roll.0
    )];

    if let Some(defence) = target.melee_defence() {
        let defence_roll = D6::roll();
        to_hit_roll = to_hit_roll.modify(-1 * defence_roll.0 as i8);
        log.push(format!("{} counters with {} (roll: {})", target.name, defence.name, defence_roll.0));
    }

    let attack_difficulty = attack.to_hit.val() - target.attr(Attr::Defence).val();
    let to_hit_result = RR::from_roll(to_hit_roll, attack_difficulty);

    match to_hit_result {
        RR::FF | RR::F_ => {
            log.push("Miss".to_string());
            (CombatResult::Miss(target), to_display_str(log))
        }

        RR::SF | RR::S_ | RR::SS => {
            let mut hit_str = "Hit!".to_string();
                
            if let RR::SF = to_hit_result {
                hit_str = "Scratch".to_string();
                attack.to_wound = attack.to_wound.modify(DisplayStr::new("Scratch hit"), -1);
            }

            if let RR::SS = to_hit_result {
                hit_str = "Critical Hit!!!".to_string();
                attack.to_wound = attack.to_wound.modify(DisplayStr::new("Critical hit"), 1);
            }
        
            let to_wound_difficulty = target.attr(Attr::Protection).val() - attack.to_wound.val();
            let to_wound_result = D6::roll().result(to_wound_difficulty);

            log.push(hit_str);

            match to_wound_result {
                RR::FF | RR::F_ => {
                    log.push(format!("{} could not be wounded", target.name));
                    (CombatResult::Block(), to_display_str(log))
                }

                RR::SF | RR::S_ | RR::SS => {
                    let w = Wound::from_roll(&to_wound_result);

                    if w.wound == 1 {
                        log.push(format!("{} was wounded", target.name));
                    } else if w.wound > 1 {
                        log.push(format!("{} suffered a critical wound", target.name));
                    } else if w.pain > 0 {
                        log.push(format!("{} feels the pain but will not have a lasting insury", target.name));
                    } else {
                        log.push(format!("{} was completely unharmed", target.name));
                    }

                    (
                        CombatResult::Hit(target.wound(Wound::from_roll(&to_wound_result))),
                        to_display_str(log),
                    )
                }
            }
        }
    }
}

fn to_display_str(l: Vec<String>) -> DisplayStr {
    DisplayStr::new(l.join("\n"))
}
