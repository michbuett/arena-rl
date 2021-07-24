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

// const VISUAL_BODY: u8 = 0;
// const VISUAL_HEAD: u8 = 1;
// const VISUAL_ACCESSORY_1: u8 = 2;
// const VISUAL_ACCESSORY_2: u8 = 3;
// const VISUAL_ITEM_1: u8 = 4;
// const VISUAL_ITEM_2: u8 = 5;

// #[test]
// fn test_array_indexed_by_enum() {
//     let arr = [1, 2, 3];
//     let x = arr[Visual::Body as usize];
//     let y = arr[Visual::Head as usize];

//     assert_eq!(x, 1);
//     assert_eq!(y, 2);
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
        let mut result = vec![];

        for e in self.effects.iter() {
            if let (_, Effect::GiveTrait(name, AbilityTarget::OnSelf, t)) = e {
                result.push((name.clone(), t.clone(), 0));
            }
        }

        result.push((
            DisplayStr::new("Recover"),
            Trait {
                name: DisplayStr::new("Recovering"),
                effects: vec![Effect::Recovering],
                source: TraitSource::Temporary(1),
            },
            0,
        ));

        result
    }

    pub fn melee_attack(&self) -> AttackOption {
        for (_, eff) in self.effects.iter() {
            match eff {
                Effect::MeleeAttack(name, reach, to_hit, to_wound) => {
                    return AttackOption {
                        name: name.clone(),
                        min_distance: 0,
                        max_distance: *reach,
                        to_hit: *to_hit,
                        to_wound: *to_wound,
                    }
                }
                _ => {}
            }
        }

        AttackOption {
            name: DisplayStr::new("Unarmed attack"),
            min_distance: 0,
            max_distance: 1,
            to_hit: 0,
            to_wound: 0,
        }
    }

    pub fn range_attack(&self, _distance: u8) -> Option<AttackOption> {
        for (_, eff) in self.effects.iter() {
            match eff {
                Effect::RangeAttack(name, min_distance, max_distance, to_hit, to_wound) => {
                    return Some(AttackOption {
                        name: name.clone(),
                        min_distance: *min_distance,
                        max_distance: *max_distance,
                        to_hit: *to_hit,
                        to_wound: *to_wound,
                    })
                }
                _ => {}
            }
        }

        None
    }

    pub fn melee_defence(&self) -> Option<Defence> {
        for (_, eff) in self.effects.iter() {
            match eff {
                Effect::MeleeDefence(name, modifier) => {
                    return Some(Defence {
                        name: name.clone(),
                        defence: self
                            .attr(Attr::MeleeDefence)
                            .modify(name.clone(), *modifier),
                    });
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

    fn wound(self, w: Wound) -> Condition {
        let default_wounds_num = 3 + self.attr(Attr::Wound).val();
        let max_wounds = max(1, default_wounds_num) as u8;
        let wounds = self.wounds + w.wound;
        let pain = self.pain + w.pain;

        if wounds < max_wounds {
            Condition::Alive(Self {
                wounds,
                pain,
                ..self
            })
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

    /// Describes the current health condition of an actor (pain, wounds, ...)
    pub fn health(&self) -> (u8, u8) {
        (self.pain, self.wounds)
    }

    pub fn look(&self) -> &Look {
        &self.look
    }

    /// -3 => None
    /// -2 => Puny
    /// -1 => Low — rusty
    ///  0 => Average
    ///  1 => Good — trained (decent)
    ///  2 => Very good
    ///  3 => Elite (only the best have elite stats)
    ///  4 => Exceptional (once per generagion; the best of the best)
    ///  5 => Legendary (once per era)
    ///  6 => Supernatural
    ///  7 => Godlike (unlimited power)
    pub fn attr(&self, s: Attr) -> AttrVal {
        let mut result = AttrVal::new(s, &self.effects);

        if self.pain > 0 {
            result = result.modify(DisplayStr::new("pain"), -1 * self.pain as i8);
        }

        result
    }

    pub fn active_traits(&self) -> ActiveTraitIter {
        ActiveTraitIter(self.traits.iter())
    }
}

pub struct ActiveTraitIter<'a>(std::slice::Iter<'a, Trait>);

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
    pub min_distance: u8,
    pub max_distance: u8,
    pub to_hit: i8,
    pub to_wound: i8,
}

impl AttackOption {
    fn into_attack(self, a: &Actor) -> Attack {
        Attack {
            to_hit: a.attr(Attr::ToHit).modify(self.name.clone(), self.to_hit),
            to_wound: a
                .attr(Attr::ToWound)
                .modify(self.name.clone(), self.to_wound),
            name: self.name,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Attack {
    name: DisplayStr,
    to_hit: AttrVal,
    to_wound: AttrVal,
}

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
            RR::FF(_) => Self { pain: 0, wound: 0 },
            RR::SF => Self { pain: 1, wound: 0 },
            RR::SS(n) => Self {
                pain: *n,
                wound: *n,
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct CombatResult {
    pub attacker: Condition,
    pub target: Condition,
    pub log: Vec<CombatEvent>,
}

#[derive(Debug, Clone)]
pub enum Condition {
    Unchanged,
    Alive(Actor),
    Dead(WorldPos, Item),
}

#[derive(Debug, Clone)]
pub enum CombatEventFx {
    Text(DisplayStr, WorldPos, u64),
    Scream(DisplayStr, WorldPos, u64),
    Sprite(String, WorldPos, u64),
    Projectile(String, WorldPos, WorldPos, u64),
}

#[derive(Debug, Clone)]
pub struct CombatEvent {
    pub fx: Option<CombatEventFx>,
    pub log: DisplayStr,
}

pub fn combat(attack: AttackOption, attacker: Actor, target: Actor) -> CombatResult {
    let mut attack = attack.into_attack(&attacker);
    let mut to_hit_roll = D6::roll();
    let mut combat_log = vec![CombatEvent {
        fx: Some(CombatEventFx::Sprite("fx-hit-1".to_string(), target.pos, 400)),
        log: DisplayStr::new(format!("{} attacks {} with {}", attacker.name, target.name, attack.name)),
    }];
    let attack_difficulty = target.attr(Attr::Defence).val() - attack.to_hit.val();

    let to_hit_result = if let Some(defence) = target.melee_defence() {
        let defence_roll = D6::roll();
        let result_before_defence = RR::from_roll(to_hit_roll, attack_difficulty);

        to_hit_roll = to_hit_roll.modify(-1 * defence_roll.0 as i8);

        let result_after_defence = RR::from_roll(to_hit_roll, attack_difficulty);

        combat_log.push(combat_event_attack_with_defence(
            &attacker,
            &target,
            &defence,
            result_before_defence,
            result_after_defence,
        ));

        result_after_defence
    } else {
        let roll_result = RR::from_roll(to_hit_roll, attack_difficulty);

        combat_log.push(combat_event_attack_without_defence(&attacker, roll_result));

        roll_result
    };

    let mut attacker_condition = Condition::Unchanged;
    let mut target_condition = Condition::Unchanged;

    match to_hit_result {
        RR::FF(n) if n > 1 => {
            attacker_condition = Condition::Alive(attacker.clone().add_traits(&mut vec![Trait {
                name: DisplayStr::new("Off balance"),
                effects: vec![Effect::AttrMod(Attr::Defence, -1 * n as i8)],
                source: TraitSource::Temporary(1),
            }]));
        }

        RR::SF => {
            attack.to_wound = attack.to_wound.modify(DisplayStr::new("Scratch hit"), -1);
        }

        RR::SS(n) if n > 1 => {
            attack.to_wound = attack.to_wound.modify(DisplayStr::new("Critical hit"), n as i8);
        }

        _ => {}
    };

    match to_hit_result {
        RR::SF | RR::SS(_) => {
            let to_wound_difficulty = target.attr(Attr::Protection).val() - attack.to_wound.val();
            let to_wound_result = RR::from_roll(D6::roll(), to_wound_difficulty);

            combat_log.push(combat_event_wound(&target, to_wound_result));

            target_condition = target.clone().wound(Wound::from_roll(&to_wound_result));

            if let Condition::Dead(_, _) = target_condition {
                combat_log.push(CombatEvent {
                    fx: None,
                    log: DisplayStr::new(format!("{} was killed by {}", target.name, attacker.name)),
                });
            }
        }

        _ => {}
    };

    // println!(
    //     "[DEBUG ATTACK]\n  attack to-hit: {:?}  attack to-wound: {:?}\n  defence: {:?}\n  attack_difficulty: {},  wound_difficulty: {}\n  attack roll: {:?}\n  result: {:?}",
    //     attack.to_hit,
    //     attack.to_wound,
    //     target.attr(Attr::Defence),
    //     attack_difficulty,
    //     target.attr(Attr::Protection).val() - attack.to_wound.val(),
    //     to_hit_roll,
    //     to_hit_result,
    // );

    CombatResult {
        attacker: attacker_condition,
        target: target_condition,
        log: combat_log,
    }
}

pub fn ranged_combat(attack: AttackOption, attacker: Actor, target: Actor) -> CombatResult {
    // println!("[DEBUG] ranged combat: attacker={:?}, target={:?}, attack={:?}", attacker, target, attack);

    let attack = attack.into_attack(&attacker);
    let to_hit_roll = D6::roll();
    let attack_difficulty = target.attr(Attr::Defence).val() - attack.to_hit.val();
    let to_hit_result = RR::from_roll(to_hit_roll, attack_difficulty);

    let mut combat_log = vec![CombatEvent {
        fx: Some(CombatEventFx::Projectile("fx-projectile-1".to_string(), attacker.pos, target.pos, 2000)),
        log: DisplayStr::new(format!("{} shoots at {}", attacker.name, target.name)),
    }];

    combat_log.push(combat_event_attack_without_defence(&attacker, to_hit_result));

    CombatResult {
        attacker: Condition::Alive(attacker),
        target: Condition::Alive(target),
        log: combat_log,
    }
}

fn combat_event_attack_with_defence(
    attacker: &Actor,
    target: &Actor,
    defence: &Defence,
    result_before_defence: RR,
    result_after_defence: RR,
) -> CombatEvent {
    match (result_before_defence, result_after_defence) {
        (RR::SF, RR::FF(_)) | (RR::SS(_), RR::FF(_)) => CombatEvent {
            fx: say("Block!", target.pos),
            log: DisplayStr::new(format!(
                "{} successfully blocks the attack with {}",
                target.name, defence.name
            )),
        },

        (RR::SS(_), RR::SF) => CombatEvent {
            fx: None,
            log: DisplayStr::new(format!(
                "{} uses {} but cannot avoid the attack completely",
                target.name, defence.name,
            )),
        },

        _ => combat_event_attack_without_defence(attacker, result_before_defence),
    }
}

fn combat_event_attack_without_defence(
    attacker: &Actor,
    roll_result: RR,
) -> CombatEvent {
    match roll_result {
        RR::FF(n) => if n > 1 {
            CombatEvent {
                fx: scream("DAMNED !!!", attacker.pos),
                log: DisplayStr::new(format!("{} misses badly and loses balance", attacker.name)),
            }
        } else {
            CombatEvent {
                fx: say("No", attacker.pos),
                log: DisplayStr::new(format!("{} misses", attacker.name)),
            }
        },

        RR::SF => CombatEvent {
            fx: None,
            log: DisplayStr::new(format!("{} hits but only briefly", attacker.name)),
        },

        RR::SS(n) => if n > 1 {
            CombatEvent {
                fx: scream("YES! DIE!!!", attacker.pos),
                log: DisplayStr::new(format!("{} scores a direct hit (+{})", attacker.name, n)),
            }
        } else {
            CombatEvent {
                fx: say("Nice!", attacker.pos),
                log: DisplayStr::new(format!("{} hits", attacker.name)),
            }
        },
    }
}

fn combat_event_wound(
    target: &Actor,
    roll_result: RR,
) -> CombatEvent {
    match roll_result {
        RR::FF(_) => CombatEvent {
            fx: say("Klong", target.pos),
            log: DisplayStr::new("The attack bounced harmlessly off the armor"),
        },

        RR::SF => CombatEvent {
            fx: say("Ouch!", target.pos),
            log: DisplayStr::new(format!("{} feels the pain but will not have a lasting insury", target.name)),
        },

        RR::SS(n) => if n > 1 {
            CombatEvent {
                fx: scream("AAAEIII!!!", target.pos),
                log: DisplayStr::new(format!("{} suffered a critical wound", target.name)),
            }
        } else {
            CombatEvent {
                fx: scream("Aaarg!!!", target.pos),
                log: DisplayStr::new(format!("{} was wounded", target.name)),
            }
        },
    }
}

fn say(s: &str, p: WorldPos) -> Option<CombatEventFx> {
    Some(CombatEventFx::Text(DisplayStr::new(s), p, 1000))
}

fn scream(s: &str, p: WorldPos) -> Option<CombatEventFx> {
    Some(CombatEventFx::Scream(DisplayStr::new(s), p, 1000))
}
