use std::cmp::max;
use std::collections::HashMap;
use std::time::Instant;

pub use super::traits::*;

use crate::core::dice::*;
use crate::core::{Act, DisplayStr, WorldPos};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct ID(Instant, u64, u64);

impl ID {
    pub fn new() -> Self {
        use rand::random;
        Self(Instant::now(), random(), random())
    }
}

#[derive(Debug, Clone)]
pub struct Item {
    pub id: ID,
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
    traits: HashMap<String, Trait>,
}

impl ActorBuilder {
    pub fn new(name: String, pos: WorldPos, team: Team) -> Self {
        Self {
            pos,
            team,
            name,
            behaviour: None,
            look: vec![],
            traits: HashMap::new(),
        }
    }

    pub fn build(self) -> Actor {
        let mut a = Actor {
            id: ID::new(),
            name: self.name,
            active: false,
            pos: self.pos,
            health: Health::new(0),
            effort: (0, 0),
            effects: Vec::new(),
            traits: self.traits,
            pending_action: None,
            behaviour: self.behaviour,
            team: self.team,
            look: self.look,
            engaged_in_combat: false,
        }
        .update_effects();

        let physical_strength = 3 + AttrVal::new(Attr::Physical, &a.effects).val();
        a.health = Health::new(max(physical_strength, 1) as u8);
        a
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

    pub fn traits(self, mut trait_list: Vec<(String, Trait)>) -> Self {
        let mut traits = HashMap::new();
        for (key, val) in trait_list.drain(..) {
            traits.insert(key, val);
        }
        Self { traits, ..self }
    }
}

pub type Look = Vec<(u8, String)>;

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
    traits: HashMap<String, Trait>,
    look: Look,

    /// used, max
    effort: (u8, u8),

    pub id: ID,
    pub health: Health,
    pub effects: Vec<(DisplayStr, Effect)>,
    pub engaged_in_combat: bool,
    pub name: String,
    pub active: bool,
    pub team: Team,
    pub pos: WorldPos,
    pub pending_action: Option<Act>,
    pub behaviour: Option<AiBehaviour>,
}

impl Actor {
    ////////////////////////////////////////////////////////////
    // Movement

    pub fn charge_to(mut self, p: WorldPos) -> Self {
        self.pos = p;

        self.traits.insert(
            "ability#charge-buff".to_string(),
            Trait {
                name: DisplayStr::new("Charging"),
                effects: vec![Effect::AttrMod(Attr::ToWound, 1)],
                source: TraitSource::Temporary(0),
                visuals: None,
            },
        );

        self.traits.insert(
            "ability#charge-debuff".to_string(),
            Trait {
                name: DisplayStr::new("Did charge"),
                effects: vec![Effect::AttrMod(Attr::MeleeDefence, -1)],
                source: TraitSource::Temporary(1),
                visuals: None,
            },
        );

        self.update_effects()
    }

    pub fn can_move(&self) -> bool {
        !self.engaged_in_combat && self.is_concious()
        // !self.engaged_in_combat && self.pending_action.is_none()
    }

    pub fn move_distance(&self) -> u8 {
        let available_e = self.available_effort() as i8;
        let move_mod = self.attr(Attr::Movement).val();

        max(1, available_e + move_mod) as u8
    }

    ////////////////////////////////////////////////////////////
    // A.I.

    pub fn is_pc(&self) -> bool {
        self.behaviour.is_none()
    }

    pub fn available_effort(&self) -> u8 {
        self.effort.1.checked_sub(self.effort.0).unwrap_or(0)
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

    pub fn prepare(mut self, act: Act) -> Self {
        let effort = act.allocated_effort.unwrap_or(self.available_effort());

        self.pending_action = Some(act);
        self.active = false;
        self.use_effort(effort)
    }

    pub fn use_ability(self, key: impl ToString, ability: Trait) -> Self {
        let msg = format!("Used ability {}", ability.name);

        self.add_trait(key.to_string(), ability)
            .prepare(Act::done(msg))
    }

    pub fn use_effort(mut self, effort: u8) -> Self {
        if effort > self.available_effort() {
            self.health = self.health.wound(Wound { pain: 1, wound: 0 });
        }
        self.effort.0 += effort;
        self
    }

    pub fn rest(mut self) -> Self {
        self.health = self.health.rest();
        self
    }

    pub fn start_next_turn(mut self, engaged_in_combat: bool) -> Actor {
        let mut new_traits = HashMap::new();
        let reserved_effort = self.available_effort();
        let (health, new_max_available_effort) = self.health.next_turn(reserved_effort);

        // handle temporary traits
        for (k, t) in self.traits.drain() {
            if let TraitSource::Temporary(time) = t.source {
                if time > 1 {
                    let mut new_t = t;
                    new_t.source = TraitSource::Temporary(time - 1);
                    new_traits.insert(k, new_t);
                }
            } else {
                // not a temporary trait
                new_traits.insert(k, t);
            }
        }

        Self {
            engaged_in_combat,
            health,
            effort: (0, new_max_available_effort),
            pending_action: None,
            traits: new_traits,
            ..self
        }
        .update_effects()
    }

    pub fn ability_self(&self) -> Vec<(String, Trait, u8)> {
        let mut result = vec![];

        for e in self.effects.iter() {
            if let (_, Effect::GiveTrait(key, t, AbilityTarget::OnSelf)) = e {
                result.push((key.clone(), t.clone(), 0));
            }
        }

        result.push((
            "ability#GatherStrength".to_string(),
            Trait {
                name: DisplayStr::new("Gather strength"),
                effects: vec![Effect::GatherStrength],
                source: TraitSource::Temporary(1),
                visuals: None,
            },
            0,
        ));

        result
    }

    pub fn attacks(&self) -> Vec<AttackOption> {
        let attacks = self
            .effects
            .iter()
            .filter_map(|(_, eff)| {
                match eff {
                    Effect::MeleeAttack {
                        name,
                        required_effort,
                        advance,
                        distance,
                        to_hit,
                        to_wound,
                        fx,
                        effects,
                    } => {
                        if *required_effort <= self.available_effort() + 1 {
                            Some(AttackOption {
                                name: name.clone(),
                                min_distance: 1,
                                max_distance: max(1, distance.unwrap_or(1)),
                                advance: advance.unwrap_or(0),
                                to_hit: to_hit.unwrap_or(0),
                                to_wound: to_wound.unwrap_or(0),
                                attack_type: AttackType::Melee(fx.to_string()),
                                required_effort: *required_effort,
                                effects: effects.clone(),
                            })
                        } else {
                            None
                        }
                    }

                    Effect::RangeAttack {
                        name,
                        distance,
                        to_hit,
                        to_wound,
                        fx,
                    } => Some(AttackOption {
                        name: name.clone(),
                        min_distance: distance.0,
                        max_distance: distance.1,
                        advance: 0,
                        to_hit: *to_hit,
                        to_wound: *to_wound,
                        attack_type: AttackType::Ranged(fx.to_string()),
                        required_effort: 3, // TODO read from effect
                        effects: None,      // TODO read from effect
                    }),

                    _ => None,
                }
            })
            .collect::<Vec<_>>();

        if attacks.is_empty() {
            vec![AttackOption {
                name: DisplayStr::new("Unarmed attack"),
                min_distance: 0,
                max_distance: 1,
                advance: 0,
                to_hit: 0,
                to_wound: -1,
                attack_type: AttackType::Melee("fx-hit-1".to_string()),
                required_effort: 2,
                effects: None,
            }]
        } else {
            attacks
        }
    }

    // #[deprecated]
    // pub fn melee_attack(&self) -> AttackOption {
    //     AttackOption {
    //         name: DisplayStr::new("Unarmed attack"),
    //         min_distance: 0,
    //         max_distance: 1,
    //         advance: 0,
    //         to_hit: 0,
    //         to_wound: 0,
    //         attack_type: AttackType::Melee("fx-hit-1".to_string()),
    //         difficulty: 3,
    //         required_effort: 2, // TODO read from effect
    //     }
    // }

    // #[deprecated]
    // pub fn range_attack(&self, d: u8) -> Option<AttackOption> {
    //     None
    // }

    // pub fn defence(&self, attack: &Attack) -> Option<Defence> {
    //     for (name, eff) in self.effects.iter() {
    //         match eff {
    //             Effect::Defence(modifier, defence_type) => {
    //                 match attack.attack_type {
    //                     AttackType::Melee(..) => {
    //                         return Some(Defence {
    //                             defence_type: defence_type.clone(),
    //                             defence: self
    //                                 .attr(Attr::MeleeDefence)
    //                                 .modify(name.clone(), *modifier),
    //                             num_dice: 3, // TODO
    //                         });
    //                     }

    //                     AttackType::Ranged(..) => {
    //                         // TODO: implement "take cover"
    //                     }
    //                 }
    //             }
    //             _ => {}
    //         }
    //     }

    //     None
    // }

    fn update_effects(mut self) -> Self {
        let effects = self
            .traits
            .values()
            .flat_map(|t| {
                t.effects
                    .iter()
                    .map(|e| (t.name.clone(), e.clone()))
                    .collect::<Vec<_>>()
            })
            .collect();

        self.effects = effects;
        self
    }

    pub fn add_trait(mut self, key: String, new_trait: Trait) -> Self {
        self.traits.insert(key, new_trait);
        self.update_effects()
    }

    // pub fn remove_trait(mut self, key: &str) -> Self {
    //     if let Some(_) = self.traits.remove(key) {
    //         self.update_effects()
    //     } else {
    //         self
    //     }
    // }

    pub fn wound(mut self, w: Wound) -> Self {
        self.health = self.health.wound(w);
        self

        // let wounds = self.wounds + w.wound;
        // let focus = self.focus - w.pain as i8;

        // Self {
        //     wounds,
        //     focus,
        //     ..self
        // }
    }

    pub fn is_alive(&self) -> bool {
        self.health.remaining_wounds > 0
    }

    pub fn is_concious(&self) -> bool {
        self.is_alive() && self.health.remaining_wounds >= self.health.pain
    }

    pub fn corpse(&self) -> Item {
        Item {
            id: ID::new(),
            name: format!("Corpse of {}", self.name),
            look: vec![(1, "corpses_1".to_string())],
        }
    }

    pub fn look(&self) -> Vec<String> {
        let mut result = self.look.clone();

        for Trait { visuals, .. } in self.traits.values() {
            if let Some(v) = visuals {
                result.push(v.clone());
            }
        }

        result.sort();
        result.drain(..).map(|(_, v)| v).collect()
    }

    /// Returns the active modifier for a given attribute
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
        AttrVal::new(s, &self.effects)
        // let mut av = AttrVal::new(s, &self.effects);
        // if self.health.focus > 0 {
        //     av = av.modify(DisplayStr::new("Focus"), 1);
        // } else if self.health.focus < 0 {
        //     av = av.modify(DisplayStr::new("Distracted"), -1);
        // }
        // av
    }

    pub fn active_traits(&self) -> ActiveTraitIter {
        ActiveTraitIter(self.traits.values())
    }
}

#[derive(Debug, Clone)]
pub struct Health {
    pub pain: u8,
    pub max_wounds: u8,
    pub recieved_wounds: u8,
    pub remaining_wounds: u8,
}

impl Health {
    fn new(max_wounds: u8) -> Self {
        Self {
            pain: 0,
            recieved_wounds: 0,
            max_wounds,
            remaining_wounds: max_wounds,
        }
    }

    fn rest(mut self) -> Self {
        let roll = Roll::new(self.remaining_wounds, 4);
        self.pain = self.pain.checked_sub(roll.num_successes).unwrap_or(0);
        println!(
            "[DEBUG REST] remaining wounds={}, pain={}, roll={:?}",
            self.remaining_wounds, self.pain, roll
        );
        self
    }

    fn next_turn(mut self, mut reserved_effort: u8) -> (Self, u8) {
        let roll = Roll::new(reserved_effort, 4);

        if roll.num_successes > self.pain {
            reserved_effort = 1;
            self.pain = 0;
        } else {
            reserved_effort = 0;
            self.pain -= roll.num_successes;
        }

        let max_effort = self
            .max_wounds
            .checked_sub(self.pain + self.recieved_wounds)
            .unwrap_or(0);
        let new_available_effort = max_effort + reserved_effort;

        (self, new_available_effort)
    }

    fn wound(mut self, w: Wound) -> Self {
        self.recieved_wounds += w.wound;
        self.remaining_wounds = self.remaining_wounds.checked_sub(w.wound).unwrap_or(0);
        self.pain += w.pain;
        // println!("[DEBUG] wounding actor: {:?}", w);
        self
    }
}

pub struct ActiveTraitIter<'a>(std::collections::hash_map::Values<'a, String, Trait>);

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
    pub advance: u8,
    pub to_hit: i8,
    pub to_wound: i8,
    pub attack_type: AttackType,
    pub required_effort: u8,
    pub effects: Option<Vec<(HitEffectCondition, HitEffect)>>,
}

impl AttackOption {
    pub fn into_attack(self, a: &Actor) -> Attack {
        let to_hit = a.attr(Attr::ToHit).modify(self.name.clone(), self.to_hit);
        let to_wound = a
            .attr(Attr::ToWound)
            .modify(self.name.clone(), self.to_wound);
        let num_dice = self.required_effort;

        // if a.available_effort() < self.required_effort {
        //     to_hit = to_hit.modify(DisplayStr::new("Over commitment"), -1);
        //     to_wound = to_wound.modify(DisplayStr::new("Over commitment"), -1);
        //     num_dice = a.available_effort();
        // }

        Attack {
            origin_pos: a.pos,
            to_hit,
            to_wound,
            name: self.name,
            attack_type: self.attack_type,
            num_dice,
            effects: self.effects,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Attack {
    pub origin_pos: WorldPos,
    pub name: DisplayStr,
    pub to_hit: AttrVal,
    pub to_wound: AttrVal,
    pub num_dice: u8,
    pub attack_type: AttackType,
    pub effects: Option<Vec<(HitEffectCondition, HitEffect)>>,
}

#[derive(Debug, Clone)]
pub enum AttackType {
    Melee(String),
    Ranged(String),
}

impl AttackType {
    pub fn is_melee(&self) -> bool {
        match self {
            AttackType::Melee(_) => true,
            _ => false,
        }
    }

    pub fn is_ranged(&self) -> bool {
        match self {
            AttackType::Ranged(_) => true,
            _ => false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Defence {
    pub defence: AttrVal,
    pub defence_type: DefenceType,
    pub num_dice: u8,
}

#[derive(Debug, Clone)]
pub struct Wound {
    pub pain: u8,
    pub wound: u8,
}

// impl Wound {
//     pub fn from_wound_roll(r: &Roll) -> Self {
//         match r.successes() {
//             0 => Self { pain: 0, wound: 0 },
//             1 => Self { pain: 1, wound: 0 },
//             n => Self {
//                 pain: n,
//                 wound: n - 1,
//             },
//         }
//     }
// }
