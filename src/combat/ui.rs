use bevy::color::palettes::tailwind::RED_100;
use bevy::{input::mouse::MouseMotion, prelude::*, window::PrimaryWindow};

use crate::MarkedForDeath;
use crate::combat::actor::{ActionSelectedEvent, ActionTriggeredEvent};
use crate::core::{Card, Hand, Health, ItemState, Items, Protection, Suite};
use crate::style::{TextStyle, WINDOW_BACKGROUND_HL, WINDOW_BACKGROUND_TANSPARENT, text};
use crate::{GameState, style::WINDOW_BACKGROUND};

use super::actor::{
    ActivationEndedEvent, Active, AttackCommandData, CombatFinishedEvent, Controller,
    PossibleUserActions, RiskComplications, Team,
};
use super::combat_resolution::CombatConsequence;
use super::fx::FxRunning;
use super::{
    OnCombatState, ScrollBounds, Visual,
    actor::{Action, Activations, Actor},
    flow::{TurnNumber, TurnPhase},
    map::{HexMap, MapPos},
};

pub const Z_LAYER_FLOOR: f32 = 1.0;
pub const Z_LAYER_UI_MAP_MARKER: f32 = 10.0;
pub const Z_LAYER_ACTOR: f32 = 100.0;
pub const Z_LAYER_VFX: f32 = 200.0;

#[derive(Component)]
pub struct TurnInfo;

#[derive(Component)]
pub struct DetailsWindow;

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
pub struct SelectedMapPos(pub MapPos);

#[derive(Debug, Event)]
pub struct MapPosSelectedEvent(pub Option<MapPos>);

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

#[derive(Component)]
pub struct PlayerHandWindow;

#[derive(Component)]
pub struct PlayerHandCard(usize);

pub fn combat_ui_plugin(app: &mut App) {
    app.add_event::<MapPosSelectedEvent>()
        .add_observer(handle_combat_finished_event)
        .add_observer(handle_map_pos_selected_event)
        .add_observer(update_indicators_when_activation_ended)
        .add_systems(OnEnter(GameState::Combat), setup_ui)
        .add_systems(
            Update,
            (
                process_keyboard_input,
                process_mouse_input,
                update_description_on_changed,
                update_player_hand_window,
                update_details_window_text_on_change.run_if(resource_exists::<PossibleUserActions>),
                (update_available_playeractions,).run_if(resource_exists::<PossibleUserActions>),
                update_indicators_when_activations_changed,
                update_turn_info.run_if(state_changed::<TurnPhase>),
                update_action_buttons.run_if(resource_changed_or_removed::<PossibleUserActions>),
            )
                .run_if(in_state(GameState::Combat)),
        );
}

fn setup_ui(mut commands: Commands) {
    commands.insert_resource(ScrollState(false));

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
            DetailsWindow,
        ))
        .with_children(|parent| {
            parent.spawn((text("", TextStyle::UiNormal), DetailsWindowText));
        });

    commands.spawn((
        Name::from("Action Button Container"),
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
        CombatLogContainer,
    ));
}

fn process_mouse_input(
    selected_map_pos: Option<Res<PossibleUserActions>>,
    dim: Res<ScrollBounds>,
    map: Res<HexMap>,
    fx_running_q: Query<(), With<FxRunning>>,
    interaction_q: Query<(&Interaction, &ActionTrigger), Changed<Interaction>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,

    mut commands: Commands,
    mut mouse_button_input: ResMut<ButtonInput<MouseButton>>,
    mut mouse_motion_evr: EventReader<MouseMotion>,
    mut camera_transform_query: Query<&mut Transform, With<Camera>>,
    mut scroll_state: ResMut<ScrollState>,
) -> Result<(), BevyError> {
    for (interaction, ActionTrigger { index, action }) in interaction_q.iter() {
        match (&selected_map_pos, interaction) {
            (Some(pa), Interaction::Pressed) => {
                if pa.get_selected_action_index() == *index {
                    commands.remove_resource::<PossibleUserActions>();
                    commands.trigger(ActionTriggeredEvent(action.clone()));
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
            if !fx_running_q.is_empty() {
                // there is other stuff going on
                // => just ignore user other input then scrolling
                return Ok(());
            }

            if let Some(mouse_pos) = window_query.single()?.cursor_position() {
                let (camera, camera_global_transform) = camera_query.single()?;
                if let Ok(wp) = camera.viewport_to_world_2d(camera_global_transform, mouse_pos) {
                    if let Some((hex, ..)) = map.find_tile(&MapPos::from(wp)) {
                        commands.insert_resource(SelectedMapPos(hex));
                        commands.trigger(MapPosSelectedEvent(Some(hex)));
                    } else {
                        commands.remove_resource::<SelectedMapPos>();
                        commands.trigger(MapPosSelectedEvent(None));
                    }
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

fn handle_map_pos_selected_event(
    trigger: Trigger<MapPosSelectedEvent>,
    description_q: Query<(&MapPos, &Description)>,
    details_window_q: Query<Entity, With<DetailsWindow>>,
    mut details_window_text_q: Query<&mut Text, With<DetailsWindowText>>,
    mut selected_tile_q: Query<(Entity, &mut Transform), With<SelectedTile>>,
    mut visibility_q: Query<&mut Visibility>,
) -> Result<(), BevyError> {
    let MapPosSelectedEvent(hex) = trigger.event();
    let details_window = details_window_q.single()?;
    let mut details_window_txt = details_window_text_q.single_mut()?;
    let (selected_tile, mut selected_tile_transform) = selected_tile_q.single_mut()?;
    let [mut details_window_visibility, mut selected_tile_visibility] =
        visibility_q.get_many_mut([details_window, selected_tile])?;

    if let Some(pos) = hex {
        let descr_at = description_q
            .iter()
            .filter_map(|(mp, descr)| if pos == mp { Some(descr) } else { None })
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

        selected_tile_transform.translation = pos.into_vec3().with_z(Z_LAYER_UI_MAP_MARKER);
        *details_window_visibility = Visibility::Inherited;
        *selected_tile_visibility = Visibility::Inherited;
    } else {
        details_window_txt.0 = "No tile".to_string();
        *details_window_visibility = Visibility::Hidden;
        *selected_tile_visibility = Visibility::Hidden;
    }

    Ok(())
}

fn update_details_window_text_on_change(
    description_q: Query<(&MapPos, Ref<Description>), Without<SelectedTile>>,
    selected_map_pos: Option<Res<SelectedMapPos>>,
    mut details_window_text_q: Query<&mut Text, With<DetailsWindowText>>,
) -> Result<(), BevyError> {
    let Some(selected_mpos) = selected_map_pos else {
        // No valid tile selected at the moment
        // => ignore change; when there is a new selection the text will be updated again
        return Ok(());
    };

    let mut details_window_text = details_window_text_q.single_mut()?;

    for (mpos, descr) in description_q {
        if descr.is_changed() && *mpos == selected_mpos.0 {
            details_window_text.0 = format!(
                "You look at {:?}, you see...\n{}",
                mpos.coordinates(),
                descr.0.as_str()
            );
        }
    }

    Ok(())
}

fn update_available_playeractions(
    mut commands: Commands,
    player_actions: Res<PossibleUserActions>,
    indicatorq: Query<(Entity, &PlayerActionIndicator)>,
) {
    for (e, _) in indicatorq.iter() {
        commands.entity(e).despawn_related::<Children>();
    }

    match player_actions.get_selected_action() {
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

fn update_turn_info(
    turn_number: Res<TurnNumber>,
    turn_phase: Res<State<TurnPhase>>,
    mut turn_info_q: Query<Mut<Text>, With<TurnInfo>>,
) {
    if let Ok(mut turn_info) = turn_info_q.single_mut() {
        let phase = match turn_phase.get() {
            TurnPhase::StartTurn => "Beginning new turn",
            TurnPhase::BoostActivations => "Boosting activations",
            TurnPhase::PerformActions => "Performing actions",
        };
        turn_info.0 = format!("Turn: {} - {}", turn_number.0, phase);
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
            Option<&Active>,
            &Activations,
            &Name,
            &Health,
            &Protection,
            &Items,
            Mut<Description>,
        ),
        Or<(
            Changed<Active>,
            Changed<Activations>,
            Changed<Health>,
            Changed<Items>,
            Changed<Protection>,
        )>,
    >,
) {
    for (_, active, activations, name, health, protection, items, mut description) in
        actor_q.iter_mut()
    {
        // println!("UPDATE description of actor {}", name);
        description.0 = describe_actor(name, health, protection, (active, activations), items);
    }
}

fn describe_actor(
    name: &Name,
    health: &Health,
    protection: &Protection,
    activations: (Option<&Active>, &Activations),
    items: &Items,
) -> String {
    let active_activations_txt = if let Some(Active(activation_card)) = activations.0 {
        card_descr(&activation_card)
    } else {
        " - ".to_string()
    };

    let remaining_activations_txt = if activations.1.remaining().is_empty() {
        " - ".to_string()
    } else {
        activations
            .1
            .remaining()
            .iter()
            .rev()
            .map(card_descr)
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

fn spawn_activation_indicators(
    parent: &mut ChildSpawnerCommands,
    card: &Card,
    (offset_x, offset_y): (f32, f32),
) {
    // let offset_x = -24.0 + 16.0 * (pos as f32);
    // let pos = Vec3::new(offset_x, -64.0, Z_LAYER_VFX);

    parent.spawn((
        Visual::Single(card_visual_name(card)),
        Transform::from_translation(Vec3::new(offset_x, offset_y, Z_LAYER_VFX)),
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

fn update_indicators_when_activation_ended(
    trigger: Trigger<ActivationEndedEvent>,
    actor_q: Query<&Activations>,
    activation_indicator_q: Query<(Entity, &ChildOf), With<ActivationIndicator>>,
    mut commands: Commands,
) -> Result<(), BevyError> {
    let ActivationEndedEvent(e) = trigger.event();
    let activations = actor_q.get(*e)?;

    clear_activation_indicator_for(activation_indicator_q, &mut commands, *e);
    insert_activation_indicator_for_entity(&mut commands, *e, activations, None);

    Ok(())
}

fn update_indicators_when_activations_changed(
    actor_q: Query<
        (Entity, &Activations, Option<&Active>),
        Or<(Changed<Activations>, Added<Active>)>,
    >,
    activation_indicator_q: Query<(Entity, &ChildOf), With<ActivationIndicator>>,
    mut commands: Commands,
) {
    for (e, activations, active) in actor_q.iter() {
        clear_activation_indicator_for(activation_indicator_q, &mut commands, e);
        insert_activation_indicator_for_entity(&mut commands, e, activations, active);
    }
}

fn clear_activation_indicator_for(
    activation_indicator_q: Query<'_, '_, (Entity, &ChildOf), With<ActivationIndicator>>,
    commands: &mut Commands<'_, '_>,
    e: Entity,
) {
    for (child_entity, child_of) in activation_indicator_q.iter() {
        if child_of.parent() == e {
            commands.entity(child_entity).insert(MarkedForDeath);
        }
    }
}

fn insert_activation_indicator_for_entity(
    commands: &mut Commands,
    entity: Entity,
    activations: &Activations,
    active: Option<&Active>,
) {
    commands.entity(entity).with_children(|parent| {
        if let Some(Active(card)) = active {
            spawn_activation_indicators(parent, &card, (0.0, 72.0));
        }

        let num_remaining = activations.remaining().len();
        let card_size = 32;
        for (idx, card) in activations.remaining().iter().enumerate() {
            let offset_x = (idx * card_size) as f32 - (card_size / 2 * (num_remaining - 1)) as f32;

            spawn_activation_indicators(parent, card, (offset_x, -48.0));
        }
    });
}

#[derive(Component)]
pub struct RiskComplicationsToggleText;

pub fn update_action_buttons(
    mut commands: Commands,
    player_actions: Option<Res<PossibleUserActions>>,
    button_container_q: Query<Entity, With<ActionButtonContainer>>,
    active_actor_q: Query<(&Active, Option<&RiskComplications>)>,
) {
    let Ok(container_entity) = button_container_q.single() else {
        // container does not exist
        // => ignore change
        return;
    };

    commands
        .entity(container_entity)
        .despawn_related::<Children>();

    if let Some((_, risk_complications)) = active_actor_q.iter().next() {
        let txt = risk_complication_toggle_txt(risk_complications.is_some());

        commands.entity(container_entity).with_children(|parent| {
            parent
                .spawn((
                    Name::from("Action Stance Selector"),
                    BackgroundColor(WINDOW_BACKGROUND.into()),
                    BorderRadius::top(Val::Px(20.)),
                    Node {
                        display: Display::Block,
                        width: Val::Percent(100.0),
                        margin: UiRect::vertical(Val::Px(5.0)),
                        padding: UiRect::all(Val::Px(10.0)),
                        ..Default::default()
                    },
                    OnCombatState,
                    children![(text(txt, TextStyle::UiNormal), RiskComplicationsToggleText),],
                ))
                .observe(toggle_risk_complication);
        });
    }

    if let Some(player_actions) = player_actions {
        commands.entity(container_entity).with_children(|parent| {
            for (idx, action) in player_actions.available_actions().enumerate() {
                let (background_color, border_color) =
                    if idx == player_actions.get_selected_action_index() {
                        (WINDOW_BACKGROUND_HL, RED_100.into())
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
}

fn button_text_for_action(action: &Action) -> String {
    match action {
        Action::MoveAlong { .. } => "Move",
        Action::Attack(AttackCommandData { name, .. }) => name,
        Action::NoOp(..) => "Do Nothing",
        Action::EndPlanningPhase(..) => "Start the Action",
        Action::AssignActivation { .. } => "Assign Activation",
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

    let effect_txt = if result.attack_quality_result.success {
        if result.consequences.is_empty() {
            "Blocked".to_string()
        } else {
            result
                .consequences
                .iter()
                .map({
                    |(_, c)| match c {
                        CombatConsequence::ArmorBreak => "Armor -1".to_string(),
                        CombatConsequence::ClumsyAttack => {
                            format!("{} was thrown off balance", attacker_name)
                        }
                        CombatConsequence::Wound { damage } => {
                            let dmg: u8 = damage.iter().map(|c| c.value_high()).sum();
                            format!("Hit for {} damage", dmg)
                        }
                    }
                })
                .collect::<Vec<_>>()
                .join(", ")
        }
    } else {
        "Miss".to_string()
    };

    let attack_details_txt = format!(
        "{} VS T:{}",
        short_format_cards(&result.attack_quality_result.cards),
        result.attack_quality_result.tn.0
    );

    let txt = format!(
        "{} attacks {}: {}\n{}",
        attacker_name, target_name, attack_details_txt, effect_txt
    );
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

fn short_format_cards(cards: &Vec<Card>) -> String {
    cards
        .iter()
        .map(|card| {
            let val = card.value_high();
            let suite = match card.suite() {
                Suite::Clubs => "C",
                Suite::Spades => "S",
                Suite::Hearts => "H",
                Suite::Diamonds => "D",
            };
            format!("[{}{}]", val, suite)
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn update_player_hand_window(
    mut commands: Commands,
    player_hand_window_q: Query<(Entity, &Children, &Team), With<PlayerHandWindow>>,
    hand_q: Query<(Entity, &Hand, &Controller), Changed<Hand>>,
) -> Result<(), BevyError> {
    for (team, hand, Controller(is_player)) in hand_q.iter() {
        if !*is_player {
            continue;
        }

        let phw = player_hand_window_q.iter().find(|(_, _, t)| t.0 == team);
        let mut phw_entity_commands = if let Some((e, cn, ..)) = phw {
            for c in cn.iter() {
                commands.entity(c).insert(MarkedForDeath);
            }
            commands.entity(e)
        } else {
            commands.spawn((
                Name::new("PlayerHandWindow"),
                BackgroundColor(WINDOW_BACKGROUND_TANSPARENT.into()),
                BorderRadius::all(Val::Px(5.)),
                Node {
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(20.0),
                    left: Val::Px(20.0),
                    margin: UiRect::all(Val::Px(3.)),
                    ..Default::default()
                },
                OnCombatState,
                UserInputElement,
                PlayerHandWindow,
                Team(team),
            ))
        };

        phw_entity_commands.with_children(|parent| {
            if hand.is_empty() {
                parent.spawn((
                    Name::new("Empty hand info"),
                    Node {
                        width: Val::Px(50.),
                        height: Val::Px(80.),
                        padding: UiRect::all(Val::Px(5.)),
                        margin: UiRect::all(Val::Px(5.)),
                        ..Default::default()
                    },
                    children![text("Your hand is empty", TextStyle::UiNormal),],
                ));
            } else {
                for (index, card) in hand.cards().enumerate() {
                    let background_color = if hand.is_selected(index) {
                        WINDOW_BACKGROUND_HL
                    } else {
                        WINDOW_BACKGROUND
                    };

                    parent
                        .spawn((
                            Name::new("PlayerHandCard"),
                            BackgroundColor(background_color),
                            BorderRadius::all(Val::Px(5.)),
                            Node {
                                width: Val::Px(50.),
                                height: Val::Px(80.),
                                padding: UiRect::all(Val::Px(5.)),
                                margin: UiRect::all(Val::Px(5.)),
                                ..Default::default()
                            },
                            PlayerHandCard(index),
                            Team(team),
                            children![text(card_descr(card), TextStyle::UiNormal),],
                        ))
                        .observe(on_click_hand_card);
                }
            }
        });
    }

    Ok(())
}

fn on_click_hand_card(
    click: Trigger<Pointer<Click>>,
    hand_cards_q: Query<(&PlayerHandCard, &Team)>,
    mut hand_q: Query<Mut<Hand>>,
) -> Result<(), BevyError> {
    let (PlayerHandCard(index), Team(team_entity)) = hand_cards_q.get(click.target())?;
    let mut hand = hand_q.get_mut(*team_entity)?;

    hand.toggle_selection(*index);

    Ok(())
}

fn toggle_risk_complication(
    mut event: Trigger<Pointer<Click>>,
    active_actor_q: Query<(Entity, &Active, Option<&RiskComplications>)>,
    mut rc_toggle_text_q: Query<Mut<Text>, With<RiskComplicationsToggleText>>,
    mut commands: Commands,
) -> Result<(), BevyError> {
    let Some((e, _, risk_complications)) = active_actor_q.iter().next() else {
        return Ok(());
    };

    if risk_complications.is_some() {
        commands.entity(e).remove::<RiskComplications>();
    } else {
        commands.entity(e).insert(RiskComplications);
    }

    let mut text = rc_toggle_text_q.single_mut()?;
    text.0 = risk_complication_toggle_txt(!risk_complications.is_some());

    event.propagate(false);

    Ok(())
}

fn risk_complication_toggle_txt(risk_complications: bool) -> String {
    if risk_complications {
        "Push Your Luck"
    } else {
        "Play Save"
    }
    .to_string()
}
