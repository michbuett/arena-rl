use crate::core::{DisplayStr, Sprite, UserInput};
use crate::ui::{Align, ClickArea, ClickAreas, Scene, ScreenPos, ScreenSprite, ScreenText};

pub fn render(viewport: (i32, i32, u32, u32)) -> (Scene, ClickAreas) {
    let (width, height) = (400, 106);
    let (_, _, viewport_width, viewport_height) = viewport;
    let xpos = ((viewport_width - width) / 2) as i32;
    let ypos = ((viewport_height - height) / 2) as i32;
    let mut scene = Scene::empty();

    scene.images.push((
        "logo".to_string(),
        ScreenSprite(
            ScreenPos(xpos, ypos),
            Align::TopLeft,
            Sprite {
                source: ((0, 0), None, None),
                dim: (width, height),
                offset: (0, 0),
                alpha: 255,
                scale: 1.0,
                rotate: None,
            },
        ),
    ));

    scene.texts.push(ScreenText::new(
        DisplayStr::new("Click somewhere to continue ..."),
        ScreenPos(xpos, viewport_height as i32 - 60),
    ));

    (
        scene,
        vec![ClickArea {
            clipping_area: viewport,
            action: Box::new(|_| UserInput::NewGame),
        }],
    )
}
