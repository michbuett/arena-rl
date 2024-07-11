use std::cmp::{max, min};
use std::collections::HashMap;
use std::time::Instant;

use serde::Deserialize;

pub use super::traits::AttributeModifier::*;
pub use super::traits::*;

use super::ActorTemplateName;

use crate::core::{Card, Deck, DisplayStr, MapPos, Suite, WorldPos};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct ID(Instant, u64, u64);

impl ID {
    pub fn new() -> Self {
        use rand::random;
        Self(Instant::now(), random(), random())
    }
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
    attributes: ActorAttriubes,
    traits: HashMap<String, Trait>,
}

impl ActorBuilder {
    pub fn new(
        name: String,
        pos: WorldPos,
        team: TeamId,
        attributes: ActorAttriubes,
        max_activations: u8,
    ) -> Self {
        Self {
            pos,
            team,
            name,
            attributes,
            max_activations,
            behaviour: None,
            visual: Visual::new(VisualElements::empty()),
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
            keywords: 0,
            effects: Vec::new(),
            attributes: self.attributes,
            modifier: Default::default(),
            traits: self.traits,
            behaviour: self.behaviour,
            team: self.team,
            visual: self.visual,
            max_activations: self.max_activations,
            activations: vec![],
            active_activation: None,
        }
        .process_traits();

        a.health = Health::new(a.skill(Suite::PhysicalStr, 0));
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
pub enum Activation {
    Single(Card),
    Boosted(Card, Card),
    // Hindered(Card, Card),
}

impl Activation {
    pub fn initiativ(&self) -> u8 {
        match self {
            Activation::Single(c) => c.value,
            Activation::Boosted(c1, c2) => min(c1.value, c2.value),
        }
    }

    pub fn value_card(&self, suite: Suite) -> Card {
        match self {
            Activation::Single(c) => *c,
            Activation::Boosted(c1, c2) => {
                if c1.value(suite) >= c2.value(suite) {
                    *c1
                } else {
                    *c2
                }
            }
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ActorAttriubes {
    physical_strength: u8,
    physical_agility: u8,
    mental_strength: u8,
    mental_agility: u8,
}

#[derive(Debug, Clone)]
pub struct Actor {
    traits: HashMap<String, Trait>,
    visual: Visual,
    keywords: u64,
    max_activations: u8,
    attributes: ActorAttriubes,
    modifier: [i8; NUM_ATTRIBUTE_MODIFIER],

    pub id: ID,
    pub health: Health,
    pub effects: Vec<(DisplayStr, Effect)>,
    pub name: String,
    pub active: bool,
    pub team: TeamId,
    pub pos: WorldPos,
    pub activations: Vec<Activation>,
    pub active_activation: Option<Activation>,
    pub behaviour: Option<AiBehaviour>,
}

impl Actor {
    ////////////////////////////////////////////////////////////
    // Movement
    pub fn can_move(&self) -> bool {
        self.is_concious()
    }

    pub fn move_distance(&self) -> u8 {
        self.attr_value(3, MoveDistance, 0)
    }

    pub fn is_flying(&self) -> bool {
        self.is_concious() && self.has_keyword(Keyword::Flying)
    }

    pub fn is_underground(&self) -> bool {
        self.is_concious() && self.has_keyword(Keyword::Underground)
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

    pub fn add_activation(mut self, new_card: Card) -> Self {
        debug_assert!(
            self.activations.len() < self.max_activations as usize + 1,
            "It is not allowed to assign this activation {:?}",
            new_card
        );
        self.activations.push(Activation::Single(new_card));
        self.activations.sort_by_key(|a| a.initiativ());
        self
    }

    pub fn boost_activation(mut self, new_card: Card) -> Self {
        if let Some((idx, activation)) = self.activations.iter().enumerate().find_map(|(idx, a)| {
            if let Activation::Boosted(..) = a {
                return None;
            } else {
                return Some((idx, a));
            }
        }) {
            match activation {
                Activation::Single(curr_card) => {
                    self.activations[idx] = Activation::Boosted(*curr_card, new_card);
                }
                Activation::Boosted(..) => {
                    // ignore this case (we filtered for non-boosted activation)
                    // case is explicit to help the compiler identify new types
                    // of activations
                }
            }

            self.activations.sort_by_key(|a| a.initiativ());
            self
        } else {
            // no existing non-boosted activation found
            // => add a new one (only one bonus activation is allowed)
            self.add_activation(new_card)
        }
    }

    pub fn initiative(&self) -> u8 {
        self.activations
            .first()
            .map(|a| a.initiativ())
            .unwrap_or(u8::MAX)
    }

    pub fn activate(mut self) -> Self {
        debug_assert!(
            !self.activations.is_empty(),
            "Activating an actor who has no more activations"
        );

        self.active = true;
        self.active_activation = Some(self.activations.remove(0));
        self
    }

    pub fn deactivate(self) -> Self {
        Self {
            active: false,
            ..self
        }
    }

    ////////////////////////////////////////////////////////////
    // Handle turn cycle

    pub fn done(mut self) -> Actor {
        self.active = false;
        self
    }

    pub fn start_next_turn(mut self, deck: &mut Deck) -> Actor {
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

        let mut result = Self {
            traits: new_traits,
            ..self
        }
        .process_traits();

        for _ in 1..=result.num_activation() {
            let card = if result.has_keyword(Keyword::Quick) {
                // Quick actors draw two cards and discard the higher one
                // (lower cards act faster)
                let (c1, c2) = (deck.deal(), deck.deal());
                if c1.value < c2.value {
                    c1
                } else {
                    c2
                }
            } else if result.has_keyword(Keyword::Slow) {
                // Slow actors draw two cards and discard the lower one
                let (c1, c2) = (deck.deal(), deck.deal());
                if c1.value > c2.value {
                    c1
                } else {
                    c2
                }
            } else {
                deck.deal()
            };

            result = result.add_activation(card);
        }

        result
    }

    pub fn attacks(&self) -> Vec<AttackOption> {
        let attacks = self
            .effects
            .iter()
            .filter_map(|(_, eff)| match eff {
                Effect::AttackSingleTarget {
                    name,
                    challenge_value,
                    to_hit,
                    to_wound,
                    defence,
                    distance_min,
                    distance_max,
                    rend,
                    fx,
                    effects,
                } => Some(AttackOption {
                    name: name.clone(),
                    to_hit: *to_hit,
                    to_wound: *to_wound,
                    defence: *defence,
                    challenge_value: *challenge_value,
                    min_distance: distance_min.unwrap_or(0),
                    max_distance: distance_max.unwrap_or(1),
                    advance: 0,
                    rend: rend.unwrap_or(0),
                    attack_fx: fx.clone(),
                    effects: effects.clone(),
                }),

                _ => None,
            })
            .collect::<Vec<_>>();

        if attacks.is_empty() {
            vec![AttackOption {
                name: DisplayStr::new("Unarmed attack"),
                min_distance: 0,
                max_distance: 1,
                to_hit: (Suite::PhysicalStr, 0),
                to_wound: (Suite::PhysicalStr, 0),
                defence: Suite::Physical,
                challenge_value: 7,
                advance: 0,
                rend: 0,
                attack_fx: AttackFx::MeleeSingleTarget {
                    name: "fx-hit-1".to_string(),
                },
                effects: None,
            }]
        } else {
            attacks
        }
    }

    fn process_traits(mut self) -> Self {
        let mut effects = vec![];
        let mut keywords = 0;

        for t in self.traits.values() {
            for e in t.effects.iter() {
                match e {
                    Effect::Keyword(k) => keywords = keywords | k.as_bit(),
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

    pub fn has_keyword(&self, kw: Keyword) -> bool {
        self.keywords & kw.as_bit() > 0
    }

    pub fn add_trait(mut self, key: String, new_trait: Trait) -> Self {
        self.traits.insert(key, new_trait);
        self.process_traits()
    }

    pub fn active_traits(&self) -> ActiveTraitIter {
        ActiveTraitIter(self.traits.values())
    }

    ////////////////////////////////////////////////////////////
    // Health

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

    /// Returns the active value for a given attribute
    ///  0 => None
    ///  1 => Puny
    ///  2 => Low — rusty
    ///  3 => Average
    ///  4 => Good — trained (decent)
    ///  5 => Very good
    ///  6 => Elite (only the best have elite stats)
    ///  7 => Exceptional (once per generagion; the best of the best)
    ///  8 => Legendary (once per era)
    ///  9 => Supernatural
    /// 10 => Godlike (unlimited power)
    pub fn skill(&self, s: Suite, modifier: i8) -> u8 {
        let a = &self.attributes;

        match s {
            Suite::PhysicalStr => self.attr_value(a.physical_strength, PhysicalStrength, modifier),
            Suite::PhysicalAg => self.attr_value(a.physical_agility, PhysicalAgility, modifier),
            Suite::MentalStr => self.attr_value(a.mental_strength, MentalStrength, modifier),
            Suite::MentalAg => self.attr_value(a.mental_agility, MentalAgility, modifier),
            Suite::Physical => max(
                self.skill(Suite::PhysicalStr, modifier),
                self.skill(Suite::PhysicalAg, modifier),
            ),
            Suite::Mental => max(
                self.skill(Suite::MentalStr, modifier),
                self.skill(Suite::MentalAg, modifier),
            ),
            Suite::Strength => max(
                self.skill(Suite::PhysicalStr, modifier),
                self.skill(Suite::MentalStr, modifier),
            ),
            Suite::Agility => max(
                self.skill(Suite::PhysicalAg, modifier),
                self.skill(Suite::MentalAg, modifier),
            ),
            Suite::Any => max(
                self.skill(Suite::Physical, modifier),
                self.skill(Suite::Mental, modifier),
            ),
        }
    }

    fn attr_value(&self, base_val: u8, attr_mod: AttributeModifier, skill_mod: i8) -> u8 {
        let attr_val = base_val as i8 + self.modifier[attr_mod as usize] + skill_mod;
        max(0, attr_val) as u8
    }

    pub fn soak(&self) -> u8 {
        self.attr_value(self.attributes.physical_strength, PhysicalResistence, 0)
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
    pub advance: u8,
    pub to_hit: (Suite, i8),
    pub to_wound: (Suite, i8),
    pub attack_fx: AttackFx,
    pub challenge_value: u8,
    pub defence: Suite,
    pub effects: Option<Vec<(HitEffectCondition, HitEffect)>>,
    pub max_distance: u8,
    pub min_distance: u8,
    pub name: DisplayStr,
    pub rend: u8,
}

impl AttackOption {
    pub fn into_attack(self, a: &Actor) -> Attack {
        let effort_card = a
            .active_activation
            .as_ref()
            .unwrap()
            .value_card(self.to_hit.0);

        Attack {
            origin_pos: a.pos,
            rend: self.rend,
            name: self.name,
            attack_fx: self.attack_fx,
            advantage: 0,
            effects: self.effects,
            effort_card,
            to_hit: self.to_hit,
            to_wound: self.to_wound,
            challenge_value: self.challenge_value,
            defence: self.defence,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Attack {
    pub origin_pos: WorldPos,
    pub name: DisplayStr,
    pub rend: u8,
    pub attack_fx: AttackFx,
    pub advantage: i8,
    pub effects: Option<Vec<(HitEffectCondition, HitEffect)>>,

    pub effort_card: Card,
    pub challenge_value: u8,
    pub to_hit: (Suite, i8),
    pub to_wound: (Suite, i8),
    pub defence: Suite,
}

#[derive(Debug, Clone)]
pub struct Wound {
    pub pain: u8,
    pub wound: u8,
}
