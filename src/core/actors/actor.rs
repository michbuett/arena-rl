use std::collections::HashMap;
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
        Actor {
            name: self.name,
            active: false,
            pos: self.pos,
            pain: 0,
            wounds: 0,
            effects: Vec::new(),
            traits: self.traits,
            pending_action: None,
            behaviour: self.behaviour,
            team: self.team,
            look: self.look,
            engaged_in_combat: false,
        }
        .update_effects()
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
    pain: u8,
    wounds: u8,
    traits: HashMap<String, Trait>,
    look: Look,

    pub effects: Vec<(DisplayStr, Effect)>,
    pub engaged_in_combat: bool,
    pub name: String,
    pub active: bool,
    pub team: Team,
    pub pos: WorldPos,
    pub pending_action: Option<(Action, u8)>,
    pub behaviour: Option<AiBehaviour>,
}

impl Actor {
    pub fn move_to(self, to: Tile) -> Self {
        // assert!(self.can_move(), "Actor cannot move: {:?}", self);
        Self {
            pos: to.to_world_pos(),
            ..self
        }
    }

    pub fn charge_to(self, to: Tile) -> Self {
        let mut result = self.move_to(to);
        result.traits.insert("ability#charge-buff".to_string(), Trait {
            name: DisplayStr::new("Charging"),
            effects: vec![
                Effect::AttrMod(Attr::ToWound, 1),
            ],
            source: TraitSource::Temporary(1),
            visuals: None,
        });
        result.traits.insert("ability#charge-debuff".to_string(), Trait {
            name: DisplayStr::new("Did charge"),
            effects: vec![
                Effect::AttrMod(Attr::MeleeDefence, -1),
            ],
            source: TraitSource::Temporary(2),
            visuals: None,
        });
        result.update_effects()
    }

    pub fn can_move(&self) -> bool {
        !self.engaged_in_combat && self.pending_action.is_none()
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
            active: false,
            ..self
        }
    }

    pub fn use_ability(self, key: impl ToString, ability: Trait) -> Self {
        let msg = format!("Used ability {}", ability.name);
        self.add_trait(key.to_string(), ability).prepare(Action::done(msg))
    }

    pub fn start_next_turn(mut self, engaged_in_combat: bool) -> Actor {
        let mut new_traits = HashMap::new();

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
            "ability#Recover".to_string(),
            Trait {
                name: DisplayStr::new("Recovering"),
                effects: vec![Effect::Recovering],
                source: TraitSource::Temporary(1),
                visuals: None,
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
                        attack_type: AttackType::Melee("fx-hit-1".to_string()),
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
            attack_type: AttackType::Melee("fx-hit-1".to_string()),
        }
    }

    pub fn range_attack(&self, d: u8) -> Option<AttackOption> {
        for (_, eff) in self.effects.iter() {
            match eff {
                Effect::RangeAttack { name, distance, to_hit, to_wound, fx } => {
                    if distance.0 <= d && d <= distance.1 {
                        return Some(AttackOption {
                            name: name.clone(),
                            min_distance: distance.0,
                            max_distance: distance.1,
                            to_hit: *to_hit,
                            to_wound: *to_wound,
                            attack_type: AttackType::Ranged(fx.to_string()),
                        })
                    }
                }

                _ => {}
            }
        }

        None
    }

    pub fn defence(&self, attack: &Attack) -> Option<Defence> {
        for (name, eff) in self.effects.iter() {
            match eff {
                Effect::Defence(modifier, defence_type) => {
                    match attack.attack_type {
                        AttackType::Melee(..) => {
                            return Some(Defence {
                                defence_type: defence_type.clone(),
                                defence: self
                                    .attr(Attr::MeleeDefence)
                                    .modify(name.clone(), *modifier),
                                num_dice: 3, // TODO
                            });
                        }

                        AttackType::Ranged(..) => {
                            // TODO: implement "take cover"
                        }
                    }
                }
                _ => {}
            }
        }

        None
    }

    fn update_effects(mut self) -> Self {
        let effects = self.traits
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

    pub fn remove_trait(mut self, key: &str) -> Self {
        if let Some(_) = self.traits.remove(key) {
            self.update_effects()
        } else {
            self
        }
    }

    // pub fn add_traits(self, new_traits: &mut Vec<Trait>) -> Self {
    //     let mut traits = self.traits;
    //     traits.append(new_traits);
    //     let effects = traits
    //         .iter()
    //         .flat_map(|t| {
    //             t.effects
    //                 .iter()
    //                 .map(|e| (t.name.clone(), e.clone()))
    //                 .collect::<Vec<_>>()
    //         })
    //         .collect();

    //     Self {
    //         traits,
    //         effects,
    //         ..self
    //     }
    // }

    pub fn wound(self, w: Wound) -> Self {
        // pub fn wound(self, w: Wound) -> Condition {
        // let default_wounds_num = 3 + self.attr(Attr::Wound).val();
        // let max_wounds = max(1, default_wounds_num) as u8;
        let wounds = self.wounds + w.wound;
        let pain = self.pain + w.pain;

        // if wounds < max_wounds {
        //     Condition::Alive(Self {
        //         wounds,
        //         pain,
        //         ..self
        //     })
        // } else {
        //     Condition::Dead(
        //         self.pos,
        //         Item {
        //             name: format!("Corpse of {}", self.name),
        //             look: vec![("corpses", 1)],
        //         },
        //     )
        // }

        Self {
            wounds,
            pain,
            ..self
        }
    }

    pub fn is_alive(&self) -> bool {
        let default_wounds_num = 3 + self.attr(Attr::Wound).val();
        let max_wounds = max(1, default_wounds_num) as u8;

        self.wounds < max_wounds
    }

    pub fn corpse(&self) -> Item {
        Item {
            name: format!("Corpse of {}", self.name),
            look: vec![(1, "corpses".to_string())],
        }
    }

    /// Describes the current health condition of an actor (pain, wounds, max_distance)
    pub fn health(&self) -> (u8, u8, u8) {
        let default_wounds_num = 3 + self.attr(Attr::Wound).val();
        let max_wounds = max(1, default_wounds_num) as u8;

        (self.pain, self.wounds, max_wounds)
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
        ActiveTraitIter(self.traits.values())
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
    pub to_hit: i8,
    pub to_wound: i8,
    pub attack_type: AttackType,
}

impl AttackOption {
    pub fn into_attack(self, a: &Actor) -> Attack {
        let (pain, wounds, max_wounds) = a.health();

        Attack {
            to_hit: a.attr(Attr::ToHit).modify(self.name.clone(), self.to_hit),
            to_wound: a
                .attr(Attr::ToWound)
                .modify(self.name.clone(), self.to_wound),
            name: self.name,
            attack_type: self.attack_type,
            num_dice: max(1, max_wounds.checked_sub(wounds + pain).unwrap_or(0)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Attack {
    pub name: DisplayStr,
    pub to_hit: AttrVal,
    pub to_wound: AttrVal,
    pub num_dice: u8,
    pub attack_type: AttackType,
}

#[derive(Debug, Clone)]
pub enum AttackType {
    Melee(String),
    Ranged(String),
}

#[derive(Debug, Clone)]
pub struct Defence {
    pub defence: AttrVal,
    pub defence_type: DefenceType,
    pub num_dice: u8,
}

#[derive(Debug, Clone)]
pub struct Wound {
    pain: u8,
    wound: u8,
}

impl Wound {
    // pub fn new(num_hits: u8) -> Self {
    //     match num_hits {
    //         0 => Self { pain: 0, wound: 0 },
    //         1 => Self { pain: 1, wound: 0 },
    //         n => Self {
    //             pain: n,
    //             wound: n -1,
    //         },
    //     }
    // }

    pub fn from_wound_roll(r: &Roll) -> Self {
        match r.successes() {
            0 => Self { pain: 0, wound: 0 },
            1 => Self { pain: 1, wound: 0 },
            n => Self {
                pain: n,
                wound: n - 1,
            },
        }
    }
}
