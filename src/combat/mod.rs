mod actor;
mod ai;
mod combat_fx;
mod commands;
mod flow;
mod fx;
mod generator;
mod map;
mod ui;

use ai::combat_ai_plugin;
use bevy::prelude::*;
use generator::{ActorGenerator, setup_generators};

use crate::{GameState, assets::Visual, core::Deck, despawn_screen};

use self::{
    actor::{ActorBundle, Team, TeamBundle},
    flow::{TurnPhase, handle_turn_phase_start_turn, setup_combat_flow},
    map::{HexMap, MapPos, update_obstacles_in_map},
    ui::combat_ui_plugin,
};

pub use commands::ManeuverTemplates;

#[derive(Component)]
struct OnCombatState;

#[derive(Resource)]
struct ScrollBounds(Rect);

#[derive(Resource)]
struct GameDeck(pub Deck);

pub fn combat_plugin(app: &mut App) {
    app.add_sub_state::<TurnPhase>()
        .add_plugins((combat_ui_plugin, combat_ai_plugin))
        .add_observer(actor::handle_action_triggered_event)
        .add_observer(actor::handle_begin_activation_command)
        .add_observer(actor::handle_end_activation_command)
        .add_observer(actor::handle_move_to_command)
        .add_observer(actor::handle_action_command)
        .add_observer(actor::handle_action_finished_event)
        .add_observer(actor::handle_assign_activation_command)
        .add_observer(actor::on_actor_activated_event)
        .add_observer(actor::on_insert_status_flags)
        .add_observer(actor::on_insert_active_effects)
        .add_observer(commands::on_start_input_workflow_command)
        .add_observer(commands::on_fx_finished)
        .add_observer(commands::on_input_step_completed_event)
        .add_observer(combat_fx::handle_action_finished_event)
        .add_systems(
            OnEnter(GameState::Combat),
            (
                (setup_map, setup_camera, setup_actors).chain(),
                setup_combat_flow,
                setup_generators,
            ),
        )
        .add_systems(OnEnter(TurnPhase::StartTurn), handle_turn_phase_start_turn)
        .add_systems(
            Update,
            (
                update_obstacles_in_map,
                fx::update_check_fx_ready,
                flow::handle_turn_phase_perform_actions.run_if(in_state(TurnPhase::PerformActions)),
            )
                .run_if(in_state(GameState::Combat)),
        )
        .add_systems(OnExit(GameState::Combat), despawn_screen::<OnCombatState>);

    // setup_actor_changed(app);
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
    let mut deck = Deck::new_rnd();

    commands.spawn((
        ActorBundle::new(
            Name::new("Player (Leader)"),
            player_team,
            true,
            MapPos::from_oddr(5, 5),
        ),
        actor_generator.generate_actor("player_leader", &mut deck),
    ));

    commands.spawn((
        ActorBundle::new(
            Name::new("Player (Tank)"),
            player_team,
            true,
            MapPos::from_oddr(5, 6),
        ),
        actor_generator.generate_actor("player_tank", &mut deck),
    ));

    commands.spawn((
        ActorBundle::new(
            Name::new("Sucker #1"),
            cpu_team,
            false,
            MapPos::from_oddr(2, 2),
        ),
        actor_generator.generate_actor("sucker", &mut deck),
    ));

    commands.spawn((
        ActorBundle::new(
            Name::new("Sucker #2"),
            cpu_team,
            false,
            MapPos::from_oddr(8, 2),
        ),
        actor_generator.generate_actor("sucker", &mut deck),
    ));

    commands.insert_resource(GameDeck(deck));
}
