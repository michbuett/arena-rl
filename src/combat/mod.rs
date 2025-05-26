mod actor;
mod ai;
mod combat_resolution;
mod flow;
mod fx;
mod map;
mod ui;

use actor::{
    handle_action_triggered_event, handle_attack_command, ActionSelectedEvent, AttackCommand,
};
use ai::combat_ai_plugin;
use bevy::prelude::*;
use ui::{update_action_buttons, SelectedMapPos};

use crate::{assets::Visual, despawn_screen, GameState};

use self::{
    actor::{
        handle_action_selected_event, handle_begin_activation_command,
        handle_end_activation_command, handle_move_to_command, ActionTriggeredEvent, ActorBundle,
        AiBehaviour, BeginActivationCommand, EndActivationCommand, MoveToCommand, Team, TeamBundle,
    },
    flow::{setup_combat_flow, update_combat_flow, CombatFlowEvent},
    map::{update_obstacles_in_map, HexMap, MapPos},
    ui::{combat_ui_plugin, UiState},
};

#[derive(Component)]
struct OnCombatState;

#[derive(Resource)]
struct ScrollBounds(Rect);

pub fn combat_plugin(app: &mut App) {
    app.add_plugins((combat_ui_plugin, combat_ai_plugin))
        .add_event::<ActionTriggeredEvent>()
        .add_event::<ActionSelectedEvent>()
        .add_event::<BeginActivationCommand>()
        .add_event::<EndActivationCommand>()
        .add_event::<MoveToCommand>()
        .add_event::<AttackCommand>()
        .add_event::<CombatFlowEvent>()
        .add_observer(handle_action_selected_event)
        .add_observer(handle_action_triggered_event)
        .add_observer(handle_begin_activation_command)
        .add_observer(handle_end_activation_command)
        .add_observer(handle_move_to_command)
        .add_observer(handle_attack_command)
        .add_systems(
            OnEnter(GameState::Combat),
            (
                (setup_map, setup_camera, setup_actors).chain(),
                setup_combat_flow,
            ),
        )
        .add_systems(
            Update,
            (
                actor::handle_select_map_pos,
                update_obstacles_in_map,
                fx::update_check_fx_ready,
                update_combat_flow.run_if(progress_game),
                update_action_buttons.run_if(resource_exists_and_changed::<SelectedMapPos>),
            )
                .run_if(in_state(GameState::Combat)),
        )
        .add_systems(OnExit(GameState::Combat), despawn_screen::<OnCombatState>);
}

fn setup_map(mut commands: Commands) {
    let map = map::dummy_hex();

    for (hex_pos, _tile_type) in map.tiles() {
        commands.spawn((
            Name::from(format!("Tile_{:?}", hex_pos)),
            Transform::from_translation(hex_pos.into_vec3()),
            Visual::Single("floor".to_string()),
            OnCombatState,
        ));
    }

    commands.insert_resource(map);
}

fn setup_camera(
    mut commands: Commands,
    mut camera_query: Query<&mut Transform, With<Camera>>,
    map: Res<HexMap>,
) -> Result<(), BevyError> {
    let mut camera_transform = camera_query.single_mut()?;
    let scroll_zone = Rect::from_corners(
        Vec2::new(map.scroll_limit_x.0, map.scroll_limit_y.0),
        Vec2::new(map.scroll_limit_x.1, map.scroll_limit_y.1),
    );

    *camera_transform = Transform::from_translation(map.camera_focus.into_vec3());

    commands.insert_resource(ScrollBounds(scroll_zone));

    Ok(())
}

fn setup_actors(mut commands: Commands) {
    let player_team = Team(commands.spawn(TeamBundle::new("Player", true)).id());
    let cpu_team = Team(commands.spawn(TeamBundle::new("CPU", false)).id());

    commands.spawn(ActorBundle::new(
        Name::new("Player"),
        player_team,
        true,
        MapPos::from_oddr(5, 5),
        Visual::Multi(vec![
            "body-heavy_1".to_string(),
            "head-heavy_1".to_string(),
            "melee-1h_2".to_string(),
        ]),
    ));

    commands.spawn((
        ActorBundle::new(
            Name::new("Sucker #1"),
            cpu_team,
            false,
            MapPos::from_oddr(2, 2),
            Visual::Single("monster-sucker_1".to_string()),
        ),
        AiBehaviour::Zombi,
    ));

    commands.spawn((
        ActorBundle::new(
            Name::new("Sucker #2"),
            cpu_team,
            false,
            MapPos::from_oddr(8, 2),
            Visual::Single("monster-sucker_1".to_string()),
        ),
        AiBehaviour::Zombi,
    ));
}

fn progress_game(ui_state: Option<Res<UiState>>) -> bool {
    ui_state
        .map(|ui_state| matches!(ui_state.as_ref(), UiState::Processing))
        .unwrap_or(false)
}
