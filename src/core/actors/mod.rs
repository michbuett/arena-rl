mod actor;
mod generator;
mod traits;

pub use actor::{
    combat, Actor, ActorBuilder, AiBehaviour, Attack, AttackOption, CombatResult, Condition,
    Effect, Item, Look, Attr, Team, Trait, TraitSource,
};
pub use generator::{generate_enemy_easy, generate_player, generate_player2};

use crate::core::WorldPos;

/// Anything that exists in the world
#[derive(Debug, Clone)]
pub enum GameObject {
    Actor(Actor),
    Item(WorldPos, Item),
}
