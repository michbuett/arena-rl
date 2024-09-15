use bevy::{input::mouse::MouseMotion, prelude::*, window::PrimaryWindow};

use crate::style::WINDOW_BACKGROUND;

use super::{
    actor::Action,
    flow::{Turn, TurnPhase},
    map::{HexMap, MapPos},
    OnCombatState, ScrollBounds, Visual,
};

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

pub fn setup_ui(mut commands: Commands) {
    commands
        .spawn((
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
        ))
        .with_children(|parent| {
            parent.spawn((
                TextBundle::from_section(
                    "Test",
                    TextStyle {
                        color: Color::BLACK,
                        font_size: 12.0,
                        ..Default::default()
                    },
                ),
                DetailsWindowText,
            ));
        });

    commands.spawn((
        SpriteBundle {
            visibility: Visibility::Hidden,
            ..Default::default()
        },
        Visual::Single("floor-selected".to_string()),
        SelectedTile,
        OnCombatState,
    ));

    commands.insert_resource(ScrollState(false));
    commands.insert_resource(PlayerActions::empty());
}

pub fn update_user_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    dim: Res<ScrollBounds>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
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

pub fn handle_select_map_pos(
    map: Res<HexMap>,
    mut user_input_evr: EventReader<MapPosSelectedEvent>,
    mut details_window_text: Query<&mut Text, With<DetailsWindowText>>,
    mut selected_tile_q: Query<(&mut Transform, &mut Visibility), With<SelectedTile>>,
) {
    let Some(MapPosSelectedEvent(hex)) = user_input_evr.read().last() else {
        return;
    };

    let mut txt = details_window_text.single_mut();
    let (mut selected_tile_transform, mut selected_tile_visibility) = selected_tile_q.single_mut();

    if let Some((pos, ..)) = map.find_tile(hex) {
        txt.sections[0].value = format!("You look at {:?}, there is nothing", pos);
        selected_tile_transform.translation = hex.into_vec3().with_z(1.0);
        *selected_tile_visibility = Visibility::Inherited;
    } else {
        txt.sections[0].value = "No tile selected".to_string();
        *selected_tile_visibility = Visibility::Hidden;
    }
}

#[derive(Debug, Resource)]
pub struct WaitUntil(Timer);

#[derive(Debug, Resource)]
pub struct WaitForUser();

pub fn update_waiting_state(
    mut commands: Commands,
    time: Res<Time>,
    wait_until: Option<ResMut<WaitUntil>>,
) {
    if let Some(mut wait_until) = wait_until {
        wait_until.0.tick(time.delta());

        if wait_until.0.finished() {
            commands.remove_resource::<WaitUntil>();
        }
    }
}

pub fn update_available_playeractions(
    mut commands: Commands,
    player_actions: Res<PlayerActions>,
    indicatorq: Query<(Entity, &PlayerActionIndicator)>,
) {
    for (e, _) in indicatorq.iter() {
        commands.entity(e).despawn();
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
                    Transform::from_translation(pos.into_vec3().with_z(1.0)),
                    Visual::Single("floor-selected".to_string()),
                    PlayerActionIndicator,
                    OnCombatState,
                ));
            }
        }

        _ => {}
    }
}

#[derive(Component)]
pub struct TurnInfo;

pub fn setup_turn_info(mut commands: Commands) {
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

pub fn update_turn_info(turn: Res<Turn>, mut turn_info_q: Query<Mut<Text>, With<TurnInfo>>) {
    let mut text = turn_info_q.single_mut();
    let phase = match turn.turn_phase {
        TurnPhase::StartTurn => "Beginning new turn",
        TurnPhase::BoostActivations => "Boosting activations",
        TurnPhase::PerformActions => "Performing actions",
    };
    text.sections[0].value = format!("Turn: {} - {}", turn.turn_number, phase);
}
