mod actor;
mod animation;
mod cards;
mod flow;
mod map;
mod ui;

use std::time::Duration;

use bevy::{ecs::system::EntityCommands, prelude::*};

use rand::{
    distributions::uniform::{SampleRange, SampleUniform},
    prelude::*,
};

use crate::{
    assets::{SpriteConfig, SpriteConfigMap},
    despawn_screen, GameState,
};

use self::{
    actor::{
        handle_action_selected_event, handle_move_to_command, ActionSelectedEvent, ActivateActor,
        ActorBundle, AiBehaviour, MoveToCommand, Team, TeamBundle,
    },
    animation::update_movement_animation,
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
                update_movement_animation,
                update_sprites_from_visuals,
                update_sprite_animation,
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

#[derive(Debug, Component)]
struct SpriteAnimation {
    indices: Vec<usize>,
    current_idx: usize,
    timer: Timer,
}

fn update_sprite_animation(
    mut animations: Query<(&mut TextureAtlas, &mut SpriteAnimation)>,
    time: Res<Time>,
) {
    for (mut atlas, mut anim) in animations.iter_mut() {
        anim.timer.tick(time.delta());

        if anim.timer.finished() {
            if anim.current_idx < anim.indices.len() - 1 {
                anim.current_idx += 1;
            } else {
                anim.current_idx = 0;
            }

            atlas.index = anim.indices[anim.current_idx];
        }
    }
}

fn rand_between<R, T>(r: R) -> T
where
    T: SampleUniform,
    R: SampleRange<T>,
{
    let mut rng = thread_rng();
    rng.gen_range(r)
}

#[derive(Component)]
pub enum Visual {
    Single(String),
    Multi(Vec<String>),
}

fn update_sprites_from_visuals(
    mut commands: Commands,
    sprite_cfg_map: Res<SpriteConfigMap>,
    layouts: Res<Assets<TextureAtlasLayout>>,
    visual_q: Query<(Entity, &Visual, &Transform), Changed<Visual>>,
) {
    if visual_q.is_empty() {
        return;
    }

    let Some(layout) = layouts.get(sprite_cfg_map.layout.id()) else {
        panic!("Could not find layout in assets for sprite map")
    };

    for (entity, visual, transform) in visual_q.iter() {
        let mut entity_commands = commands.entity(entity);

        entity_commands.clear_children();

        match visual {
            Visual::Single(v) => {
                insert_sprite(
                    entity_commands,
                    &sprite_cfg_map,
                    layout,
                    v,
                    transform.translation,
                );
            }

            Visual::Multi(visuals) => {
                for (idx, v) in visuals.iter().enumerate() {
                    entity_commands.with_children(|parent| {
                        let c = parent.spawn_empty();
                        insert_sprite(c, &sprite_cfg_map, layout, v, Vec3::ZERO.with_y(idx as f32));
                    });
                }
            }
        }
    }
}

fn insert_sprite(
    mut commands: EntityCommands,
    sprite_cfg_map: &Res<SpriteConfigMap>,
    layout: &TextureAtlasLayout,
    visual: &str,
    translation: Vec3,
) {
    match sprite_cfg_map.map.get(visual) {
        Some(SpriteConfig::Single {
            image_id,
            offset: (dx, dy),
        }) => {
            let index = layout.get_texture_index(*image_id).unwrap_or(0);

            commands.insert((
                SpriteBundle {
                    texture: sprite_cfg_map.texture.clone(),
                    transform: Transform::from_translation(translation + Vec3::new(*dx, *dy, 0.0)),
                    ..Default::default()
                },
                TextureAtlas {
                    layout: sprite_cfg_map.layout.clone(),
                    index,
                },
            ));
        }

        Some(SpriteConfig::Animated {
            image_ids,
            offset: (dx, dy),
            frame_duration,
        }) => {
            let indices = image_ids
                .iter()
                .map(|image_id| layout.get_texture_index(*image_id).unwrap_or(0))
                .collect::<Vec<_>>();

            let current_idx: usize = rand_between(0..indices.len());
            let mut timer = Timer::new(
                Duration::from_millis(*frame_duration as u64),
                TimerMode::Repeating,
            );

            timer.tick(Duration::from_millis(
                rand_between(0..*frame_duration) as u64
            ));

            commands.insert((
                SpriteBundle {
                    texture: sprite_cfg_map.texture.clone(),
                    transform: Transform::from_translation(translation + Vec3::new(*dx, *dy, 0.0)),
                    ..Default::default()
                },
                TextureAtlas {
                    layout: sprite_cfg_map.layout.clone(),
                    index: *indices.first().unwrap(),
                },
                SpriteAnimation {
                    indices,
                    current_idx,
                    timer,
                },
            ));
        }

        _ => {}
    }
}

fn not_waiting(
    wait_until: Option<Res<WaitUntil>>,
    wait_for_user: Option<Res<WaitForUser>>,
) -> bool {
    wait_for_user.is_none() && wait_until.is_none()
}
