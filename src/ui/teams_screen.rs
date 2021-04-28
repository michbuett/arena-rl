use crate::ScreenPos;
use crate::ScreenText;
use sdl2::rect::Rect;

use crate::core::{GameObject, UserInput, DisplayStr};
use crate::ui::{ClickArea, ClickAreas, Scene, FontFace};

pub fn render(
    viewport: &Rect,
    game_objects: &Vec<GameObject>,
) -> (Scene, ClickAreas) {
    let mut scene = Scene::empty();
    let mut click_areas = vec!();

    scene.texts.push(
    // scene.texts[FontFace::Big as usize].push(
        ScreenText::new(
            DisplayStr::new("Your Team"),
            ScreenPos(((viewport.width() - 185) / 2) as i32, 50),
        ).font(FontFace::Big)
    );

    render_actors(&mut scene, &mut click_areas, viewport, game_objects);
    render_start_btn(&mut scene, &mut click_areas, viewport);

    (scene, click_areas)
}

pub fn render_start_btn(
    scene: &mut Scene,
    click_areas: &mut ClickAreas,
    viewport: &Rect,
) {
    let (w, h) = (230, 76);
    let (x, y) = ((viewport.width() - w - 20) as i32, (viewport.height() - h - 20) as i32);
    
    scene.texts.push(
    // scene.texts[FontFace::Normal as usize].push(
        ScreenText::new(
            DisplayStr::new("Enter the arena ..."),
            ScreenPos(x, y),
        )
        .padding(20)
        .border(3, (23, 22, 21, 255))
        .background((242, 241, 240, 255))
    );

    click_areas.push(ClickArea {
        clipping_area: Rect::new(x, y, w as u32, h as u32),
        action: Box::new(|_| UserInput::SelectTeam(vec!(
            // TODO: pass configured player characters
        ))),
    });
}

pub fn render_actors(
    _scene: &mut Scene,
    _click_areas: &mut ClickAreas,
    _viewport: &Rect,
    _game_objects: &Vec<GameObject>,
) {
    // TODO display selected team and allow changes
}
