use std::time::Duration;

use bevy::color::palettes::tailwind::RED_100;
use bevy::{input::mouse::MouseMotion, prelude::*, window::PrimaryWindow};

use crate::MarkedForDeath;
use crate::combat::actor::{ActionSelectedEvent, ActionTriggeredEvent};
use crate::core::{Card, Health, ItemState, Items, Protection, Suite};
use crate::style::{BUTTON_BG_HIGHLIGHT, TextStyle, text};
use crate::{GameState, style::WINDOW_BACKGROUND};

use super::actor::{AttackData, CombatFinishedEvent};
use super::combat_resolution::CombatConsequence;
use super::{
    OnCombatState, ScrollBounds, Visual,
    actor::{Action, Activation, Activations, Actor},
    flow::{Turn, TurnPhase},
    map::{HexMap, MapPos},
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

#[derive(Component)]
pub struct ActionButtonContainer;

#[derive(Component)]
pub struct CombatLogContainer;

#[derive(Resource)]
pub struct SelectedMapPos {
    active_actor: Entity,
    map_pos: MapPos,
    available_actions: Vec<Action>,
    selected_action: usize,
}

impl SelectedMapPos {
    pub fn new(active_actor: Entity, map_pos: MapPos, available_actions: Vec<Action>) -> Self {
        Self {
            active_actor,
            map_pos,
            available_actions,
            selected_action: 0,
        }
    }

    pub fn select_action(&mut self, new_index: usize) {
        self.selected_action = new_index.clamp(0, self.available_actions.len() - 1);
    }

    pub fn get_selected_action_index(&self) -> usize {
        self.selected_action
    }

    pub fn get_selected_action(&self) -> Option<&Action> {
        self.available_actions.get(self.selected_action)
    }

    pub fn get_selected_action_when_at(&self, other_pos: &MapPos) -> Option<&Action> {
        if self.map_pos == *other_pos {
            self.get_selected_action()
        } else {
            None
        }
    }
}

#[derive(Debug, Event)]
pub struct MapPosSelectedEvent(pub MapPos);

#[derive(Component)]
pub struct UserInputElement;

#[derive(Component)]
#[require(Interaction)] // TODO: consider using new click API
pub struct ActionTrigger {
    index: usize,
    action: Action,
}

#[derive(Component)]
pub struct ActivationIndicator;

pub fn combat_ui_plugin(app: &mut App) {
    app.add_event::<MapPosSelectedEvent>()
        .add_event::<UiStateTransitionedEvent>()
        .add_observer(update_ui_on_state_change)
        .add_observer(handle_combat_finished_event)
        // .add_observer(process_action_button_click)
        .add_systems(OnEnter(GameState::Combat), setup_ui)
        .add_systems(
            Update,
            (
                process_keyboard_input,
                (process_mouse_input, handle_select_map_pos).chain(),
                update_description_on_changed,
                (
                    update_details_window_text_on_change,
                    update_available_playeractions,
                )
                    .run_if(resource_exists_and_changed::<SelectedMapPos>),
                update_turn_info.run_if(resource_exists_and_changed::<Turn>),
                update_waiting_state,
            )
                .run_if(in_state(GameState::Combat)),
        );
}

fn setup_ui(mut commands: Commands) {
    commands.insert_resource(ScrollState(false));
    commands.insert_resource(UiState::prossing());

    commands
        .spawn((
            BackgroundColor(WINDOW_BACKGROUND.into()),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(20.0),
                left: Val::Px(20.0),
                padding: UiRect::all(Val::Px(3.)),
                ..Default::default()
            },
            OnCombatState,
        ))
        .with_children(|parent| {
            parent.spawn((text("Turn: -", TextStyle::UiNormal), TurnInfo));
        });

    commands
        .spawn((
            Name::from("DetailsWindow"),
            BackgroundColor(WINDOW_BACKGROUND.into()),
            Visibility::Hidden,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(20.0),
                right: Val::Px(20.0),
                width: Val::Px(200.0),
                padding: UiRect::all(Val::Px(10.0)),
                ..Default::default()
            },
            OnCombatState,
            UserInputElement,
        ))
        .with_children(|parent| {
            parent.spawn((text("", TextStyle::UiNormal), DetailsWindowText));
        });

    commands.spawn((
        Name::from("Action Button Container"),
        Visibility::Hidden,
        Node {
            position_type: PositionType::Absolute,
            flex_direction: FlexDirection::Column,
            justify_items: JustifyItems::Start,
            align_items: AlignItems::Stretch,
            display: Display::Flex,
            bottom: Val::Px(20.0),
            right: Val::Px(20.0),
            width: Val::Px(200.0),
            ..Default::default()
        },
        OnCombatState,
        UserInputElement,
        ActionButtonContainer,
    ));

    commands.spawn((
        Name::from("SelectedTile"),
        Visibility::Hidden,
        Transform::default(),
        Visual::Single("floor-selected".to_string()),
        SelectedTile,
        OnCombatState,
        UserInputElement,
    ));

    commands.spawn((
        Name::from("Combat log container"),
        // Visibility::Hidden,
        Node {
            position_type: PositionType::Absolute,
            flex_direction: FlexDirection::Column,
            justify_items: JustifyItems::Start,
            align_items: AlignItems::Stretch,
            display: Display::Flex,
            row_gap: Val::Px(5.0),
            top: Val::Px(100.0),
            left: Val::Px(20.0),
            width: Val::Px(200.0),
            padding: UiRect::all(Val::Px(10.0)),
            ..Default::default()
        },
        OnCombatState,
        // UserInputElement,
        CombatLogContainer,
    ));
}

// fn process_action_button_click(click: Trigger<Pointer<Click>>) {
//        println!("{} was clicked!", click.entity());
// }

fn process_mouse_input(
    mut commands: Commands,
    mut mouse_button_input: ResMut<ButtonInput<MouseButton>>,
    selected_map_pos: Option<Res<SelectedMapPos>>,
    dim: Res<ScrollBounds>,
    interaction_q: Query<(&Interaction, &ActionTrigger), Changed<Interaction>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    ui_state: Res<UiState>,
    mut mouse_motion_evr: EventReader<MouseMotion>,
    mut camera_transform_query: Query<&mut Transform, With<Camera>>,
    mut map_pos_selected_ew: EventWriter<MapPosSelectedEvent>,
    mut scroll_state: ResMut<ScrollState>,
) -> Result<(), BevyError> {
    for (interaction, ActionTrigger { index, action }) in interaction_q.iter() {
        match (&selected_map_pos, interaction) {
            (Some(pa), Interaction::Pressed) => {
                if pa.get_selected_action_index() == *index {
                    commands.remove_resource::<SelectedMapPos>();
                    commands.trigger_targets(ActionTriggeredEvent(action.clone()), pa.active_actor);
                } else {
                    commands.trigger(ActionSelectedEvent(*index));
                }

                mouse_button_input.reset_all();
                return Ok(());
            }

            (_, Interaction::Hovered) => {
                // ignore all other mouse interactions while hovering over a button (e.g. no scrolling)
                mouse_button_input.reset_all();
                return Ok(());
            }
            _ => {}
        }
    }

    let mut camera_transform = camera_transform_query.single_mut()?;
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
                return Ok(());
            }

            if let Some(mouse_pos) = window_query.single()?.cursor_position() {
                let (camera, camera_global_transform) = camera_query.single()?;
                if let Ok(wp) = camera.viewport_to_world_2d(camera_global_transform, mouse_pos) {
                    map_pos_selected_ew.write(MapPosSelectedEvent(MapPos::from(wp)));
                }
            }
        }
    }
    Ok(())
}

fn process_keyboard_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    dim: Res<ScrollBounds>,
    mut camera_transform_query: Query<&mut Transform, With<Camera>>,
) -> Result<(), BevyError> {
    let mut camera_transform = camera_transform_query.single_mut()?;

    if keyboard_input.pressed(KeyCode::KeyA) || keyboard_input.pressed(KeyCode::ArrowLeft) {
        move_camera(&mut camera_transform, -10.0, 0.0, &dim);
    } else if keyboard_input.pressed(KeyCode::KeyD) || keyboard_input.pressed(KeyCode::ArrowRight) {
        move_camera(&mut camera_transform, 10.0, 0.0, &dim);
    } else if keyboard_input.pressed(KeyCode::KeyW) || keyboard_input.pressed(KeyCode::ArrowUp) {
        move_camera(&mut camera_transform, 0.0, 10.0, &dim);
    } else if keyboard_input.pressed(KeyCode::KeyS) || keyboard_input.pressed(KeyCode::ArrowDown) {
        move_camera(&mut camera_transform, 0.0, -10.0, &dim);
    }
    Ok(())
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
) -> Result<(), BevyError> {
    let Some(MapPosSelectedEvent(hex)) = user_input_evr.read().last() else {
        return Ok(());
    };

    let mut details_window_txt = details_window_text.single_mut()?;
    let selected_tile = selected_tile_q.single_mut()?;
    let (mut selected_tile_transform, mut selected_tile_visibility) = selected_tile;

    if let Some((pos, ..)) = map.find_tile(hex) {
        let descr_at = description_q
            .iter()
            .filter_map(|(mp, descr)| if pos == *mp { Some(descr) } else { None })
            .next();

        if let Some(descr) = descr_at {
            details_window_txt.0 = format!(
                "You look at {:?}. There is...\n{}",
                pos.coordinates(),
                descr.as_str()
            );
        } else {
            details_window_txt.0 = format!("You look at {:?}. There is nothing", pos.coordinates());
        }

        selected_tile_transform.translation = hex.into_vec3().with_z(Z_LAYER_UI_MAP_MARKER);
        *selected_tile_visibility = Visibility::Inherited;
    } else {
        details_window_txt.0 = "No tile selected".to_string();
        *selected_tile_visibility = Visibility::Hidden;
    }

    Ok(())
}

fn update_details_window_text_on_change(
    description_q: Query<(&MapPos, Ref<Description>), Without<SelectedTile>>,
    selected_map_pos: Res<SelectedMapPos>,
    mut details_window_text_q: Query<&mut Text, With<DetailsWindowText>>,
) -> Result<(), BevyError> {
    let selected_mpos = selected_map_pos.map_pos;
    let mut details_window_text = details_window_text_q.single_mut()?;

    for (mpos, descr) in description_q {
        if descr.is_changed() && *mpos == selected_mpos {
            details_window_text.0 = format!(
                "You look at {:?}, you see...\n{}",
                mpos.coordinates(),
                descr.0.as_str()
            );
        }
    }

    Ok(())
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
        matches!(self, Self::AwaitInput(..))
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
    player_actions: Res<SelectedMapPos>,
    indicatorq: Query<(Entity, &PlayerActionIndicator)>,
) {
    for (e, _) in indicatorq.iter() {
        commands.entity(e).despawn_related::<Children>();
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

fn update_turn_info(turn: Res<Turn>, mut turn_info_q: Query<Mut<Text>, With<TurnInfo>>) {
    if let Ok(mut turn_info) = turn_info_q.single_mut() {
        // let mut text = turn_info_q.single_mut();
        let phase = match turn.turn_phase {
            TurnPhase::StartTurn => "Beginning new turn",
            TurnPhase::BoostActivations => "Boosting activations",
            TurnPhase::PerformActions => "Performing actions",
        };
        turn_info.0 = format!("Turn: {} - {}", turn.turn_number, phase);
        // text.0 = format!("Turn: {} - {}", turn.turn_number, phase);
    }
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

fn update_description_on_changed(
    mut actor_q: Query<
        (
            &Actor,
            &Activations,
            &Name,
            &Health,
            &Protection,
            &Items,
            Mut<Description>,
        ),
        Or<(
            Changed<Activations>,
            Changed<Health>,
            Changed<Items>,
            Changed<Protection>,
        )>,
    >,
) {
    for (_, activations, name, health, protection, items, mut description) in actor_q.iter_mut() {
        description.0 = describe_actor(name, health, protection, activations, items);
    }
}

fn describe_actor(
    name: &Name,
    health: &Health,
    protection: &Protection,
    activations: &Activations,
    items: &Items,
) -> String {
    let active_activations_txt = if let Some(activation) = &activations.active() {
        card_descr(&activation.0)
    } else {
        " - ".to_string()
    };

    let remaining_activations_txt = if activations.remaining().is_empty() {
        " - ".to_string()
    } else {
        activations
            .remaining()
            .iter()
            .rev()
            .map(|a| card_descr(&a.0))
            .collect::<Vec<_>>()
            .join(", ")
    };

    let health_text = describe_actor_condition(health, protection);
    let items_txt = describe_actor_items(items);

    format!(
        "{}\n{}\nActivations:\n - active: {}\n - remaining: {}\n{}",
        name, health_text, active_activations_txt, remaining_activations_txt, items_txt
    )
}

fn describe_actor_condition(health: &Health, protection: &Protection) -> String {
    let condition = if health.wounds.is_empty() {
        "Unharmed"
    } else {
        let dmg = health.damage_total() as f32 * 100.0 / health.max_health as f32;
        if dmg >= 50.0 {
            "Heavily wounded"
        } else {
            "Wounded"
        }
    };

    format!(
        "Condition: {} ({}/{}), Armor: {}",
        condition,
        health.damage_total(),
        health.max_health,
        protection.total_resistance()
    )
}

fn describe_actor_items(items: &Items) -> String {
    if items.0.is_empty() {
        "(carries nothing)".to_string()
    } else {
        format!(
            "Equipped with:\n - {}",
            items
                .0
                .iter()
                .map(|item| {
                    let state = match item.state {
                        ItemState::New => "new",
                        ItemState::Damaged => "damaged",
                        ItemState::Broken => "broken",
                    };
                    format!("{} ({})", item.name, state)
                })
                .collect::<Vec<_>>()
                .join("\n - ")
        )
    }
}

fn card_descr(card: &Card) -> String {
    let val = match card.value_low() {
        1 => "Ace".to_string(),
        11 => "Jack".to_string(),
        12 => "Queen".to_string(),
        13 => "King".to_string(),
        _ => format!("{}", card.value_low()),
    };

    let suite = match card.suite() {
        Suite::Clubs => "Clubs",
        Suite::Spades => "Spades",
        Suite::Hearts => "Hearts",
        Suite::Diamonds => "Diamonds",
    };

    format!("{} of {}", val, suite)
}

fn spawn_activation_indicators(parent: &mut ChildSpawnerCommands, card: &Card, pos: usize) {
    let offset_x = -24.0 + 16.0 * (pos as f32);
    let pos = Vec3::new(offset_x, -32.0, 200.0);

    parent.spawn((
        Visual::Single(card_visual_name(card)),
        Transform::from_translation(pos),
        UserInputElement,
        ActivationIndicator,
    ));
}

fn card_visual_name(card: &Card) -> String {
    let suite = match card.suite() {
        Suite::Clubs => "ps",
        Suite::Spades => "pa",
        Suite::Hearts => "ms",
        Suite::Diamonds => "ma",
    };

    let value = match card.value_low() {
        1 => "A".to_string(),
        11 => "J".to_string(),
        12 => "Q".to_string(),
        13 => "K".to_string(),
        _ => format!("{}", card.value_low()),
    };

    format!("icon-ai-{}-{}", suite, value)
}

fn update_ui_on_state_change(
    trigger: Trigger<UiStateTransitionedEvent>,
    actor_q: Query<(Entity, &Activations), With<Actor>>,
    activation_indicator_q: Query<Entity, With<ActivationIndicator>>,
    mut commands: Commands,

    mut ui_elements_q: Query<Mut<Visibility>, With<UserInputElement>>,
    mut map_pos_selected_ew: EventWriter<MapPosSelectedEvent>,
    mut ui_state: ResMut<UiState>,
) {
    let UiStateTransitionedEvent(new_ui_state) = trigger.event();

    if let UiState::AwaitInput(mpos) = new_ui_state {
        for mut visibility in ui_elements_q.iter_mut() {
            *visibility = Visibility::Visible;
        }

        for (e, activations) in actor_q.iter() {
            commands.entity(e).with_children(|parent| {
                if let Some(Activation(card)) = activations.active() {
                    spawn_activation_indicators(parent, &card, 0);
                }

                for (idx, Activation(card)) in activations.remaining().iter().enumerate() {
                    spawn_activation_indicators(parent, card, idx + 2);
                }
            });
        }

        map_pos_selected_ew.write(MapPosSelectedEvent(*mpos));
    } else {
        if let UiState::AwaitInput(..) = ui_state.as_ref() {
            for mut visibility in ui_elements_q.iter_mut() {
                *visibility = Visibility::Hidden;
            }
        }
        for e in activation_indicator_q.iter() {
            commands.entity(e).insert(MarkedForDeath);
        }
    }

    *ui_state.as_mut() = new_ui_state.clone();
}

pub fn update_action_buttons(
    mut commands: Commands,
    player_actions: Res<SelectedMapPos>,
    button_container_q: Query<Entity, With<ActionButtonContainer>>,
) {
    let Ok(container_entity) = button_container_q.single() else {
        // container does not exist
        // => ignore change
        return;
    };

    commands
        .entity(container_entity)
        .despawn_related::<Children>();
    commands.entity(container_entity).with_children(|parent| {
        for (idx, action) in player_actions.available_actions.iter().enumerate() {
            let (background_color, border_color) = if idx == player_actions.selected_action {
                (BUTTON_BG_HIGHLIGHT, RED_100.into())
            } else {
                (WINDOW_BACKGROUND, Color::BLACK)
            };

            let txt = button_text_for_action(action);

            parent
                .spawn((
                    Name::new(format!("Button [{}]", txt)),
                    BackgroundColor(background_color),
                    BorderColor(border_color),
                    Node {
                        display: Display::Block,
                        width: Val::Percent(100.0),
                        border: UiRect::all(Val::Px(3.0)),
                        margin: UiRect::vertical(Val::Px(5.0)),
                        padding: UiRect::all(Val::Px(10.0)),
                        ..Default::default()
                    },
                    OnCombatState,
                    UserInputElement,
                    ActionTrigger {
                        index: idx,
                        action: action.clone(),
                    },
                ))
                .with_children(|button| {
                    button.spawn(text(txt, TextStyle::UiNormal));
                });
        }
    });
}

fn button_text_for_action(action: &Action) -> String {
    match action {
        Action::MoveAlong { .. } => "Move",
        Action::Attack(AttackData { name, .. }) => name,
        _ => "Unknown",
    }
    .to_string()
}

pub fn handle_combat_finished_event(
    trigger: Trigger<CombatFinishedEvent>,
    actor_data_q: Query<(&Name, &Health)>,
    log_container_q: Query<Entity, With<CombatLogContainer>>,
    children_q: Query<&Children>,
    mut commands: Commands,
) -> Result<(), BevyError> {
    let CombatFinishedEvent {
        attacker,
        target,
        result,
    } = trigger.event();
    let [(attacker_name, _), (target_name, _)] = actor_data_q.get_many([*attacker, *target])?;
    let combat_log_ct = log_container_q.single()?;

    let effect_txt = if result.is_empty() {
        "Miss".to_string()
    } else {
        result
            .iter()
            .map({
                |(_, c)| match c {
                    CombatConsequence::ArmorBreak => "Armor -1".to_string(),
                    CombatConsequence::ClumsyAttack => "Fumble".to_string(),
                    CombatConsequence::Wound { damage } => {
                        let dmg: u8 = damage.iter().map(|c| c.value_high()).sum();
                        format!("Hit for {} damage", dmg)
                    }
                }
            })
            .collect::<Vec<_>>()
            .join(", ")
    };

    let txt = format!("{} attacks {}: {}", attacker_name, target_name, effect_txt);
    let log_entry = commands
        .spawn((
            Node {
                padding: UiRect::all(Val::Px(5.0)),
                ..Default::default()
            },
            BackgroundColor(WINDOW_BACKGROUND.into()),
            children![text(txt, TextStyle::UiNormal),],
        ))
        .id();

    commands
        .entity(combat_log_ct)
        .insert_children(0, &[log_entry]);

    if children_q.contains(combat_log_ct) {
        for (idx, entity) in children_q.get(combat_log_ct)?.iter().enumerate() {
            if idx >= 4 {
                commands.entity(entity).insert(MarkedForDeath);
            }
        }
    }

    Ok(())
}
