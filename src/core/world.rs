use std::collections::HashMap;

use specs::join::JoinIter;
use specs::prelude::*;
use specs::World as SpecsWorld;

use crate::components::{ActorCmp, ObstacleCmp, Position};

use super::{
    flow::TeamSet, Actor, ActorTemplateName, Deck, Map, MapPos, ObjectGenerator, TeamId,
    TraitStorage, ID,
};

// #[derive(Debug, Clone)]
// pub enum Change {
//     Draws(HashMap<TeamId, Deck>),
//     Update(Entity, GameObject),
//     Insert(GameObject),
//     Remove(Entity),
// }

pub struct CoreWorld<'a> {
    world: &'a SpecsWorld,

    // world resources
    generator: Read<'a, ObjectGenerator>,
    decks: HashMap<TeamId, Deck>,
    decks_changed: bool,

    // component storages
    actors: ReadStorage<'a, ActorCmp>,

    entity_map: HashMap<ID, Entity>,
    updates: HashMap<ID, Option<Actor>>,
}

impl<'a> CoreWorld<'a> {
    pub fn new(w: &'a SpecsWorld) -> Self {
        let mut entity_map = HashMap::new();
        let (generator, entities, actors): (
            Read<ObjectGenerator>,
            Entities,
            ReadStorage<ActorCmp>,
        ) = w.system_data();

        for (e, ActorCmp(a)) in (&entities, &actors).join() {
            entity_map.insert(a.id, e);
        }

        let teams: Read<TeamSet> = w.read_resource::<TeamSet>().into();

        Self {
            world: w,
            decks: teams.decks(),
            decks_changed: false,
            generator,
            actors,
            entity_map,
            updates: HashMap::new(),
        }
    }

    pub fn map(&self) -> Read<Map> {
        self.world.read_resource::<Map>().into()
    }

    pub fn teams(&self) -> Read<TeamSet> {
        self.world.read_resource::<TeamSet>().into()
    }

    pub fn decks_mut(&mut self) -> &mut HashMap<TeamId, Deck> {
        self.decks_changed = true;
        &mut self.decks
    }

    pub fn traits(&self) -> &TraitStorage {
        &self.generator.traits()
    }

    pub fn collect_obstacles(&self) -> HashMap<MapPos, (ObstacleCmp, Option<ID>)> {
        let mut result: HashMap<MapPos, (ObstacleCmp, Option<ID>)> = HashMap::new();
        let (entities, obstacles, positions): (
            Entities,
            ReadStorage<ObstacleCmp>,
            ReadStorage<Position>,
        ) = self.world.system_data();

        for (e, o, p) in (&entities, &obstacles, &positions).join() {
            if let Some(ActorCmp(a)) = self.actors.get(e) {
                // obstacle is a game object which may have been changed
                // -> check for updates
                if self.updates.contains_key(&a.id) {
                    if let Some(Some(a)) = self.updates.get(&a.id) {
                        // there are updates for this object/Obstacle
                        // -> use the updated infos
                        result.insert(MapPos::from_world_pos(p.0), (o.clone(), Some(a.id)));
                    } else {
                        // object has been removed
                        // -> ignore
                    }
                } else {
                    // no update for this game object is known
                    // -> use the stored data
                    result.insert(MapPos::from_world_pos(p.0), (o.clone(), Some(a.id)));
                }
            } else {
                result.insert(MapPos::from_world_pos(p.0), (o.clone(), None));
            }
        }

        result
    }

    pub fn find_map<F, T>(&'a self, f: F) -> Option<T>
    where
        F: Fn(&Actor) -> Option<T>,
    {
        for go in self.updates.values() {
            if let Some(go) = go {
                if let Some(v) = f(go) {
                    return Some(v);
                }
            }
        }

        for ActorCmp(a) in (&self.actors).join() {
            if self.updates.contains_key(&a.id) {
                // an updated for of this entity has already been tested
                // -> do not return the old instance
                continue;
            }

            if let Some(v) = f(&a) {
                return Some(v);
            }
        }

        None
    }

    pub fn find_actor<P>(&self, predicate: P) -> Option<Actor>
    where
        P: Fn(&Actor) -> bool,
    {
        self.find_map(|a| {
            if predicate(a) {
                return Some(a.clone());
            }

            None
        })
    }

    pub fn get_actor(&self, id: ID) -> Option<&Actor> {
        if self.updates.contains_key(&id) {
            return self.updates.get(&id).unwrap().as_ref();
        }

        if let Some(e) = self.entity_map.get(&id) {
            if let Some(ActorCmp(a)) = self.actors.get(*e) {
                return Some(a);
            }
        }

        None
    }

    pub fn remove(&mut self, id: ID) {
        self.updates.insert(id, None);
    }

    pub fn update(&mut self, a: Actor) {
        self.updates.insert(a.id, Some(a));
    }

    pub fn modify_actor<F>(&mut self, a_id: ID, f: F)
    where
        F: Fn(Actor) -> Actor,
    {
        if let Some(a) = self.get_actor(a_id).map(|a| f(a.clone())) {
            self.update(a);
        }
    }

    pub fn generate_enemy(&self, pos: MapPos, team: TeamId, template: ActorTemplateName) -> Actor {
        self.generator
            .generate_enemy(pos.to_world_pos(), team, template)
    }

    pub fn into_changes(mut self) -> (Option<HashMap<TeamId, Deck>>, Vec<(ID, Option<Actor>)>) {
        let decks = if self.decks_changed {
            Some(self.decks)
        } else {
            None
        };

        let updates = self.updates.drain().collect::<Vec<_>>();

        (decks, updates)
        // let updates = self.updates.values().drain(..).collect::<Vec<_>>();
    }

    pub fn actors<'b>(&'b self) -> GameObjectIterator<'b> {
        GameObjectIterator {
            actors: self.actors.join(),
            updates: &self.updates,
        }
    }
}

pub struct GameObjectIterator<'a> {
    actors: JoinIter<&'a ReadStorage<'a, ActorCmp>>,
    updates: &'a HashMap<ID, Option<Actor>>,
}

impl<'a> Iterator for GameObjectIterator<'a> {
    type Item = &'a Actor;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(ActorCmp(a)) = self.actors.next() {
            if self.updates.contains_key(&a.id) {
                return self.updates.get(&a.id).unwrap().as_ref();
            } else {
                return Some(&a);
            }
        }

        None
    }
}
