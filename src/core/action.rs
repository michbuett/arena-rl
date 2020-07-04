use std::time::Duration;

use specs::prelude::Entity;
// use specs::prelude::*;

use crate::components::Fx;
use crate::core::Tile;
// use crate::core::{Tile, WorldPos};

use super::actors::{Actor, Attack, CombatResult, Defence, GameObject, Condition};

#[derive(Debug, Clone)]
pub enum Action {
    Wait(u8),
    MoveTo(u8, Tile),
    Attack(EA, Attack),
    Defence(EA, EA, Attack, Defence),
    EndTurn(u64), // next turn
}

impl Action {
    pub fn wait(costs: u8) -> Self {
        Self::Wait(costs)
    }

    pub fn move_to(costs: u8, to: Tile) -> Self {
        Self::MoveTo(costs, to)
    }

    pub fn attack(target: EA, attack: Attack) -> Self {
        Self::Attack(target, attack)
    }

    pub fn defence(attacker: EA, target: EA, a: Attack, d: Defence) -> Self {
        Self::Defence(attacker, target, a, d)
    }
}

#[derive(Debug, Clone)]
pub enum Opportunity {
    IncommingAttack(EA, Attack),
}

pub enum Change {
    Fx(Fx),
    Update(Entity, GameObject),
    Insert(GameObject),
    Remove(Entity),
}

pub type EA = (Entity, Actor);
pub type Reaction = (EA, Opportunity);
pub type ActionResult = (Vec<Change>, Duration, Vec<Reaction>);

pub fn run_action<'a>((entity, actor): EA, action: Action) -> ActionResult {
    match action {
        Action::Wait(costs) => (
            vec![update_actor(entity, actor.done(costs))],
            Duration::new(0, 0),
            vec![],
        ),

        Action::MoveTo(costs, to) => (
            vec![update_actor(
                entity,
                actor.move_to(to.to_world_pos()).done(costs),
            )],
            millis(200),
            vec![],
        ),

        Action::Attack(target, attack) => {
            init_attack((entity, actor), target, attack)
        }

        Action::Defence(attacker, target, attack, defence) => {
            resolve_attack(attacker, target, attack, defence)
        }
        
        Action::EndTurn(next_turn) => {
            handle_end_turn(entity, actor, next_turn)
        }
    }
}

fn init_attack<'a>(
    attacker: (Entity, Actor),
    target: (Entity, Actor),
    attack: Attack,
) -> ActionResult {

    return (
        vec![update_actor(
            attacker.0,
            attacker.1.clone().done(attack.costs),
        )],
        millis(0),
        vec![(
            target,
            Opportunity::IncommingAttack(attacker.clone(), attack),
        )],
    );
}

fn resolve_attack<'a>(
    _attacker: (Entity, Actor), // TODO: modify attacker (e.g. critical miss)
    target: (Entity, Actor),
    attack: Attack,
    defence: Defence,
) -> ActionResult {
    let fx_pos = target.1.pos.clone();
    let combat_result = super::actors::combat(attack, defence, target.1);
    let changes = match combat_result {
        CombatResult::Strike(new_actor) => vec!(
            update_actor(target.0, new_actor),
            Change::Fx(Fx::text("Strike".to_string(), &fx_pos, 100)),
        ),

        CombatResult::Hit(condition) => {
            match condition {
                Condition::Alive(new_actor) => vec!(
                    update_actor(target.0, new_actor),
                    Change::Fx(Fx::text("Hit!".to_string(), &fx_pos, 100)),
                ),

                Condition::Dead(pos, corpse) => vec!(
                    Change::Remove(target.0),
                    Change::Insert(GameObject::Item(pos, corpse)),
                    Change::Fx(Fx::text("KILL!!!".to_string(), &fx_pos, 100)),
                ),
            }
        }
    };

    (changes, millis(1000), Vec::new())
}

fn handle_end_turn(e: Entity, a: Actor, next_turn: u64) -> ActionResult {
    let condition = a.next_turn(next_turn);
    let changes = match condition {
        Condition::Alive(actor) =>
            vec!(update_actor(e, actor)),

        Condition::Dead(pos, corpse) => vec!(
            Change::Remove(e),
            Change::Insert(GameObject::Item(pos, corpse))
        ),
    };
    
    (changes, millis(0), Vec::new())
}

fn millis(ms: u64) -> Duration {
    Duration::from_millis(ms)
}

fn update_actor(e: Entity, a: Actor) -> Change {
    Change::Update(e, GameObject::Actor(a))
}
