use std::collections::HashMap;

use specs::prelude::*;
use specs::World as SpecsWorld;

use crate::components::{GameObjectCmp, ObstacleCmp, Position};

use super::{Actor, GameObject, Map, MapPos, ID};

pub enum Change {
    Update(Entity, GameObject),
    Insert(GameObject),
    Remove(Entity),
}

pub struct CoreWorld<'a> {
    game_objects: ReadStorage<'a, GameObjectCmp>,
    obstacles: ReadStorage<'a, ObstacleCmp>,
    positions: ReadStorage<'a, Position>,
    entities: Entities<'a>,
    map: Read<'a, Map>,

    entity_map: HashMap<ID, Entity>,
    updates: HashMap<ID, Option<GameObject>>,
}

impl<'a> CoreWorld<'a> {
    pub fn new(w: &'a SpecsWorld) -> Self {
        let mut entity_map = HashMap::new();
        let (entities, game_objects, obstacles, positions, map): (
            Entities,
            ReadStorage<GameObjectCmp>,
            ReadStorage<ObstacleCmp>,
            ReadStorage<Position>,
            Read<Map>,
        ) = w.system_data();

        for (e, GameObjectCmp(go)) in (&entities, &game_objects).join() {
            entity_map.insert(go.id(), e);
        }

        Self {
            entities,
            game_objects,
            obstacles,
            positions,
            map,
            entity_map,
            updates: HashMap::new(),
        }
    }

    pub fn map(&self) -> &Map {
        &self.map
    }

    pub fn collect_obstacles(&self) -> HashMap<MapPos, (ObstacleCmp, Option<ID>)> {
        let mut result: HashMap<MapPos, (ObstacleCmp, Option<ID>)> = HashMap::new();

        for (e, o, p) in (&self.entities, &self.obstacles, &self.positions).join() {
            if let Some(GameObjectCmp(go)) = self.game_objects.get(e) {
                // obstacle is a game object which may have been changed
                // -> check for updates
                if self.updates.contains_key(&go.id()) {
                    if let Some(Some(go)) = self.updates.get(&go.id()) {
                        // there are updates for this object/Obstacle
                        // -> use the updated infos
                        result.insert(MapPos::from_world_pos(go.pos()), (o.clone(), Some(go.id())));
                    } else {
                        // object has been removed
                        // -> ignore
                    }
                } else {
                    // no update for this game object is known
                    // -> use the stored data
                    result.insert(MapPos::from_world_pos(p.0), (o.clone(), Some(go.id())));
                }
            } else {
                result.insert(MapPos::from_world_pos(p.0), (o.clone(), None));
            }
        }

        result
    }

    pub fn find_map<F, T>(&'a self, f: F) -> Option<T>
    where
        F: Fn(&GameObject) -> Option<T>,
    {
        for go in self.updates.values() {
            if let Some(go) = go {
                if let Some(v) = f(go) {
                    return Some(v);
                }
            }
        }

        for GameObjectCmp(go) in (&self.game_objects).join() {
            if self.updates.contains_key(&go.id()) {
                // an updated for of this entity has already been tested
                // -> do not return the old instance
                continue;
            }

            if let Some(v) = f(&go) {
                return Some(v);
            }
        }

        None
    }

    pub fn find_actor<P>(&self, predicate: P) -> Option<Actor>
    where
        P: Fn(&Actor) -> bool,
    {
        self.find_map(|go| {
            if let GameObject::Actor(a) = go {
                if predicate(a) {
                    return Some(a.clone());
                }
            }

            None
        })
    }

    pub fn get_actor(&self, id: ID) -> Option<&Actor> {
        if self.updates.contains_key(&id) {
            if let Some(Some(GameObject::Actor(a))) = self.updates.get(&id) {
                return Some(a);
            } else {
                return None;
            }
        }

        if let Some(e) = self.entity_map.get(&id) {
            if let Some(GameObjectCmp(GameObject::Actor(a))) = self.game_objects.get(*e) {
                return Some(a);
            }
        }

        None
    }

    pub fn remove(&mut self, id: ID) {
        self.updates.insert(id, None);
    }

    pub fn update(&mut self, o: GameObject) {
        self.updates.insert(o.id(), Some(o));
    }

    pub fn update_actor(&mut self, a: Actor) {
        self.update(GameObject::Actor(a));
    }

    pub fn modify_actor<F>(&mut self, a_id: ID, f: F)
    where
        F: Fn(Actor) -> Actor,
    {
        if let Some(a) = self.get_actor(a_id).map(|a| f(a.clone())) {
            self.update(GameObject::Actor(a));
        }
    }

    pub fn into_changes(self) -> Vec<Change> {
        let mut result = vec![];
        let mut updates = self.updates;

        for (id, go) in updates.drain() {
            match (go, self.entity_map.get(&id)) {
                (Some(go), Some(e)) => {
                    result.push(Change::Update(*e, go));
                }

                (Some(go), None) => {
                    result.push(Change::Insert(go));
                }

                (None, Some(e)) => {
                    result.push(Change::Remove(*e));
                }

                _ => {}
            }
        }

        result
    }
}
