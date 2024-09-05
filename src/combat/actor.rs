use bevy::prelude::*;

use super::{
    cards::{Card, Suite},
    map::{HexMap, MapPos, Obstacle},
    ui::{MapPosSelectedEvent, PlayerActions},
    Visual,
};

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

// impl Activation {
//     pub fn initiative_value(&self) -> u8 {
//         match self {
//             Activation::Single(c) => c.value,
//             Activation::Boosted(c1, c2) => std::cmp::min(c1.value, c2.value),
//         }
//     }

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
// }

#[derive(Component)]
pub struct Activations {
    pub active: Option<Activation>,
    pub remaining: Vec<Activation>,
}

// pub struct Activation();

#[derive(Bundle)]
pub struct ActorBundle {
    pub actor: Actor,
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
    pub fn new(map_pos: MapPos, visual: Visual) -> Self {
        Self {
            actor: Actor::new(),
            map_pos,
            visual,
            activations: Activations {
                active: None,
                remaining: vec![],
            },
            obstacle: Obstacle(f32::MAX),
            transform: Transform::from_translation(map_pos.into_vec3().with_z(100.0)),
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
pub struct ActionSelectedEvent(Action);

#[derive(Debug, Clone)]
pub enum Action {
    MoveAlong {
        actor_id: ActorId,
        path: Vec<MapPos>,
    },
}
pub fn handle_select_map_pos(
    map: Res<HexMap>,
    actorq: Query<(&Actor, &Activations, &MapPos), Without<AiBehaviour>>,
    mut map_pos_selected_er: EventReader<MapPosSelectedEvent>,
    mut action_selected_ew: EventWriter<ActionSelectedEvent>,
    mut player_actions: ResMut<PlayerActions>,
) {
    let Some(MapPosSelectedEvent(hex)) = map_pos_selected_er.read().last() else {
        return;
    };

    if let Some((actor_id, actor_pos)) = find_active_player_actor(&actorq) {
        if let Some(action) = player_actions.get_selected_action_when_at(hex) {
            action_selected_ew.send(ActionSelectedEvent(action.clone()));
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

pub fn handle_action_selected(
    map: Res<HexMap>,
    actorq: Query<(&Actor, &Activations, &MapPos), Without<AiBehaviour>>,
    mut action_selected_er: EventReader<ActionSelectedEvent>,
) {
    let Some(ActionSelectedEvent(action)) = action_selected_er.read().last() else {
        return;
    };

    println!("[DEBUG] handle_action_selected: {:?}", action);
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
