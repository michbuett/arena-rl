use std::time::Duration;

use bevy::prelude::*;

use crate::animations::{MovementAnimation, MovementModification};

use super::{
    cards::{Card, Deck},
    map::{HexMap, MapPos, Obstacle},
    ui::{MapPosSelectedEvent, PlayerActions, WaitForUser},
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActorId(u64);

impl ActorId {
    fn new() -> Self {
        Self(rand::random())
    }
}

#[derive(Component)]
pub struct Actor {
    pub id: ActorId,
}

impl Actor {
    fn new() -> Self {
        Self { id: ActorId::new() }
    }
}

#[derive(Component)]
pub enum AiBehaviour {
    Zombi,
}

#[derive(Debug, Clone)]
pub enum Activation {
    Single(Card),
    // Boosted(Card, Card),
    // Hindered(Card, Card),
}

impl Activation {
    pub fn initiative_value(&self) -> u8 {
        match self {
            Activation::Single(c) => c.value,
            // Activation::Boosted(c1, c2) => std::cmp::min(c1.value, c2.value),
        }
    }

    //     pub fn effort_value(&self, suite: Suite) -> Card {
    //         match self {
    //             Activation::Single(c) => *c,
    //             Activation::Boosted(c1, c2) => {
    //                 if c1.value(suite) >= c2.value(suite) {
    //                     *c1
    //                 } else {
    //                     *c2
    //                 }
    //             }
    //         }
    //     }
}

#[derive(Component)]
pub struct Activations {
    pub active: Option<Activation>,
    pub remaining: Vec<Activation>,
}

impl Activations {
    pub fn next_activation_initiative(&self) -> Option<u8> {
        self.remaining
            .iter()
            .map(Activation::initiative_value)
            .min()
    }
}

// #[derive(Resource)]
// pub struct ActorEntityMap(HashMap<ActorId, Entity>);

// pub fn setup_actor_entity_map(mut commands: Commands) {
//     commands.insert_resource(ActorEntityMap(HashMap::new()));
// }

#[derive(Bundle)]
pub struct ActorBundle {
    pub actor: Actor,
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
    pub fn new(team: Team, is_pc: bool, map_pos: MapPos, visual: Visual) -> Self {
        Self {
            actor: Actor::new(),
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
pub struct ActivateActor(pub Entity);

#[derive(Debug, Event)]
pub struct MoveToCommand(ActorId, Vec<MapPos>);

#[derive(Debug, Clone)]
pub enum Action {
    NoOp,
    MoveAlong {
        actor_id: ActorId,
        path: Vec<MapPos>,
    },
}

pub fn handle_select_map_pos(
    map: Res<HexMap>,
    actorq: Query<(&Actor, &Activations, &MapPos), Without<AiBehaviour>>,
    mut commands: Commands,
    mut map_pos_selected_er: EventReader<MapPosSelectedEvent>,
    mut player_actions: ResMut<PlayerActions>,
) {
    let Some(MapPosSelectedEvent(hex)) = map_pos_selected_er.read().last() else {
        return;
    };

    if let Some((actor_id, actor_pos)) = find_active_player_actor(&actorq) {
        if let Some(action) = player_actions.get_selected_action_when_at(hex) {
            commands.trigger(ActionSelectedEvent(action.clone()));
            player_actions.set_available_actions(*hex, vec![]);
        } else {
            if let Some(path) = map.find_path(actor_pos, *hex) {
                player_actions
                    .set_available_actions(*hex, vec![Action::MoveAlong { actor_id, path }]);
            } else {
                player_actions.set_available_actions(*hex, vec![]);
            }
        }
    }
}

pub fn handle_action_selected_event(trigger: Trigger<ActionSelectedEvent>, mut commands: Commands) {
    let ActionSelectedEvent(action) = trigger.event();
    match action {
        Action::MoveAlong { actor_id, path } => {
            commands.trigger(MoveToCommand(*actor_id, path.clone()));
        }

        Action::NoOp => {
            // Just do nothing
        }
    }

    commands.remove_resource::<WaitForUser>();
}

pub fn handle_move_to_command(
    trigger: Trigger<MoveToCommand>,
    mut commands: Commands,
    actorq: Query<(Entity, &Actor)>,
) {
    let MoveToCommand(actor_id, path) = trigger.event();

    if let Some(e) = find_entity_by_actor_id(*actor_id, &actorq) {
        commands.entity(e).insert((
            MovementAnimation::new(
                Duration::from_millis(200),
                path.iter()
                    .map(|mpos| mpos.into_vec3().with_z(ACTOR_ZLAYER))
                    .collect(),
            )
            .set_modification(MovementModification::ParabolaJump(100)),
            *path.last().unwrap(),
        ));
    }
}

fn find_active_player_actor(
    q_active_actor: &Query<(&Actor, &Activations, &MapPos), Without<AiBehaviour>>,
) -> Option<(ActorId, MapPos)> {
    for (actor, activations, map_pos) in q_active_actor.iter() {
        if activations.active.is_some() {
            return Some((actor.id, *map_pos));
        }
    }
    None
}

fn find_entity_by_actor_id(
    actor_id: ActorId,
    entity_actor_query: &Query<(Entity, &Actor)>,
) -> Option<Entity> {
    for (e, a) in entity_actor_query.iter() {
        if a.id == actor_id {
            return Some(e);
        }
    }

    None
}
