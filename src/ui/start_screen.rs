use crate::ui::{ClickArea, ClickAreas,  Scene, ScreenText, ScreenSprite, ScreenPos};
use crate::core::{Sprite, UserInput, DisplayStr};

pub fn render(viewport: (i32, i32, u32, u32)) -> (Scene, ClickAreas) {
    let (width, height) = (400, 106);
    let (_, _, viewport_width, viewport_height) = viewport;
    let xpos = ((viewport_width - width) / 2) as i32;
    let ypos = ((viewport_height - height) / 2) as i32;
    let mut scene = Scene::empty();
    
    scene.sprites.push(
        // ScreenSprite {
        //     source: ("logo".to_string(), 0, 0, width, height),
        //     offset: (0, 0),
        //     pos: ScreenPos(xpos, ypos),
        //     alpha: 255,
        //     target_size: (width, height),
        // }
        ScreenSprite(ScreenPos(xpos, ypos), Sprite {
            source: (0, 0),
            dim: (width, height),
            offset: (0, 0),
            alpha: 255,
        })
    );

    scene.texts.push(
        ScreenText::new(
            DisplayStr::new("Click somewhere to continue ..."),
            ScreenPos(xpos, viewport_height as i32 - 60)
        )
    );

    (scene, vec!(ClickArea {
        clipping_area: viewport,
        action: Box::new(|_| UserInput::NewGame ),
    }))
}
