mod actor;
mod combat;
mod generator;
mod traits;

pub use actor::*;
pub use combat::*;
pub use generator::*;

use crate::core::WorldPos;
use std::hash::{Hash, Hasher};

/// Anything that exists in the world
#[derive(Debug, Clone)]
pub enum GameObject {
    Actor(Actor),
    Item(WorldPos, Item),
}

impl GameObject {
    pub fn id(&self) -> ID {
        match self {
            GameObject::Actor(a) => a.id,
            GameObject::Item(_, i) => i.id,
        }
    }

    pub fn pos(&self) -> WorldPos {
        match self {
            GameObject::Actor(a) => a.pos,
            GameObject::Item(p, _) => *p,
        }
    }

    // pub fn dbg_info(&self) -> String {
    //     match self {
    //         GameObject::Actor(a) => format!("{} at {:?}", a.name, a.pos),
    //         GameObject::Item(p, _) => format!("(item) at {:?}", p),
    //     }
    // }
}

impl Hash for GameObject {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id().hash(state);
    }
}

impl Into<GameObject> for Actor {
    fn into(self) -> GameObject {
        GameObject::Actor(self)
    }
}
