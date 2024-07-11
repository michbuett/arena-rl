use bevy::prelude::*;

use crate::{
    despawn_screen,
    style::{TEXT_COLOR, WINDOW_BACKGROUND},
    GameState,
};

// Tag component used to tag entities added on the splash screen
#[derive(Component)]
struct OnStartState;

pub fn start_plugin(app: &mut App) {
    app.add_systems(OnEnter(GameState::MainMenu), start_setup)
        .add_systems(
            Update,
            continue_on_keypress.run_if(in_state(GameState::MainMenu)),
        )
        .add_systems(OnExit(GameState::MainMenu), despawn_screen::<OnStartState>);
}

fn start_setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let icon = asset_server.load("images/logo.png");

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    align_items: AlignItems::Center,
                    justify_items: JustifyItems::Center,
                    justify_content: JustifyContent::Center,
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    display: Display::Grid,
                    ..default()
                },
                background_color: WINDOW_BACKGROUND.into(),
                ..default()
            },
            OnStartState,
        ))
        .with_children(|parent| {
            parent.spawn(ImageBundle {
                image: UiImage::new(icon),
                style: Style {
                    width: Val::Px(400.0),
                    ..default()
                },
                ..default()
            });
            parent.spawn(TextBundle {
                text: Text::from_section(
                    "Press any key to continue",
                    TextStyle {
                        font_size: 20.0,
                        color: TEXT_COLOR,
                        ..default()
                    },
                ),
                ..default()
            });
        });
}

fn continue_on_keypress(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    let key_pressed = keyboard_input.get_pressed().count() > 0;
    let mouse_pressed = mouse_input.get_pressed().count() > 0;

    if key_pressed || mouse_pressed {
        game_state.set(GameState::Combat);
    }
}
