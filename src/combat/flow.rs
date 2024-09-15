use bevy::prelude::*;

use super::{
    actor::{
        Action, ActionSelectedEvent, ActivateActor, Activation, Activations, Actor,
        PlayerControlled, Team, TeamDeck, TeamHand,
    },
    ui::WaitForUser,
};

#[derive(Debug, Resource)]
pub struct Turn {
    pub turn_number: u64,
    pub turn_phase: TurnPhase,
}

#[derive(Debug)]
pub enum TurnPhase {
    StartTurn,
    BoostActivations,
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
    mut commands: Commands,
    mut turn: ResMut<Turn>,
    mut teams_q: Query<(Mut<TeamDeck>, Mut<TeamHand>, &PlayerControlled)>,
    mut activations_q: Query<(Entity, Mut<Activations>, &Team), With<Actor>>,
    // mut activations_q: Query<(Entity, &Activations)>,
) {
    if let TurnPhase::StartTurn = turn.turn_phase {
        // TODO: update hand
        // for (mut deck, mut hand, player_controlled) in teams_q.iter_mut() {
        //     if player_controlled.is_some() {

        //     }
        // }
        for (_, mut activations, Team(team_id)) in activations_q.iter_mut() {
            if let Ok(mut team_deck) = teams_q.get_mut(*team_id) {
                activations.remaining = vec![Activation::Single(team_deck.0 .0.deal())];
            }
        }

        turn.turn_phase = TurnPhase::BoostActivations;
    }

    if let TurnPhase::BoostActivations = turn.turn_phase {
        turn.turn_phase = TurnPhase::PerformActions;
    }

    if let TurnPhase::PerformActions = turn.turn_phase {
        // Activate actors in order if the initiative card
        let mut actor_to_activate: Option<(Entity, u8)> = None;
        // activations_q.iter().min_by(|(a1, ..), (a2, ..)| a1.)

        for (entity, activations, ..) in activations_q.iter_mut() {
            if let Some(initiative_value) = activations.next_activation_initiative() {
                actor_to_activate = actor_to_activate.map_or(
                    Some((entity, initiative_value)),
                    |(id_so_far, ini_so_far)| {
                        if initiative_value < ini_so_far {
                            Some((entity, initiative_value))
                        } else {
                            Some((id_so_far, ini_so_far))
                        }
                    },
                );
            }
        }

        if let Some((actor_id, _)) = actor_to_activate {
            commands.trigger(ActivateActor(actor_id));
        } else {
            // There are no more actors to activate
            turn.turn_number += 1;
            turn.turn_phase = TurnPhase::StartTurn;
        }
    }
}

pub fn handle_activate_actor(
    trigger: Trigger<ActivateActor>,
    mut commands: Commands,
    // time: Res<Time>,
    // mut wait_for_user: Option<ResMut<WaitForUser>>,
    // mut turn: ResMut<Turn>,
    // combat_state: Option<ResMut<WaitUntil>>,
    // mut teams_q: Query<(Mut<TeamDeck>, Mut<TeamHand>, Option<&PlayerControlled>)>,
    mut actor_activation_q: Query<(Mut<Activations>, &PlayerControlled)>,
    // mut team_q: Query<&ControlledBy>,
) {
    println!("[DEBUG] handle_activate_actor");

    let ActivateActor(e) = trigger.event();

    if let Ok((mut activation, PlayerControlled(is_pc))) = actor_activation_q.get_mut(*e) {
        activation.active = activation.remaining.pop();

        if *is_pc {
            println!("    handle_activate_actor - wait for user");
            commands.insert_resource(WaitForUser());
        } else {
            println!("    handle_activate_actor - process AI actor");
            commands.trigger(ActionSelectedEvent(Action::NoOp))
        }
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
