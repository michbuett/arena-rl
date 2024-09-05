use bevy::prelude::*;

use super::{actor::ActorId, map::MapPos, ui::WaitUntil};

#[derive(Debug, Resource)]
pub struct Turn {
    turn_number: u64,
    turn_phase: TurnPhase,
}

#[derive(Debug)]
pub enum TurnPhase {
    StartTurn,
    AssignActivations,
    PerformActions,
}

// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
// pub struct FactionId(u32);

// #[derive(Debug)]
// pub struct Faction {
//     id: FactionId,
//     is_pc: bool,
//     name: String,
// }

// #[derive(Debug)]
// pub struct Factions {
//     factions: Vec<Faction>,
//     current: usize,
// }

pub fn setup_combat_flow(mut commands: Commands) {
    commands.insert_resource(Turn {
        turn_number: 1,
        turn_phase: TurnPhase::StartTurn,
    });
}

#[derive(Event)]
pub enum CombatFlowEvent {}

pub fn update_combat_flow(
    // mut commands: Commands,
    // time: Res<Time>,
    wait_until: Option<Res<WaitUntil>>,
    mut turn: ResMut<Turn>,
    // combat_state: Option<ResMut<WaitUntil>>,
) {
    if wait_until.is_some() {
        return;
    }

    if let TurnPhase::AssignActivations = turn.turn_phase {
        // TODO: end assign activation phase when all teams are ready
    }

    if let TurnPhase::PerformActions = turn.turn_phase {
        // TODO: end perform action phase when there are no more action
        // to perform - i.e. when all actors where activated
    }
}

// fn progress_game(turn: &Turn) -> Option<Turn> {
//     match turn.turn_phase {
//         TurnPhase::StartTurn => {
//             // TODO add action to reset actor state for each turn
//             Some(Turn {
//                 turn_number: turn.turn_number,
//                 turn_phase: TurnPhase::AssignActivations,
//             })
//         }
//         TurnPhase::AssignActivations => {
//             // println!("assign activations");
//             None
//         }
//         TurnPhase::PerformActions => None,
//     }
// }

// pub fn perfrom_actions(action)
