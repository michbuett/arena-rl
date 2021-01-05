mod actor;
mod generator;

pub use generator::{generate_player, generate_enemy_easy};
pub use actor::{ActorBuilder, Actor, AiBehaviour, Team, Look, AttackOption, Attack, CombatResult, Condition, Item, combat, };

use crate::core::{WorldPos};

/// Anything that exists in the world
#[derive(Debug, Clone)]
pub enum GameObject {
    Actor(Actor),
    Item(WorldPos, Item),
}
