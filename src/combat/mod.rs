mod actor;
mod cards;
mod flow;
mod map;
mod ui;

use bevy::prelude::*;

use crate::{assets::Visual, despawn_screen, GameState};

use self::{
    actor::{
        handle_action_selected_event, handle_move_to_command, ActionSelectedEvent, ActivateActor,
        ActorBundle, AiBehaviour, MoveToCommand, Team, TeamBundle,
    },
    flow::{handle_activate_actor, setup_combat_flow, update_combat_flow, CombatFlowEvent, Turn},
    map::{update_obstacles_in_map, HexMap, MapPos},
    ui::{
        setup_turn_info, setup_ui, update_user_input, update_waiting_state, MapPosSelectedEvent,
        PlayerActions, WaitForUser, WaitUntil,
    },
};

#[derive(Component)]
struct OnCombatState;

#[derive(Resource)]
struct ScrollBounds(Rect);

pub fn combat_plugin(app: &mut App) {
    app.add_event::<MapPosSelectedEvent>()
        .add_event::<ActionSelectedEvent>()
        .add_event::<ActivateActor>()
        .add_event::<MoveToCommand>()
        .add_event::<CombatFlowEvent>()
        .observe(handle_action_selected_event)
        .observe(handle_move_to_command)
        .observe(handle_activate_actor)
        .add_systems(
            OnEnter(GameState::Combat),
            (
                (setup_map, setup_camera, setup_actors).chain(),
                setup_combat_flow,
                setup_ui,
                setup_turn_info,
            ),
        )
        .add_systems(
            Update,
            (
                (
                    update_user_input,
                    (ui::handle_select_map_pos, actor::handle_select_map_pos),
                )
                    .chain(),
                ui::update_available_playeractions
                    .run_if(resource_exists_and_changed::<PlayerActions>),
                ui::update_turn_info.run_if(resource_exists_and_changed::<Turn>),
                update_obstacles_in_map,
                update_waiting_state,
                update_combat_flow.run_if(not_waiting),
            )
                .run_if(in_state(GameState::Combat)),
        )
        .add_systems(OnExit(GameState::Combat), despawn_screen::<OnCombatState>);
}

fn setup_map(mut commands: Commands) {
    let map = map::dummy_hex();

    for (hex_pos, _tile_type) in map.tiles() {
        commands.spawn((
            SpriteBundle {
                transform: Transform::from_translation(hex_pos.into_vec3()),
                ..Default::default()
            },
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
) {
    let mut camera_transform = camera_query.single_mut();
    let scroll_zone = Rect::from_corners(
        Vec2::new(map.scroll_limit_x.0, map.scroll_limit_y.0),
        Vec2::new(map.scroll_limit_x.1, map.scroll_limit_y.1),
    );

    *camera_transform = Transform::from_translation(map.camera_focus.into_vec3());

    commands.insert_resource(ScrollBounds(scroll_zone));
}

fn setup_actors(mut commands: Commands) {
    let player_team = Team(commands.spawn(TeamBundle::new("Player", true)).id());
    let cpu_team = Team(commands.spawn(TeamBundle::new("CPU", false)).id());

    commands.spawn(ActorBundle::new(
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
            cpu_team,
            false,
            MapPos::from_oddr(2, 2),
            Visual::Single("monster-sucker_1".to_string()),
        ),
        AiBehaviour::Zombi,
    ));

    commands.spawn((
        ActorBundle::new(
            cpu_team,
            false,
            MapPos::from_oddr(8, 2),
            Visual::Single("monster-sucker_1".to_string()),
        ),
        AiBehaviour::Zombi,
    ));
}

fn not_waiting(
    wait_until: Option<Res<WaitUntil>>,
    wait_for_user: Option<Res<WaitForUser>>,
) -> bool {
    wait_for_user.is_none() && wait_until.is_none()
}
