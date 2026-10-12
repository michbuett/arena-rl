use bevy::prelude::*;

use crate::{
    GameState,
    core::{ActiveEffects, Card, Hand, Suite},
};

use super::{
    GameDeck,
    actor::{Activations, Active, Actor, BeginActivationCommand, Controller, TeamReady},
    fx::FxRunning,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Initiative(u8);

impl From<Card> for Initiative {
    fn from(card: Card) -> Self {
        let face_val = (card.value() - 1) * 4;
        let suite_val = match card.suite() {
            Suite::Spades => 0,
            Suite::Diamonds => 1,
            Suite::Hearts => 2,
            Suite::Clubs => 3,
        };
        Self(face_val + suite_val)
    }
}

impl Initiative {
    pub fn start() -> Self {
        Initiative(0)
    }

    pub fn as_card(&self) -> Card {
        let card_value = (self.0 / 4 + 1).try_into();
        let value = match card_value {
            Ok(cv) => cv,
            Err(e) => panic!("{e}"),
        };
        let suite = match self.0 % 4 {
            0 => Suite::Spades,
            1 => Suite::Diamonds,
            2 => Suite::Hearts,
            3 => Suite::Clubs,
            _ => panic!("Unreachable"),
        };
        Card::new(value, suite)
    }
}

#[derive(Debug, Resource, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Turn(u64, Initiative);

impl Turn {
    pub fn start() -> Self {
        Turn(0, Initiative::start())
    }

    pub fn next_activation(&self, card: Card) -> Self {
        let initiative: Initiative = card.into();
        if self.1 >= initiative {
            Self(self.0 + 1, initiative)
        } else {
            Self(self.0, initiative)
        }
    }

    pub fn turn_number(&self) -> u64 {
        self.0
    }

    pub fn initiative(&self) -> Initiative {
        self.1
    }
}

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
    mut commands: Commands,
    mut deck: ResMut<GameDeck>,
    mut teams_q: Query<(Mut<Hand>, Mut<TeamReady>, &Controller)>,
    activations_q: Query<(Entity, &ActiveEffects), With<Actor>>,
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

    for (e, active_effects) in activations_q.iter() {
        commands.entity(e).insert(active_effects.new_turn());
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
        .filter_map(|(e, a)| a.next().map(|t| (e, t)))
        .collect::<Vec<_>>();

    actors_to_activate.sort_by_key(|(_, i)| i.clone());

    let Some((actor_to_activate, activation_turn)) = actors_to_activate.first() else {
        // This should not happen because every actor gets a new activation when he completed his last activation
        panic!("There is no actor to activate");
    };

    // println!("ACTIVATE:{actor_to_activate:?} ({activation_turn:?})");
    if activation_turn.turn_number() > curr_turn_number {
        turn.0 = curr_turn_number + 1;
        turn.1 = Initiative::start();
        next_turn_phase.set(TurnPhase::StartTurn);
    } else {
        turn.1 = activation_turn.initiative();
        commands.trigger(BeginActivationCommand(*actor_to_activate));
    }
}

#[test]
fn turns_are_sorted_correctly() {
    use crate::core::CardValue::*;
    use crate::core::Suite::*;

    let t1 = Turn::start().next_activation(Card::new(Five, Spades));
    let t2 = Turn::start().next_activation(Card::new(Three, Clubs));
    let t3 = Turn::start().next_activation(Card::new(Three, Diamonds));
    let t4 = Turn::start().next_activation(Card::new(Three, Hearts));
    let t5 = Turn::start().next_activation(Card::new(Eight, Hearts));
    let t6 = Turn::start().next_activation(Card::new(Ace, Hearts));
    let t7 = Turn::start()
        .next_activation(Card::new(Eight, Hearts))
        .next_activation(Card::new(Two, Spades));
    let t8 = Turn::start()
        .next_activation(Card::new(Three, Hearts))
        .next_activation(Card::new(Ace, Spades));
    let mut set = vec![t1, t2, t3, t4, t5, t6, t7, t8];

    set.sort();

    assert_eq!(set[0], t6);
    assert_eq!(set[1], t3);
    assert_eq!(set[2], t4);
    assert_eq!(set[3], t2);
    assert_eq!(set[4], t1);
    assert_eq!(set[5], t5);
    assert_eq!(set[6], t8);
    assert_eq!(set[7], t7);
}
