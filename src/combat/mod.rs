mod actor;
mod ai;
mod combat_fx;
mod combat_resolution;
mod flow;
mod fx;
mod generator;
mod map;
mod ui;

use actor::{
    ActionSelectedEvent, AttackCommand, CombatFinishedEvent, handle_action_triggered_event,
    handle_attack_command,
};
use ai::combat_ai_plugin;
use bevy::prelude::*;
use generator::{ActorGenerator, setup_generators};
use ui::{SelectedMapPos, update_action_buttons};

use crate::{GameState, assets::Visual, core::Deck, despawn_screen};

use self::{
    actor::{
        ActionTriggeredEvent, ActorBundle, AiBehaviour, BeginActivationCommand,
        EndActivationCommand, MoveToCommand, Team, TeamBundle, handle_action_selected_event,
        handle_begin_activation_command, handle_end_activation_command, handle_move_to_command,
    },
    flow::{CombatFlowEvent, setup_combat_flow, update_combat_flow},
    map::{HexMap, MapPos, update_obstacles_in_map},
    ui::{UiState, combat_ui_plugin},
};

#[derive(Component)]
struct OnCombatState;

#[derive(Resource)]
struct ScrollBounds(Rect);

#[derive(Resource)]
struct GameDeck(pub Deck);

pub fn combat_plugin(app: &mut App) {
    app.add_plugins((combat_ui_plugin, combat_ai_plugin))
        .add_event::<ActionTriggeredEvent>()
        .add_event::<ActionSelectedEvent>()
        .add_event::<CombatFinishedEvent>()
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
        .add_observer(actor::handle_combat_finished_event)
        .add_observer(combat_fx::handle_combat_finished_event)
        .add_systems(
            OnEnter(GameState::Combat),
            (
                (setup_map, setup_camera, setup_actors).chain(),
                setup_combat_flow,
                setup_generators,
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

fn setup_actors(mut commands: Commands, actor_generator: Res<ActorGenerator>) {
    let player_team = Team(commands.spawn(TeamBundle::new("Player", true)).id());
    let cpu_team = Team(commands.spawn(TeamBundle::new("CPU", false)).id());
    let deck = Deck::new_rnd();

    commands.spawn((
        ActorBundle::new(
            Name::new("Player"),
            player_team,
            true,
            MapPos::from_oddr(5, 5),
        ),
        actor_generator.generate_actor("player"),
    ));

    commands.spawn((
        ActorBundle::new(
            Name::new("Sucker #1"),
            cpu_team,
            false,
            MapPos::from_oddr(2, 2),
        ),
        actor_generator.generate_actor("sucker"),
        AiBehaviour::Zombi,
    ));

    commands.spawn((
        ActorBundle::new(
            Name::new("Sucker #2"),
            cpu_team,
            false,
            MapPos::from_oddr(8, 2),
        ),
        actor_generator.generate_actor("sucker"),
        AiBehaviour::Zombi,
    ));

    commands.insert_resource(GameDeck(deck));
}

fn progress_game(ui_state: Option<Res<UiState>>) -> bool {
    ui_state
        .map(|ui_state| matches!(ui_state.as_ref(), UiState::Processing))
        .unwrap_or(false)
}
