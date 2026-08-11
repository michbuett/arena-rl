use bevy::{
    ecs::{lifecycle::HookContext, world::DeferredWorld},
    prelude::*,
};

pub const BLACK: Color = Color::srgb(0.15, 0.14, 0.13);
pub const WINDOW_BACKGROUND: Color = Color::srgb(0.90, 0.89, 0.88);
pub const WINDOW_BACKGROUND_HL: Color = Color::srgb(1.0, 0.99, 0.98);
pub const WINDOW_BACKGROUND_TANSPARENT: Color = Color::srgba(0.99, 0.98, 0.97, 0.5);

#[derive(Debug, Clone, Copy, Component)]
pub enum TextStyle {
    HUDElement,
    UiNormal,
    InGameScream,
}

impl TextStyle {
    pub fn on_insert(mut world: DeferredWorld, context: HookContext) {
        let text_style = world.get::<TextStyle>(context.entity).unwrap();
        let (text_font, text_color) = match text_style {
            Self::HUDElement => {
                let handle = world.load_asset("fonts/Bangers-Regular.ttf");
                // let handle = world.load_asset("fonts/RubikDistressed-Regular.ttf");
                let text_font = TextFont {
                    font: FontSource::Handle(handle),
                    font_size: FontSize::Px(14.),
                    ..default()
                };
                let text_color = TextColor(Color::LinearRgba(LinearRgba::rgb(0.99, 0.98, 0.97)));
                (text_font, text_color)
            }

            Self::InGameScream => {
                let handle = world.load_asset("fonts/Bangers-Regular.ttf");
                let text_font = TextFont {
                    font: FontSource::Handle(handle),
                    font_size: FontSize::Px(48.),
                    ..default()
                };
                let text_color = TextColor(Color::LinearRgba(LinearRgba::rgb(0.8, 0.2, 0.1)));
                (text_font, text_color)
            }

            Self::UiNormal => {
                // let handle = world.load_asset("fonts/ComicRelief-Regular.ttf");
                let handle = world.load_asset("fonts/Oldenburg-Regular.ttf");
                let text_font = TextFont {
                    font: FontSource::Handle(handle),
                    font_size: FontSize::Px(12.),
                    ..default()
                };
                let text_color = TextColor(Color::srgb(0.03, 0.02, 0.01));
                (text_font, text_color)
            }
        };

        world
            .commands()
            .entity(context.entity)
            .insert((text_font, text_color));
    }
}

pub fn text(txt: impl Into<String>, style: TextStyle) -> impl Bundle {
    (Text::new(txt.into()), style)
}

pub fn text2d(txt: impl Into<String>, style: TextStyle) -> impl Bundle {
    (Text2d::new(txt.into()), style)
}

pub fn button(txt: impl Into<String>) -> impl Bundle {
    let txt: String = txt.into();

    (
        Name::new(format!("Button [{txt}]")),
        BorderColor::all(Color::BLACK),
        Node {
            display: Display::Block,
            width: Val::Percent(100.0),
            border: UiRect::all(Val::Px(3.0)),
            margin: UiRect::vertical(Val::Px(5.0)),
            padding: UiRect::all(Val::Px(10.0)),
            ..Default::default()
        },
        children![text(txt, TextStyle::UiNormal)],
    )
}
