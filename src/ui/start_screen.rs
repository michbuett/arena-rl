use sdl2::rect::Rect;
// use sdl2::render::{TextureQuery, WindowCanvas};

use crate::ui::{ClickArea, ClickAreas,  Scene, FontFace, ScreenText, ScreenSprite, ScreenPos};
use crate::core::{UserInput, DisplayStr};

pub fn render(viewport: &Rect) -> (Scene, ClickAreas) {
    let (width, height) = (400, 106);
    let xpos = ((viewport.width() - width) / 2) as i32;
    let ypos = ((viewport.height() - height) / 2) as i32;
    let mut scene = Scene::empty();
    
    scene.sprites.push(
        ScreenSprite {
            source: ("logo".to_string(), 0, 0, width, height),
            offset: (0, 0),
            pos: ScreenPos(xpos, ypos),
            alpha: 255,
            target_size: (width, height),
        }
    );

    scene.texts.push(
    // scene.texts[FontFace::Normal as usize].push(
        ScreenText::new(
            DisplayStr::new("Click somewhere to continue ..."),
            ScreenPos(
                xpos,
                viewport.height() as i32 - 60
            )
        )
    );

    (scene, vec!(ClickArea {
        clipping_area: viewport.clone(),
        action: Box::new(|_| UserInput::NewGame ),
    }))
}
