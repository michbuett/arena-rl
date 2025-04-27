mod animations;
mod assets;
mod combat;
mod core;
mod start;
mod style;

use std::time::Duration;

use bevy::{prelude::*, render::camera::ScalingMode, window::PrimaryWindow};

#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
enum GameState {
    #[default]
    Start,
    MainMenu,
    Combat,
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            assets::assets_plugin,
            animations::animation_plugin,
            start::start_plugin,
            combat::combat_plugin,
        ))
        .init_state::<GameState>()
        .add_systems(Startup, setup)
        .add_systems(Update, update_end_of_live_removal)
        .run();
}

fn setup(mut commands: Commands, windows: Query<&Window, With<PrimaryWindow>>) {
    let win = windows.single();
    let scaling_mode = ScalingMode::WindowSize(1.0 / win.resolution.scale_factor());
    let mut camera_bundle = Camera2dBundle::default();

    camera_bundle.projection.scaling_mode = scaling_mode;

    commands.spawn(camera_bundle);
}

/// Generic system that takes a component as a parameter, and will despawn all entities with that component
fn despawn_screen<T: Component>(to_despawn: Query<Entity, With<T>>, mut commands: Commands) {
    for entity in &to_despawn {
        commands.entity(entity).despawn_recursive();
    }
}

#[derive(Component, Debug, Clone)]
pub struct EndOfLive(pub Timer);

impl EndOfLive {
    pub fn after(d: Duration) -> Self {
        EndOfLive(Timer::new(d, TimerMode::Once))
    }
}

fn update_end_of_live_removal(
    time: Res<Time>,
    mut commands: Commands,
    mut eol_q: Query<(Entity, &mut EndOfLive)>,
) {
    for (e, mut eol) in eol_q.iter_mut() {
        eol.0.tick(time.delta());

        if eol.0.finished() {
            commands.entity(e).despawn_recursive();
        }
    }
}
