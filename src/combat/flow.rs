use bevy::prelude::*;

use crate::{
    GameState,
    core::{ActiveEffects, Hand},
};

use super::{
    GameDeck,
    actor::{Activations, Active, Actor, BeginActivationCommand, Controller, TeamReady},
    fx::FxRunning,
};

#[derive(Debug, Resource)]
pub struct TurnNumber(pub u64);

#[derive(SubStates, Hash, Clone, Copy, Eq, PartialEq, Debug, Default)]
#[source(GameState = GameState::Combat)]
pub enum TurnPhase {
    #[default]
    StartTurn,
    BoostActivations,
    PerformActions,
}

pub fn setup_combat_flow(mut commands: Commands) {
    commands.insert_resource(TurnNumber(1));
}

pub fn handle_turn_phase_start_turn(
    mut deck: ResMut<GameDeck>,
    mut teams_q: Query<(Mut<Hand>, Mut<TeamReady>, &Controller)>,
    mut activations_q: Query<(Entity, Mut<Activations>, Mut<ActiveEffects>), With<Actor>>,
    mut next_turn_phase: ResMut<NextState<TurnPhase>>,
) {
    // println!("[enter_turn_phase_start_turn]");
    for (mut hand, mut team_ready, Controller(is_pc)) in teams_q.iter_mut() {
        if *is_pc {
            if !hand.is_full() {
                hand.add_card(deck.0.deal());
            }

            team_ready.0 = false;
            hand.reset_selection();
        }
    }

    for (_, mut activations, mut active_effects) in activations_q.iter_mut() {
        active_effects.new_turn();
        activations.refresh(vec![deck.0.deal()]);
    }

    next_turn_phase.set(TurnPhase::BoostActivations);
}

pub fn handle_turn_phase_boost_activations(
    teams_q: Query<(&Hand, &TeamReady, &Controller)>,
    mut next_turn_phase: ResMut<NextState<TurnPhase>>,
) -> Result<(), BevyError> {
    let all_ready = teams_q
        .iter()
        .fold(true, |ready_so_far, (_, team_ready, _)| {
            ready_so_far && team_ready.0
        });

    if all_ready {
        next_turn_phase.set(TurnPhase::PerformActions);
    }

    Ok(())
}

pub fn handle_turn_phase_perform_actions(
    activations_q: Query<(Entity, &Activations), With<Actor>>,
    running_fx_q: Query<(), With<FxRunning>>,
    active_q: Query<(), With<Active>>,
    mut commands: Commands,
    mut turn_number: ResMut<TurnNumber>,
    mut next_turn_phase: ResMut<NextState<TurnPhase>>,
) {
    // println!("[handle_turn_phase_perform_actions]");

    if !running_fx_q.is_empty() {
        // There are still some effects running
        // => wait for them to finish
        return;
    }

    if !active_q.is_empty() {
        // an actor is already active
        // => wait until an action is selected
        return;
    }

    // Activate actors in order if the initiative card
    let mut actor_to_activate: Option<(Entity, u8)> = None;

    // for (entity, activations, _, prep_action) in activations_q.iter() {
    for (entity, activations, ..) in activations_q.iter() {
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

    // println!("ACTIVATE:{:?}", actor_to_activate);
    if let Some((entity, _)) = actor_to_activate {
        commands.trigger(BeginActivationCommand(entity));
    } else {
        // There are no more actors to activate
        turn_number.0 += 1;
        next_turn_phase.set(TurnPhase::StartTurn);
    }
}
