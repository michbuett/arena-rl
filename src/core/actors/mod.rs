mod actor;
mod generator;
mod traits;

pub use actor::*;
pub use generator::*;

use crate::core::WorldPos;

/// Anything that exists in the world
#[derive(Debug, Clone)]
pub enum GameObject {
    Actor(Actor),
    Item(WorldPos, Item),
}
