use std::time::Duration;

use bevy::{input::mouse::MouseMotion, prelude::*, window::PrimaryWindow};

use crate::core::{Card, Suite};
use crate::{style::WINDOW_BACKGROUND, GameState};

use super::{
    actor::{Action, Activation, Activations, Actor},
    flow::{Turn, TurnPhase},
    map::{HexMap, MapPos},
    OnCombatState, ScrollBounds, Visual,
};

pub const Z_LAYER_FLOOR: f32 = 1.0;
pub const Z_LAYER_UI_MAP_MARKER: f32 = 10.0;
pub const Z_LAYER_ACTOR: f32 = 100.0;
pub const Z_LAYER_VFX: f32 = 200.0;

#[derive(Component)]
pub struct TurnInfo;

#[derive(Component)]
pub struct DetailsWindowText;

#[derive(Component)]
pub struct SelectedTile;

#[derive(Component)]
pub struct PlayerActionIndicator;

#[derive(Resource)]
pub struct ScrollState(bool);

#[derive(Resource)]
pub struct PlayerActions {
    map_pos: Option<MapPos>,
    available_actions: Vec<Action>,
    selected_action: usize,
}

impl PlayerActions {
    pub fn empty() -> Self {
        Self {
            map_pos: None,
            available_actions: Vec::new(),
            selected_action: 0,
        }
    }

    pub fn set_available_actions(&mut self, mp: MapPos, actions: Vec<Action>) {
        self.map_pos = Some(mp);
        self.available_actions = actions;
        self.selected_action = 0;
    }

    pub fn get_selected_action(&self) -> Option<&Action> {
        self.available_actions.get(self.selected_action)
    }

    pub fn get_selected_action_when_at(&self, other_pos: &MapPos) -> Option<&Action> {
        self.get_selected_action()
            .and_then(|action| match self.map_pos {
                Some(prev_selected_hex) if *other_pos == prev_selected_hex => Some(action),
                _ => None,
            })
    }
}

#[derive(Debug, Event)]
pub struct MapPosSelectedEvent(pub MapPos);

#[derive(Component)]
pub struct UiElement;

pub fn combat_ui_plugin(app: &mut App) {
    app.add_event::<MapPosSelectedEvent>()
        .add_event::<UiStateTransitionedEvent>()
        .observe(update_ui_on_state_change)
        .add_systems(OnEnter(GameState::Combat), setup_ui)
        .add_systems(
            Update,
            (
                (update_user_input, handle_select_map_pos).chain(),
                update_description_on_activation_changed,
                update_available_playeractions.run_if(resource_exists_and_changed::<PlayerActions>),
                update_turn_info.run_if(resource_exists_and_changed::<Turn>),
                update_waiting_state,
            )
                .run_if(in_state(GameState::Combat)),
        );
}

fn setup_ui(mut commands: Commands) {
    commands.insert_resource(ScrollState(false));
    commands.insert_resource(PlayerActions::empty());
    commands.insert_resource(UiState::prossing());

    commands
        .spawn((
            NodeBundle {
                background_color: WINDOW_BACKGROUND.into(),
                style: Style {
                    position_type: PositionType::Absolute,
                    top: Val::Px(20.0),
                    left: Val::Px(20.0),
                    width: Val::Px(200.0),
                    ..Default::default()
                },
                ..Default::default()
            },
            OnCombatState,
        ))
        .with_children(|parent| {
            parent.spawn((
                TextBundle::from_section(
                    "Turn: -",
                    TextStyle {
                        color: Color::BLACK,
                        font_size: 12.0,
                        ..Default::default()
                    },
                ),
                TurnInfo,
            ));
        });
}

fn update_user_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    dim: Res<ScrollBounds>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    ui_state: Res<UiState>,
    mut mouse_motion_evr: EventReader<MouseMotion>,
    mut camera_transform_query: Query<&mut Transform, With<Camera>>,
    mut map_pos_selected_ew: EventWriter<MapPosSelectedEvent>,
    mut scroll_state: ResMut<ScrollState>,
) {
    let mut camera_transform = camera_transform_query.single_mut();

    if keyboard_input.pressed(KeyCode::KeyA) || keyboard_input.pressed(KeyCode::ArrowLeft) {
        move_camera(&mut camera_transform, -10.0, 0.0, &dim);
    } else if keyboard_input.pressed(KeyCode::KeyD) || keyboard_input.pressed(KeyCode::ArrowRight) {
        move_camera(&mut camera_transform, 10.0, 0.0, &dim);
    } else if keyboard_input.pressed(KeyCode::KeyW) || keyboard_input.pressed(KeyCode::ArrowUp) {
        move_camera(&mut camera_transform, 0.0, 10.0, &dim);
    } else if keyboard_input.pressed(KeyCode::KeyS) || keyboard_input.pressed(KeyCode::ArrowDown) {
        move_camera(&mut camera_transform, 0.0, -10.0, &dim);
    }

    if mouse_button_input.pressed(MouseButton::Left) {
        for ev in mouse_motion_evr.read() {
            move_camera(&mut camera_transform, -ev.delta.x, ev.delta.y, &dim);
            scroll_state.0 = true;
        }
    }

    if mouse_button_input.just_released(MouseButton::Left) {
        if scroll_state.0 {
            // the user moved the mouse during a previous frame before
            // releasing the left mouse button
            // => interprete this as scrolling and not as a single click
            // => .. than reset the scroll state until next time
            scroll_state.0 = false;
        } else {
            if !ui_state.is_awaiting_input() {
                // there is other stuff going on
                // => just ignore user other input then scrolling
                return;
            }

            if let Some(mouse_pos) = window_query.single().cursor_position() {
                let (camera, camera_global_transform) = camera_query.single();
                if let Some(wp) = camera.viewport_to_world_2d(camera_global_transform, mouse_pos) {
                    map_pos_selected_ew.send(MapPosSelectedEvent(MapPos::from(wp)));
                }
            }
        }
    }
}

fn move_camera(camera_transform: &mut Transform, dx: f32, dy: f32, scroll_bounds: &ScrollBounds) {
    let bounds_min = scroll_bounds.0.min;
    let bounds_max = scroll_bounds.0.max;

    camera_transform.translation.x =
        (camera_transform.translation.x + dx).clamp(bounds_min.x, bounds_max.x);

    camera_transform.translation.y =
        (camera_transform.translation.y + dy).clamp(bounds_min.y, bounds_max.y);
}

fn handle_select_map_pos(
    map: Res<HexMap>,
    description_q: Query<(&MapPos, &Description)>,
    mut user_input_evr: EventReader<MapPosSelectedEvent>,
    mut details_window_text: Query<&mut Text, With<DetailsWindowText>>,
    mut selected_tile_q: Query<(&mut Transform, &mut Visibility), With<SelectedTile>>,
) {
    let Some(MapPosSelectedEvent(hex)) = user_input_evr.read().last() else {
        return;
    };

    let details_window = details_window_text.get_single_mut();
    let selected_tile = selected_tile_q.get_single_mut();

    match (details_window, selected_tile) {
        (Ok(mut txt), Ok((mut selected_tile_transform, mut selected_tile_visibility))) => {
            if let Some((pos, ..)) = map.find_tile(hex) {
                if let Some(descr) = find_descr_at(&pos, &description_q) {
                    txt.sections[0].value =
                        format!("You look at {:?}, you see...\n{}", pos, descr.as_str());
                } else {
                    txt.sections[0].value = format!("You look at {:?}, there is nothing", pos);
                }
                selected_tile_transform.translation = hex.into_vec3().with_z(Z_LAYER_UI_MAP_MARKER);
                *selected_tile_visibility = Visibility::Inherited;
            } else {
                txt.sections[0].value = "No tile selected".to_string();
                *selected_tile_visibility = Visibility::Hidden;
            }
        }
        _ => {}
    }
}

fn find_descr_at<'a>(
    mpos: &MapPos,
    description_q: &'a Query<(&MapPos, &Description)>,
) -> Option<&'a Description> {
    for (p, desc) in description_q {
        if mpos == p {
            return Some(desc);
        }
    }
    None
}

#[derive(Debug, Event)]
pub struct UiStateTransitionedEvent(pub UiState);

#[derive(Debug, Clone, Resource)]
pub enum UiState {
    Processing,
    Wait(Timer),
    AwaitInput(MapPos),
}

impl UiState {
    pub fn prossing() -> Self {
        Self::Processing
    }

    pub fn wait(millis: u64) -> Self {
        Self::Wait(Timer::new(Duration::from_millis(millis), TimerMode::Once))
    }

    pub fn await_input(mpos: MapPos) -> Self {
        Self::AwaitInput(mpos)
    }

    pub fn is_awaiting_input(&self) -> bool {
        match self {
            Self::AwaitInput(..) => true,
            _ => false,
        }
    }
}

fn update_waiting_state(
    mut commands: Commands,
    time: Res<Time>,
    // wait_until: Option<ResMut<WaitUntil>>,
    mut ui_state: ResMut<UiState>,
) {
    let ui_state = ui_state.as_mut();
    if let UiState::Wait(timer) = ui_state {
        timer.tick(time.delta());

        if timer.finished() {
            commands.trigger(UiStateTransitionedEvent(UiState::prossing()));
        }
    }
}

fn update_available_playeractions(
    mut commands: Commands,
    player_actions: Res<PlayerActions>,
    indicatorq: Query<(Entity, &PlayerActionIndicator)>,
) {
    for (e, _) in indicatorq.iter() {
        commands.entity(e).despawn_descendants();
    }

    let Some(selected_action) = player_actions
        .available_actions
        .get(player_actions.selected_action)
    else {
        return;
    };

    match selected_action {
        Action::MoveAlong { path, .. } => {
            for pos in path {
                commands.spawn((
                    Name::from("Path-Indicator"),
                    SpatialBundle {
                        transform: Transform::from_translation(pos.into_vec3().with_z(1.0)),
                        ..Default::default()
                    },
                    Visual::Single("floor-selected".to_string()),
                    PlayerActionIndicator,
                    OnCombatState,
                ));
            }
        }

        _ => {}
    }
}

fn update_turn_info(turn: Res<Turn>, mut turn_info_q: Query<Mut<Text>, With<TurnInfo>>) {
    let mut text = turn_info_q.single_mut();
    let phase = match turn.turn_phase {
        TurnPhase::StartTurn => "Beginning new turn",
        TurnPhase::BoostActivations => "Boosting activations",
        TurnPhase::PerformActions => "Performing actions",
    };
    text.sections[0].value = format!("Turn: {} - {}", turn.turn_number, phase);
}

#[derive(Component)]
pub struct Description(pub String);

impl Description {
    pub fn new(str: impl ToString) -> Self {
        Self(str.to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

fn update_description_on_activation_changed(
    mut actor_q: Query<(&Actor, &Activations, &Name, Mut<Description>), Changed<Activations>>,
) {
    for (_, activations, name, mut description) in actor_q.iter_mut() {
        description.0 = describe_actor(name, activations);
    }
}

fn describe_actor(name: &Name, activations: &Activations) -> String {
    let active_activations_txt = if let Some(activation) = &activations.active {
        card_descr(&activation.0)
    } else {
        " - ".to_string()
    };

    let remaining_activations_txt = if activations.remaining.is_empty() {
        " - ".to_string()
    } else {
        activations
            .remaining
            .iter()
            .map(|a| card_descr(&a.0))
            .collect::<Vec<_>>()
            .join(", ")
    };

    format!(
        "{}\nActivations:\n - active: {}\n - remaining: {}",
        name, active_activations_txt, remaining_activations_txt
    )
}

fn card_descr(card: &Card) -> String {
    let val = match card.value {
        1 => "Ace".to_string(),
        11 => "Jack".to_string(),
        12 => "Queen".to_string(),
        13 => "King".to_string(),
        _ => format!("{}", card.value),
    };

    let suite = match card.suite {
        Suite::PhysicalStr => "Clubs",
        Suite::PhysicalAg => "Spades",
        Suite::MentalStr => "Hearts",
        Suite::MentalAg => "Diamonds",
        _ => "(Unknown)",
    };

    format!("{} of {}", val, suite)
}

fn spawn_activation_indicators(parent: &mut ChildBuilder, card: &Card, pos: usize) {
    let offset_x = -24.0 + 16.0 * (pos as f32);
    let pos = Vec3::new(offset_x, -32.0, 200.0);

    parent.spawn((
        Visual::Single(card_visual_name(card)),
        SpatialBundle {
            transform: Transform::from_translation(pos),
            ..Default::default()
        },
        UiElement,
    ));
}

fn card_visual_name(card: &Card) -> String {
    let suite = match card.suite {
        Suite::PhysicalStr => "ps",
        Suite::PhysicalAg => "pa",
        Suite::MentalStr => "ms",
        Suite::MentalAg => "ma",
        _ => "?",
    };

    let value = match card.value {
        1 => "A".to_string(),
        11 => "J".to_string(),
        12 => "Q".to_string(),
        13 => "K".to_string(),
        _ => format!("{}", card.value),
    };

    format!("icon-ai-{}-{}", suite, value)
}

fn update_ui_on_state_change(
    trigger: Trigger<UiStateTransitionedEvent>,
    ui_elements_q: Query<Entity, With<UiElement>>,
    actor_q: Query<(Entity, &Activations), With<Actor>>,
    mut commands: Commands,
    mut map_pos_selected_ew: EventWriter<MapPosSelectedEvent>,
    mut ui_state: ResMut<UiState>,
) {
    let UiStateTransitionedEvent(new_ui_state) = trigger.event();

    if let UiState::AwaitInput(mpos) = new_ui_state {
        commands
            .spawn((
                Name::from("DetailsWindow"),
                NodeBundle {
                    background_color: WINDOW_BACKGROUND.into(),
                    style: Style {
                        position_type: PositionType::Absolute,
                        top: Val::Px(20.0),
                        right: Val::Px(20.0),
                        width: Val::Px(200.0),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                OnCombatState,
                UiElement,
            ))
            .with_children(|parent| {
                parent.spawn((
                    TextBundle::from_section(
                        "Test",
                        TextStyle {
                            color: Color::BLACK,
                            font_size: 14.0,
                            ..Default::default()
                        },
                    )
                    .with_style(Style {
                        margin: UiRect::all(Val::Px(10.0)),
                        ..Default::default()
                    }),
                    DetailsWindowText,
                ));
            });

        commands.spawn((
            Name::from("SelectedTile"),
            SpatialBundle {
                visibility: Visibility::Hidden,
                ..Default::default()
            },
            Visual::Single("floor-selected".to_string()),
            SelectedTile,
            OnCombatState,
            UiElement,
        ));

        for (e, activations) in actor_q.iter() {
            commands.entity(e).with_children(|parent| {
                if let Some(Activation(card)) = activations.active {
                    spawn_activation_indicators(parent, &card, 0);
                }

                for (idx, Activation(card)) in activations.remaining.iter().enumerate() {
                    spawn_activation_indicators(parent, &card, idx + 2);
                }
            });
        }

        map_pos_selected_ew.send(MapPosSelectedEvent(*mpos));
    } else {
        if let UiState::AwaitInput(..) = ui_state.as_ref() {
            for e in ui_elements_q.iter() {
                commands.entity(e).despawn_recursive();
            }
        }
    }
    *ui_state.as_mut() = new_ui_state.clone();
}
