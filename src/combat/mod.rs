mod map;
mod ui;

use std::time::Duration;

use bevy::{input::mouse::MouseMotion, prelude::*};

use rand::{
    distributions::uniform::{SampleRange, SampleUniform},
    prelude::*,
};

use crate::{
    assets::{SpriteConfig, SpriteConfigMap},
    despawn_screen, GameState,
};

use self::map::{HexMap, MapPosHex, TileType};

#[derive(Component)]
struct OnCombatState;

#[derive(Resource)]
struct ScrollBounds(Rect);

pub fn combat_plugin(app: &mut App) {
    app.add_systems(
        OnEnter(GameState::Combat),
        (setup_map, setup_camera, setup_actors).chain(),
    )
    .add_systems(
        Update,
        (update_camera_after_scrolling, update_sprite_animation)
            .run_if(in_state(GameState::Combat)),
    )
    .add_systems(OnExit(GameState::Combat), despawn_screen::<OnCombatState>);
}

fn setup_map(mut commands: Commands, asset_server: Res<AssetServer>) {
    let map = map::dummy_hex();

    for (hex_pos, tile_type) in map.tiles() {
        if let TileType::Void = tile_type {
            continue;
        }

        commands.spawn((
            SpriteBundle {
                transform: Transform::from_translation(hex_pos.into_vec3()),
                texture: asset_server.load("images/combat/map/floor-hex-1.png"),
                ..Default::default()
            },
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

fn setup_actors(
    mut commands: Commands,
    sprite_cfg_map: Res<SpriteConfigMap>,
    layouts: Res<Assets<TextureAtlasLayout>>,
) {
    create_actor(
        &mut commands,
        vec![
            "body-heavy_1".to_string(),
            "head-heavy_1".to_string(),
            "melee-1h_2".to_string(),
        ],
        MapPosHex::from_oddr(5, 5),
        &sprite_cfg_map,
        &layouts,
    );

    create_actor(
        &mut commands,
        vec!["monster-sucker_1".to_string()],
        MapPosHex::from_oddr(2, 2),
        &sprite_cfg_map,
        &layouts,
    );
    create_actor(
        &mut commands,
        vec!["monster-sucker_1".to_string()],
        MapPosHex::from_oddr(8, 2),
        &sprite_cfg_map,
        &layouts,
    );
}

fn update_camera_after_scrolling(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    dim: Res<ScrollBounds>,
    mut mouse_motion_evr: EventReader<MouseMotion>,
    mut camera_query: Query<&mut Transform, With<Camera>>,
) {
    let mut camera_transform = camera_query.single_mut();

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

fn create_actor(
    commands: &mut Commands,
    visual: Vec<String>,
    map_pos: MapPosHex,
    sprite_cfg_map: &Res<SpriteConfigMap>,
    layouts: &Res<Assets<TextureAtlasLayout>>,
) {
    let Some(layout) = layouts.get(sprite_cfg_map.layout.id()) else {
        panic!("Could not find layout in assets for sprite map")
    };

    let mut child_commands = commands.spawn((
        map_pos,
        Transform::from_translation(map_pos.into_vec3()),
        Visibility::Visible,
        InheritedVisibility::default(),
        GlobalTransform::default(),
    ));

    for (idx, v) in visual.iter().enumerate() {
        match sprite_cfg_map.map.get(v) {
            Some(SpriteConfig::Single {
                image_id,
                offset: (dx, dy),
            }) => {
                let index = layout.get_texture_index(*image_id).unwrap_or(0);

                child_commands.with_children(|parent| {
                    parent.spawn((
                        SpriteBundle {
                            texture: sprite_cfg_map.texture.clone(),
                            transform: Transform::from_xyz(*dx, *dy, (100 + idx) as f32),
                            ..Default::default()
                        },
                        TextureAtlas {
                            layout: sprite_cfg_map.layout.clone(),
                            index,
                        },
                    ));
                });
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

                child_commands.with_children(|parent| {
                    parent.spawn((
                        SpriteBundle {
                            texture: sprite_cfg_map.texture.clone(),
                            transform: Transform::from_xyz(*dx, *dy, (100 + idx) as f32),
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
                });
            }

            _ => {}
        }
    }
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
