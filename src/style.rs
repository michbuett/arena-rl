use bevy::prelude::*;

pub const WINDOW_BACKGROUND: Color = Color::srgb(0.99, 0.98, 0.97);
pub const BUTTON_BG_HIGHLIGHT: Color = Color::srgb(1.0, 0.99, 0.98);

#[derive(Debug, Clone, Copy)]
pub enum TextStyle {
    UiNormal,
    InGameScream,
}

impl TextStyle {
    pub fn as_text_color(&self) -> TextColor {
        let color = match self {
            Self::InGameScream => Color::LinearRgba(LinearRgba::rgb(0.8, 0.2, 0.1)),
            _ => Color::srgb(0.03, 0.02, 0.01),
        };
        TextColor(color)
    }

    pub fn as_text_font(&self) -> TextFont {
        let font_size = match self {
            Self::InGameScream => 48.,
            _ => 12.,
        };

        TextFont {
            font_size,
            ..default()
        }
    }
}

pub fn text(txt: impl Into<String>, style: TextStyle) -> (Text, TextColor, TextFont) {
    (
        Text::new(txt.into()),
        style.as_text_color(),
        style.as_text_font(),
    )
}
