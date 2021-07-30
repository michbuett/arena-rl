mod actor;
mod generator;
mod combat;
mod traits;

pub use actor::*;
pub use combat::*;
pub use generator::*;

use crate::core::WorldPos;

/// Anything that exists in the world
#[derive(Debug, Clone)]
pub enum GameObject {
    Actor(Actor),
    Item(WorldPos, Item),
}
