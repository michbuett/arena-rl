use std::time::Duration;

use specs::prelude::Entity;
// use specs::prelude::*;

use crate::components::Fx;
use crate::core::Tile;
// use crate::core::{Tile, WorldPos};

use super::actors::{Actor, AttackOption, CombatResult, GameObject, Condition};

#[derive(Debug, Clone)]
pub enum Action {
    StartTurn(),
    Wait(u8),
    MoveTo(Tile),
    Activate(),
    Attack(EA, AttackOption),
    // Defence(EA, EA, Attack, Defence),
    // EndTurn(u64), // next turn
}

impl Action {
    pub fn start_turn() -> Act {
        (Self::StartTurn(), 0)
    }

    pub fn wait(costs: u8) -> Act {
        (Self::Wait(costs), 1)
    }

    pub fn activate() -> Act {
        (Self::Activate(), 0)
    }

    pub fn move_to(to: Tile) -> Act {
        (Self::MoveTo(to), 0)
    }

    pub fn attack(target: EA, attack: AttackOption) -> Act {
        (Self::Attack(target, attack), 1)
    }

    // pub fn defence(attacker: EA, target: EA, a: Attack, d: Defence) -> Act {
    //     (Self::Defence(attacker, target, a, d), 0)
    // }
}

#[derive(Debug, Clone)]
pub enum Opportunity {
    IncommingAttack(),
}

pub enum Change {
    Fx(Fx),
    Update(Entity, GameObject),
    Insert(GameObject),
    Remove(Entity),
}

pub type Act = (Action, u8);
pub type EA = (Entity, Actor);
pub type Reaction = (EA, Opportunity);
pub type ActionResult = (Vec<Change>, Duration); 

pub fn act((entity, actor, action, delay): (Entity, Actor, Action, u8)) -> ActionResult {
    if delay > 0 {
        (
            vec![update_actor(entity, actor.prepare((action, delay - 1)))],
            millis(0),
        )
    } else {
        run_action((entity, actor), action)
    }
}

pub fn run_action<'a>((entity, actor): EA, action: Action) -> ActionResult {
    match action {
        Action::StartTurn() => {
            let (actor, pending_action) = actor.start_next_turn();
            let mut updates = vec![update_actor(entity, actor.clone())];
            let mut wait_time = millis(0);

            if let Some(pending_action) = pending_action {
                let (action, delay) = *pending_action;
                let (mut more_updates, more_wait_time) = act((entity, actor, action, delay));

                updates.append(&mut more_updates);
                wait_time += more_wait_time;
            }
            
            (updates, wait_time)
        }

        Action::Wait(costs) => (
            // vec![update_actor(entity, actor.done(costs))],
            vec![],
            Duration::new(0, 0),
        ),

        Action::MoveTo(to) => (
            vec![update_actor(entity, actor.move_to(to))],
            millis(200),
        ),

        Action::Activate() => (
            vec![update_actor(entity, actor.activate())],
            millis(0),
        ),

        Action::Attack(target, attack) => {
            handle_attack((entity, actor), target, attack)
        }

        // Action::Defence(attacker, target, attack, defence) => {
        //     resolve_attack(attacker, target, attack, defence)
        // }
        
        // Action::EndTurn(next_turn) => {
        //     handle_end_turn(entity, actor, next_turn)
        // }
    }
}

fn handle_attack<'a>(
    attacker: (Entity, Actor),
    target: (Entity, Actor),
    attack: AttackOption,
) -> ActionResult {
    let fx_pos = target.1.pos.clone();
    let combat_result = super::actors::combat(attack, attacker.1, target.1);
    let changes = match combat_result {
        CombatResult::Miss(new_actor) => vec!(
            update_actor(target.0, new_actor),
            Change::Fx(Fx::text("Miss".to_string(), &fx_pos, 100)),
        ),

        CombatResult::Block() => vec!(
            Change::Fx(Fx::text("Blocked".to_string(), &fx_pos, 100)),
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

    (changes, millis(1000))
}

// fn resolve_attack<'a>(
//     _attacker: (Entity, Actor), // TODO: modify attacker (e.g. critical miss)
//     target: (Entity, Actor),
//     attack: Attack,
//     defence: Defence,
// ) -> ActionResult {
//     let fx_pos = target.1.pos.clone();
//     let combat_result = CombatResult::Strike(target.1);
//     // let combat_result = super::actors::combat(attack, defence, target.1);
//     let changes = match combat_result {
//         CombatResult::Strike(new_actor) => vec!(
//             update_actor(target.0, new_actor),
//             Change::Fx(Fx::text("Strike".to_string(), &fx_pos, 100)),
//         ),

//         CombatResult::Hit(condition) => {
//             match condition {
//                 Condition::Alive(new_actor) => vec!(
//                     update_actor(target.0, new_actor),
//                     Change::Fx(Fx::text("Hit!".to_string(), &fx_pos, 100)),
//                 ),

//                 Condition::Dead(pos, corpse) => vec!(
//                     Change::Remove(target.0),
//                     Change::Insert(GameObject::Item(pos, corpse)),
//                     Change::Fx(Fx::text("KILL!!!".to_string(), &fx_pos, 100)),
//                 ),
//             }
//         }
//     };

//     (changes, millis(1000))
// }

// fn handle_end_turn(e: Entity, a: Actor, next_turn: u64) -> ActionResult {
//     let condition = a.next_turn(next_turn);
//     let changes = match condition {
//         Condition::Alive(actor) =>
//             vec!(update_actor(e, actor)),

//         Condition::Dead(pos, corpse) => vec!(
//             Change::Remove(e),
//             Change::Insert(GameObject::Item(pos, corpse))
//         ),
//     };
    
//     (changes, millis(0))
// }

fn millis(ms: u64) -> Duration {
    Duration::from_millis(ms)
}

fn update_actor(e: Entity, a: Actor) -> Change {
    Change::Update(e, GameObject::Actor(a))
}
