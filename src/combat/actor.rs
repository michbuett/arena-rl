use std::time::Duration;

use bevy::prelude::*;

use crate::animations::{MovementAnimation, MovementModification};

use super::{
    cards::{Card, Deck},
    map::{HexMap, MapPos, Obstacle},
    ui::{Description, MapPosSelectedEvent, PlayerActions, TransitionUiState, UiState},
    Visual,
};

#[derive(Component, PartialEq, Clone, Copy)]
pub struct Team(pub Entity);

#[derive(Component, Debug)]
pub struct TeamDeck(pub Deck);

#[derive(Component, Debug)]
pub struct TeamHand(Vec<Card>);

#[derive(Bundle)]
pub struct TeamBundle {
    name: Name,
    deck: TeamDeck,
    hand: TeamHand,
    player_controlled: PlayerControlled,
}
impl TeamBundle {
    pub fn new(name: impl Into<Name>, is_pc: bool) -> Self {
        Self {
            name: name.into(),
            deck: TeamDeck(Deck::new_rnd()),
            hand: TeamHand(vec![]),
            player_controlled: PlayerControlled(is_pc),
        }
    }
}

#[derive(Component)]
pub struct PlayerControlled(pub bool);

const ACTOR_ZLAYER: f32 = 100.0;

#[derive(Component)]
pub struct Actor {}

impl Actor {
    fn new() -> Self {
        Self {}
    }
}

#[derive(Component)]
pub enum AiBehaviour {
    Zombi,
}

#[derive(Debug, Clone)]
pub struct Activation(pub Card);

#[derive(Component)]
pub struct Activations {
    pub active: Option<Activation>,
    pub remaining: Vec<Activation>,
}

impl Activations {
    pub fn next_activation_initiative(&self) -> Option<u8> {
        self.remaining.iter().map(|Activation(c)| c.value).min()
    }
}

#[derive(Bundle)]
pub struct ActorBundle {
    pub actor: Actor,
    pub name: Name,
    pub description: Description,
    pub team: Team,
    pub play_controlled: PlayerControlled,
    pub map_pos: MapPos,
    pub visual: Visual,
    pub activations: Activations,
    pub obstacle: Obstacle,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
    pub inherited_visibility: InheritedVisibility,
}

impl ActorBundle {
    pub fn new(name: Name, team: Team, is_pc: bool, map_pos: MapPos, visual: Visual) -> Self {
        Self {
            actor: Actor::new(),
            name,
            description: Description::new("Some description for an actor"),
            team,
            play_controlled: PlayerControlled(is_pc),
            map_pos,
            visual,
            activations: Activations {
                active: None,
                remaining: vec![],
            },
            obstacle: Obstacle(f32::MAX),
            transform: Transform::from_translation(map_pos.into_vec3().with_z(ACTOR_ZLAYER)),
            global_transform: GlobalTransform::default(),
            visibility: Visibility::Visible,
            inherited_visibility: InheritedVisibility::default(),
        }
    }
}

#[derive(Bundle)]
pub struct AiActorBundle {
    actor_bundle: ActorBundle,
    ai_behaivour: AiBehaviour,
}

#[derive(Debug, Event)]
pub struct ActionSelectedEvent(pub Action);

#[derive(Debug, Event)]
pub struct BeginActivationCommand;

#[derive(Debug, Event)]
pub struct EndActivationCommand;

#[derive(Debug, Event)]
pub struct MoveToCommand(Vec<MapPos>);

#[derive(Debug, Clone)]
pub enum Action {
    NoOp,
    MoveAlong { path: Vec<MapPos> },
}

pub fn handle_select_map_pos(
    map: Res<HexMap>,
    actorq: Query<(Entity, &Actor, &Activations, &MapPos), Without<AiBehaviour>>,
    mut commands: Commands,
    mut map_pos_selected_er: EventReader<MapPosSelectedEvent>,
    mut player_actions: ResMut<PlayerActions>,
) {
    let Some(MapPosSelectedEvent(hex)) = map_pos_selected_er.read().last() else {
        return;
    };

    if let Some((entity, actor_pos)) = find_active_player_actor(&actorq) {
        if let Some(action) = player_actions.get_selected_action_when_at(hex) {
            commands.trigger_targets(ActionSelectedEvent(action.clone()), entity);
            player_actions.set_available_actions(*hex, vec![]);
        } else {
            if let Some(path) = map.find_path(actor_pos, *hex) {
                player_actions.set_available_actions(*hex, vec![Action::MoveAlong { path }]);
            } else {
                player_actions.set_available_actions(*hex, vec![]);
            }
        }
    }
}

fn find_active_player_actor(
    q_active_actor: &Query<(Entity, &Actor, &Activations, &MapPos), Without<AiBehaviour>>,
) -> Option<(Entity, MapPos)> {
    for (entity, _actor, activations, map_pos) in q_active_actor.iter() {
        if activations.active.is_some() {
            return Some((entity, *map_pos));
        }
    }
    None
}

pub fn handle_action_selected_event(trigger: Trigger<ActionSelectedEvent>, mut commands: Commands) {
    let ActionSelectedEvent(action) = trigger.event();
    let entity = trigger.entity();

    match action {
        Action::MoveAlong { path } => {
            commands.trigger_targets(MoveToCommand(path.clone()), entity);
        }

        Action::NoOp => {
            // Just do nothing
        }
    }

    commands.trigger_targets(EndActivationCommand, entity);
}

pub fn handle_begin_activation_command(
    trigger: Trigger<BeginActivationCommand>,
    mut commands: Commands,
    mut actor_activation_q: Query<(Mut<Activations>, &PlayerControlled, &MapPos)>,
) {
    let e = trigger.entity();

    if let Ok((mut activation, PlayerControlled(is_pc), mpos)) = actor_activation_q.get_mut(e) {
        activation.active = activation.remaining.pop();

        if *is_pc {
            commands.trigger(TransitionUiState(UiState::await_input(*mpos)));
        } else {
            commands.trigger_targets(ActionSelectedEvent(Action::NoOp), e)
        }
    }
}

pub fn handle_end_activation_command(
    trigger: Trigger<EndActivationCommand>,
    mut activations_q: Query<Mut<Activations>>,
) {
    if let Ok(mut activations) = activations_q.get_mut(trigger.entity()) {
        activations.active = None;
    }
}

pub fn handle_move_to_command(trigger: Trigger<MoveToCommand>, mut commands: Commands) {
    let MoveToCommand(path) = trigger.event();
    let step_durr = 200;

    commands.entity(trigger.entity()).insert((
        MovementAnimation::new(
            Duration::from_millis(step_durr),
            path.iter()
                .map(|mpos| mpos.into_vec3().with_z(ACTOR_ZLAYER))
                .collect(),
        )
        .set_modification(MovementModification::ParabolaJump(100)),
        *path.last().unwrap(),
    ));

    commands.trigger(TransitionUiState(UiState::wait(
        step_durr * path.len() as u64,
    )));
}

#[derive(Event)]
pub struct ActivationChanged;

pub fn check_actor_changes(
    mut commands: Commands,
    changes_q: Query<(Entity, &Actor), Changed<Activations>>,
) {
    for (entity, _) in &changes_q {
        commands.trigger_targets(ActivationChanged, entity);
    }
}
