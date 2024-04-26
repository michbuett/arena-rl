use std::cmp::max;
use std::collections::HashMap;
use std::time::Instant;

pub use super::traits::*;
use super::ActorTemplateName;

use crate::core::{Card, DisplayStr, MapPos, Suite, WorldPos};

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

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct TeamId(u8);

impl TeamId {
    pub fn new(raw_id: u8) -> Self {
        Self(raw_id)
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Team {
    pub name: &'static str,
    pub id: TeamId,
    pub is_pc: bool,
    pub reinforcements: Option<Vec<(u64, MapPos, ActorTemplateName)>>,
}

impl Team {
    pub fn is_member(&self, a: &Actor) -> bool {
        self.id == a.team
    }
}

// #[derive(Debug, Clone, Copy, Eq, PartialEq)]
// pub enum ActorType {
//     Tank,
//     Saw,
//     Spear,
//     Healer,
//     Gunner,
//     MonsterSucker,
//     MonsterWorm,
//     MonsterZombi,
// }

pub struct ActorBuilder {
    behaviour: Option<AiBehaviour>,
    pos: WorldPos,
    team: TeamId,
    max_activations: u8,
    visual: Visual,
    name: String,
    keywords: Vec<Keyword>,
    traits: HashMap<String, Trait>,
}

impl ActorBuilder {
    pub fn new(name: String, pos: WorldPos, team: TeamId, max_activations: u8) -> Self {
        Self {
            pos,
            team,
            name,
            max_activations,
            behaviour: None,
            visual: Visual::new(VisualElements::empty()),
            keywords: vec![],
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
            keywords: self.keywords,
            effects: Vec::new(),
            traits: self.traits,
            behaviour: self.behaviour,
            team: self.team,
            visual: self.visual,
            max_activations: self.max_activations,
            activations: vec![],
            active_activation: None,
        }
        .process_traits();

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

    pub fn visual(self, visual: Visual) -> Self {
        Self { visual, ..self }
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

#[derive(Debug, Clone)]
pub struct VisualElements([Option<String>; NUM_VISUAL_LAYERS]);

impl VisualElements {
    pub fn new(elements: Vec<(VLayers, String)>) -> Self {
        let mut result = Self::empty();
        for (layer, name) in elements {
            result.set(layer, name);
        }
        result
    }

    pub fn empty() -> Self {
        Self(Default::default())
    }

    fn set(&mut self, layer: VLayers, name: impl ToString) {
        self.0[layer as usize] = Some(name.to_string());
    }

    fn iter(&self) -> VisElIter {
        VisElIter(self.0.iter())
    }
}

struct VisElIter<'a>(std::slice::Iter<'a, Option<String>>);

impl<'a> Iterator for VisElIter<'a> {
    type Item = &'a String;

    fn next(&mut self) -> Option<&'a String> {
        let mut next = self.0.next();

        while let Some(v) = next {
            if v.is_some() {
                // current layer is set
                // => return it
                return v.as_ref();
            } else {
                // current layer is not set
                // => try the next one
                next = self.0.next();
            }
        }

        // no more items to try
        // => stop iterating
        None
    }
}

#[derive(Debug, Clone)]
pub struct Visual {
    states: [Option<VisualElements>; NUM_VISUAL_STATES],
}

impl Visual {
    pub fn new(default: VisualElements) -> Self {
        let mut states: [Option<VisualElements>; NUM_VISUAL_STATES] = Default::default();
        states[VisualState::Idle as usize] = Some(default);

        Self { states }
    }

    pub fn add_state(mut self, state: VisualState, el: VisualElements) -> Self {
        self.states[state as usize] = Some(el);
        self
    }

    fn add_elements(mut self, state: VisualState, layer: VLayers, name: String) -> Self {
        if self.states[state as usize].is_none() {
            self.states[state as usize] = Some(VisualElements::empty());
        }

        self.states[state as usize]
            .as_mut()
            .unwrap()
            .set(layer, name);
        self
    }

    pub fn get_state(&self, state: VisualState) -> impl Iterator<Item = &String> {
        if let Some(ve) = &self.states[state as usize] {
            return ve.iter();
        }

        self.states[VisualState::Idle as usize]
            .as_ref()
            .unwrap()
            .iter()
    }
}

#[derive(Debug, Clone)]
pub struct Actor {
    traits: HashMap<String, Trait>,
    visual: Visual,
    keywords: Vec<Keyword>,
    max_activations: u8,

    pub id: ID,
    pub health: Health,
    pub effects: Vec<(DisplayStr, Effect)>,
    pub name: String,
    pub active: bool,
    pub team: TeamId,
    pub pos: WorldPos,
    pub activations: Vec<Card>,
    pub active_activation: Option<Card>,
    pub behaviour: Option<AiBehaviour>,
}

impl Actor {
    ////////////////////////////////////////////////////////////
    // Movement
    pub fn can_move(&self) -> bool {
        self.is_concious()
    }

    pub fn move_distance(&self) -> u8 {
        let move_mod = self.attr(Attr::Movement).val();
        max(1, 3 + move_mod) as u8
    }

    pub fn is_flying(&self) -> bool {
        self.is_concious() && self.keywords.contains(&Keyword::Flying)
    }

    pub fn is_underground(&self) -> bool {
        self.is_concious() && self.keywords.contains(&Keyword::Underground)
    }

    ////////////////////////////////////////////////////////////
    // A.I.

    pub fn is_pc(&self) -> bool {
        self.behaviour.is_none()
    }

    ////////////////////////////////////////////////////////////
    // Activations

    pub fn can_activate(&self) -> bool {
        self.is_alive()
    }

    /// Returns the number of activations this actor gets by default
    pub fn num_activation(&self) -> u8 {
        self.max_activations
    }

    pub fn assigne_activation(mut self, card: Card) -> Self {
        self.activations.push(card);
        self
    }

    pub fn initiative(&self) -> u8 {
        self.activations
            .iter()
            .map(|c| c.value)
            .min()
            .unwrap_or(u8::MAX)
    }

    pub fn activate(mut self) -> Self {
        debug_assert!(
            !self.activations.is_empty(),
            "Activating an actor who has no more activations"
        );

        self.active = true;

        let i = self.initiative();
        let min_activation_idx = self.activations.iter().position(|c| c.value == i).unwrap();

        self.active_activation = Some(self.activations.swap_remove(min_activation_idx));
        self
    }

    pub fn deactivate(self) -> Self {
        Self {
            active: false,
            ..self
        }
    }

    // pub fn use_ability(self, key: impl ToString, ability: Trait) -> Self {
    //     let msg = format!("Used ability {}", ability.name);

    //     self.add_trait(key.to_string(), ability)
    //         .prepare(Act::done(msg))
    // }

    ////////////////////////////////////////////////////////////
    // Handle turn cycle

    pub fn done(mut self) -> Actor {
        self.active = false;
        self
    }

    pub fn start_next_turn(mut self) -> Actor {
        // handle temporary traits
        let mut new_traits = HashMap::new();
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
            traits: new_traits,
            ..self
        }
        .process_traits()
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
                        ap,
                        rend,
                        fx,
                        effects,
                    } => Some(AttackOption {
                        name: name.clone(),
                        min_distance: 1,
                        max_distance: max(1, distance.unwrap_or(1)),
                        advance: advance.unwrap_or(0),
                        to_hit: to_hit.unwrap_or(0),
                        to_wound: ap.unwrap_or(0),
                        rend: rend.unwrap_or(0),
                        advantage: 0,
                        attack_type: AttackType::Melee(fx.to_string()),
                        required_effort: *required_effort,
                        allocated_effort: 0,
                        effects: effects.clone(),
                        to_hit_threshold: 4, // TODO calculate from to-hit modifier and defence
                    }),

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
                        rend: 0,
                        advantage: 0,
                        attack_type: AttackType::Ranged(fx.to_string()),
                        required_effort: 3, // TODO read from effect
                        allocated_effort: 0,
                        effects: None, // TODO read from effect
                        to_hit_threshold: 4,
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
                rend: -1,
                advantage: 0,
                attack_type: AttackType::Melee("fx-hit-1".to_string()),
                required_effort: 2,
                allocated_effort: 0,
                effects: None,
                to_hit_threshold: 4,
            }]
        } else {
            attacks
        }
    }

    fn process_traits(mut self) -> Self {
        let mut effects = vec![];
        let mut keywords = vec![];

        for t in self.traits.values() {
            for e in t.effects.iter() {
                match e {
                    Effect::Keyword(k) => {
                        keywords.push(k.clone());
                    }
                    _ => {
                        effects.push((t.name.clone(), e.clone()));
                    }
                }
            }

            if let Some(visuals) = &t.visuals {
                for (vstate, velements) in visuals {
                    for (l, n) in velements {
                        self.visual = self.visual.add_elements(*vstate, *l, n.to_string());
                    }
                }
            }
        }

        Self {
            effects,
            keywords,
            ..self
        }
    }

    pub fn add_trait(mut self, key: String, new_trait: Trait) -> Self {
        self.traits.insert(key, new_trait);
        self.process_traits()
    }

    // pub fn remove_trait(mut self, key: &str) -> Self {
    //     if let Some(_) = self.traits.remove(key) {
    //         self.process_traits()
    //     } else {
    //         self
    //     }
    // }

    pub fn wound(mut self, w: Wound) -> Self {
        self.health = self.health.wound(w);

        if self.is_concious() {
            self
        } else {
            // K.O. => cancel any prepared action and skip turn
            self.done()
        }
    }

    pub fn is_alive(&self) -> bool {
        self.health.remaining_wounds > 0
    }

    pub fn is_concious(&self) -> bool {
        self.is_alive() && self.health.remaining_wounds >= self.health.pain
    }

    // pub fn corpse(&self) -> Item {
    //     Item {
    //         id: ID::new(),
    //         name: format!("Corpse of {}", self.name),
    //         look: vec![(1, "corpses_1".to_string())],
    //     }
    // }

    pub fn visuals(&self) -> impl Iterator<Item = &String> {
        if self.is_alive() {
            if !self.is_concious() {
                return self.visual.get_state(VisualState::Prone);
            }

            if self.is_underground() {
                return self.visual.get_state(VisualState::Hidden);
            }
        } else {
            return self.visual.get_state(VisualState::Dead);
        }

        self.visual.get_state(VisualState::Idle)
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
    }

    pub fn skill(&self, _s: Suite) -> u8 {
        5 // TODO make configurable
    }

    pub fn soak(&self) -> u8 {
        5 // TODO make configurable
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

    fn wound(mut self, w: Wound) -> Self {
        self.recieved_wounds += w.wound;
        self.remaining_wounds = self.remaining_wounds.checked_sub(w.wound).unwrap_or(0);
        self.pain += w.pain;
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

#[derive(Debug, Clone)]
pub struct AttackOption {
    pub name: DisplayStr,
    pub min_distance: u8,
    pub max_distance: u8,
    pub advance: u8,
    pub to_hit: i8,
    pub to_wound: i8,
    pub rend: i8,
    pub advantage: i8,
    pub attack_type: AttackType,
    pub required_effort: u8,
    pub allocated_effort: u8,
    pub effects: Option<Vec<(HitEffectCondition, HitEffect)>>,
    pub to_hit_threshold: u8,
}

impl AttackOption {
    pub fn into_attack(self, a: &Actor) -> Attack {
        let to_hit = match self.attack_type {
            AttackType::Melee(..) => a.attr(Attr::MeleeSkill),
            AttackType::Ranged(..) => a.attr(Attr::RangedSkill),
        }
        .modify(self.name.clone(), self.to_hit);

        let to_wound = a
            .attr(Attr::ArmorPenetration)
            .modify(self.name.clone(), self.to_wound);

        let num_dice = self.required_effort;

        // let advantage = if self.required_effort > a.available_effort() {
        //     -1
        // } else {
        //     0
        // };

        let challenge_suite = match self.attack_type {
            AttackType::Melee(..) => Suite::Clubs,
            AttackType::Ranged(..) => Suite::Spades,
        };

        Attack {
            origin_pos: a.pos,
            to_hit,
            to_wound,
            rend: self.rend,
            name: self.name,
            attack_type: self.attack_type,
            num_dice,
            advantage: 0,
            effects: self.effects,
            challenge_suite,
            effort_card: a.active_activation.unwrap(),
            challenge_value: 10, // TODO make configurable
            damage: 5,           // TODO make configurable
        }
    }
}

#[derive(Debug, Clone)]
pub struct Attack {
    pub origin_pos: WorldPos,
    pub name: DisplayStr,
    pub to_hit: AttrVal,
    pub to_wound: AttrVal,
    pub rend: i8,
    pub num_dice: u8,
    pub attack_type: AttackType,
    pub advantage: i8,
    pub effects: Option<Vec<(HitEffectCondition, HitEffect)>>,

    pub effort_card: Card,
    pub challenge_value: u8,
    pub challenge_suite: Suite,
    pub damage: u8,
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
    pub pain: u8,
    pub wound: u8,
}
