use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::WindowCanvas;

use crate::core::{GameObject, UserInput, WorldPos, Team, Armor, ActorBuilder, Attributes};
use crate::ui::{AssetRepo, ClickArea, ClickAreas};

pub fn render(
    cvs: &mut WindowCanvas,
    viewport: &Rect,
    game_objects: &Vec<GameObject>,
    assets: &AssetRepo,
) -> Result<ClickAreas, String> {
    let text = assets
        .font("big")?
        .text("Your team".to_string())
        .prepare();

    text.draw(
        cvs,
        (((viewport.width() - text.dimension().0) / 2) as i32, 50),
    )?;

    let click_areas = render_actors(cvs, viewport, game_objects, assets)?.into_iter()
        .chain(vec![render_start_btn(cvs, viewport, assets)?].into_iter())
        .collect();

    Ok(click_areas)
}

pub fn render_start_btn(
    cvs: &mut WindowCanvas,
    viewport: &Rect,
    assets: &AssetRepo,
) -> Result<ClickArea, String> {
    let start_btn = assets
        .font("normal")?
        .text("Enter the arena ...".to_string())
        .padding(20)
        .border(3, Color::RGB(23, 22, 21))
        .background(Color::RGB(252, 251, 250))
        .prepare();

    let start_btn_area = Rect::new(
        viewport.width() as i32 - start_btn.dimension().0 as i32 - 20,
        viewport.height() as i32 - start_btn.dimension().1 as i32 - 20,
        start_btn.dimension().0,
        start_btn.dimension().1,
    );

    start_btn.draw(cvs, (start_btn_area.x, start_btn_area.y))?;

    Ok(ClickArea {
        clipping_area: start_btn_area,
        action: Box::new(|_| UserInput::SelectTeam(vec!(
            create_player(WorldPos(6.0, 6.0)),
            create_player(WorldPos(6.0, 5.0)),
            create_player(WorldPos(5.0, 6.0)),
        ))),
    })
}

pub fn render_actors(
    _cvs: &mut WindowCanvas,
    _viewport: &Rect,
    _game_objects: &Vec<GameObject>,
    _assets: &AssetRepo,
) -> Result<ClickAreas, String> {
    Ok(vec![])
}

fn create_player(pos: WorldPos) -> GameObject {
    let actor = ActorBuilder::new(pos, Attributes::new(4, 4, 4), Team("Player", 1))
        .armor(Armor { look: vec!("player"), protection: 2 })
        .build();

    GameObject::Actor(actor)
}
