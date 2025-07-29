use bevy::prelude::*;

use crate::core::Hand;

use super::{
    GameDeck,
    actor::{Activations, Actor, BeginActivationCommand, PlayerControlled, TeamReady},
    map::MapPos,
    ui::{UiState, UiStateTransitionedEvent},
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
    mut deck: ResMut<GameDeck>,
    mut teams_q: Query<(Mut<Hand>, Mut<TeamReady>, &PlayerControlled)>,
    mut activations_q: Query<(Entity, Mut<Activations>), With<Actor>>,
) -> Result<(), BevyError> {
    if let TurnPhase::StartTurn = turn.turn_phase {
        for (mut hand, mut team_ready, PlayerControlled(is_pc)) in teams_q.iter_mut() {
            if *is_pc {
                if !hand.is_full() {
                    hand.add_card(deck.0.deal());
                }

                team_ready.0 = false;
                hand.reset_selection();
            }
        }

        for (_, mut activations) in activations_q.iter_mut() {
            activations.refresh(vec![deck.0.deal()]);
        }

        turn.turn_phase = TurnPhase::BoostActivations;
        return Ok(());
    }

    if let TurnPhase::BoostActivations = turn.turn_phase {
        let all_ready = teams_q
            .iter()
            .fold(true, |ready_so_far, (_, team_ready, _)| {
                ready_so_far && team_ready.0
            });

        if all_ready {
            turn.turn_phase = TurnPhase::PerformActions;
        } else {
            commands.trigger(UiStateTransitionedEvent(UiState::await_input(MapPos::new(
                0, 0,
            ))));
        }
        return Ok(());
    }

    if let TurnPhase::PerformActions = turn.turn_phase {
        // Activate actors in order if the initiative card
        let mut actor_to_activate: Option<(Entity, u8)> = None;

        // for (entity, activations, _, prep_action) in activations_q.iter() {
        for (entity, activations, ..) in activations_q.iter() {
            if activations.active().is_some() {
                // an actor is already active
                // => wait until an action is selected
                return Ok(());
            }
            // println!(" - entity:{:?} - {:?}", entity, activations);
            if let Some(initiative_value) = activations.next_activation_initiative() {
                actor_to_activate = actor_to_activate.map_or(
                    Some((entity, initiative_value)),
                    |(id_so_far, ini_so_far)| {
                        if initiative_value > ini_so_far {
                            Some((entity, initiative_value))
                        } else {
                            Some((id_so_far, ini_so_far))
                        }
                    },
                );
            }
        }

        // println!("ACTIVATE:{:?}", actor_to_activate);
        if let Some((entity, _)) = actor_to_activate {
            commands.trigger_targets(BeginActivationCommand, entity);
        } else {
            // There are no more actors to activate
            turn.turn_number += 1;
            turn.turn_phase = TurnPhase::StartTurn;
        }
    }

    Ok(())
}
