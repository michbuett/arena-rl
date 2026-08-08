use bevy::prelude::*;

use crate::{
    GameState,
    combat::actor::Initiative,
    core::{ActiveEffects, Hand},
};

use super::{
    GameDeck,
    actor::{Activations, Active, Actor, BeginActivationCommand, Controller, TeamReady},
    fx::FxRunning,
};

#[derive(Debug, Resource)]
pub struct Turn(pub u64, pub Initiative);

impl Turn {}

#[derive(SubStates, Hash, Clone, Copy, Eq, PartialEq, Debug, Default)]
#[source(GameState = GameState::Combat)]
pub enum TurnPhase {
    #[default]
    StartTurn,
    PerformActions,
}

pub fn setup_combat_flow(mut commands: Commands) {
    commands.insert_resource(Turn(1, Initiative::start()));
}

pub fn handle_turn_phase_start_turn(
    mut deck: ResMut<GameDeck>,
    mut teams_q: Query<(Mut<Hand>, Mut<TeamReady>, &Controller)>,
    mut activations_q: Query<Mut<ActiveEffects>, With<Actor>>,
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

    for mut active_effects in activations_q.iter_mut() {
        active_effects.new_turn();
    }

    next_turn_phase.set(TurnPhase::PerformActions);
}

pub fn handle_turn_phase_perform_actions(
    activations_q: Query<(Entity, &Activations), With<Actor>>,
    running_fx_q: Query<(), With<FxRunning>>,
    active_q: Query<(), With<Active>>,
    mut commands: Commands,
    mut turn: ResMut<Turn>,
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

    let curr_turn_number = turn.0;
    let mut actors_to_activate = activations_q
        .iter()
        .filter_map(|(e, a)| {
            a.next_activation_initiative(curr_turn_number)
                .map(|i| (e, i))
        })
        .collect::<Vec<_>>();

    actors_to_activate.sort_by_key(|(_, i)| *i);

    if let Some((actor_to_activate, initiative_value)) = actors_to_activate.first() {
        turn.1 = *initiative_value;
        commands.trigger(BeginActivationCommand(*actor_to_activate));
    } else {
        // There are no more actors to activate
        turn.0 = curr_turn_number + 1;
        turn.1 = Initiative::start();
        next_turn_phase.set(TurnPhase::StartTurn);
    }

    // }

    // let mut current_initiative = Some(turn_number.1);
    // while let Some(initiative) = current_initiative {
    //     for (entity, activations, ..) in activations_q.iter() {
    //         if let Some(initiative_value) = activations.next_activation_initiative(turn_number.0) {
    //             if initiative_value <= initiative {
    //                 commands.trigger(BeginActivationCommand(entity));
    //                 return;
    //             }
    //         }
    //     }
    //     current_initiative = initiative.step()
    // }

    // turn_number.0 += 1;
    // next_turn_phase.set(TurnPhase::StartTurn);

    // // Activate actors in order if the initiative card
    // // let mut actor_to_activate: Option<(Entity, u8)> = None;

    // // for (entity, activations, ..) in activations_q.iter() {
    // // if let Some(initiative_value) = activations.next_activation_initiative(turn_number.0) {}
    // //         actor_to_activate = actor_to_activate.map_or(
    // //             Some((entity, initiative_value)),
    // //             |(id_so_far, ini_so_far)| {
    // //                 if initiative_value < ini_so_far {
    // //                     Some((entity, initiative_value))
    // //                 } else {
    // //                     Some((id_so_far, ini_so_far))
    // //                 }
    // //             },
    // //         );
    // //     }
    // // }

    // // println!("ACTIVATE:{:?}", actor_to_activate);
    // if let Some((entity, _)) = actor_to_activate {
    //     commands.trigger(BeginActivationCommand(entity));
    // } else {
    //     // There are no more actors to activate
    //     turn_number.0 += 1;
    //     next_turn_phase.set(TurnPhase::StartTurn);
    // }
}
