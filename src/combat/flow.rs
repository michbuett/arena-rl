use bevy::prelude::*;

use super::actor::{
    ActionTriggeredEvent, Activation, Activations, Actor, BeginActivationCommand, PlayerControlled,
    PreparedAction, Team, TeamDeck, TeamHand,
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
    mut teams_q: Query<(Mut<TeamDeck>, Mut<TeamHand>, &PlayerControlled)>,
    mut activations_q: Query<
        (Entity, Mut<Activations>, &Team, Option<&PreparedAction>),
        With<Actor>,
    >,
) {
    if let TurnPhase::StartTurn = turn.turn_phase {
        // TODO: update hand
        // for (mut deck, mut hand, player_controlled) in teams_q.iter_mut() {
        //     if player_controlled.is_some() {

        //     }
        // }
        for (_, mut activations, Team(team_id), _) in activations_q.iter_mut() {
            if let Ok(mut team_deck) = teams_q.get_mut(*team_id) {
                activations.remaining = vec![Activation(team_deck.0 .0.deal())];
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

        println!("Processing ...");
        for (entity, activations, _, prep_action) in activations_q.iter() {
            if activations.active.is_some() {
                // info!(
                //     "Found active entity (entity={:?}, action={:?}",
                //     entity, prep_action
                // );
                if let Some(PreparedAction(action)) = prep_action {
                    commands.entity(entity).remove::<PreparedAction>();
                    commands.trigger_targets(ActionTriggeredEvent(action.clone()), entity);
                }

                // an actor is already active
                // => wait until an action is selected
                return;
            }

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

        if let Some((entity, _)) = actor_to_activate {
            commands.trigger_targets(BeginActivationCommand, entity);
        } else {
            // There are no more actors to activate
            turn.turn_number += 1;
            turn.turn_phase = TurnPhase::StartTurn;
        }
    }
}
