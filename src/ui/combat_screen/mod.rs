mod details;
mod map;

use specs::prelude::*;

use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::WindowCanvas;

use crate::core::*;
use super::asset::AssetRepo;
use super::types::*;

pub fn render(
    cvs: &mut WindowCanvas,
    viewport: &Rect,
    scroll_offset: (i32, i32),
    game: &CombatData,
    assets: &AssetRepo,
) -> Result<ClickAreas, String> {
    let focus_pos = get_focus_pos(&game.state)
        .map(|wp| map::map_pos_to_screen_pos(wp, scroll_offset));

    let mut map_clicks = map::render(cvs, viewport, scroll_offset, focus_pos, game, assets)?;
    let mut click_areas = if let Some((x, y)) = focus_pos {
        details::render(cvs, viewport, ScreenPos(x, y), game, assets)?
    } else {
        vec!()
    };

    render_screen_texts(cvs, assets, viewport, game)?;

    click_areas.append(&mut map_clicks);
    
    Ok(click_areas)
}

fn render_screen_texts(
    cvs: &mut WindowCanvas,
    assets: &AssetRepo,
    viewport: &Rect,
    game: &CombatData,
) -> Result<(), String> {
    assets
        .font("normal")?
        .text(format!("Turn: {}", game.turn))
        .padding(10)
        .background(Color::RGB(252, 251, 250))
        .prepare()
        .draw(cvs, (10, 10))?;

    if let CombatState::Win(Team(_, num)) = game.state {
        let text = assets
            .font("very big")?
            .text(format!("Team #{} wins!", num))
            .padding(50)
            .background(Color::RGBA(252, 251, 250, 150))
            .prepare();

        let (w, h) = text.dimension();
        let x = (viewport.width() - w) / 2;
        let y = (viewport.height() - h) / 2;

        text.draw(cvs, (x as i32, y as i32))?;
    }

    Ok(())
}

fn get_focus_pos<'a>(game_state: &CombatState) -> Option<WorldPos> {
    match game_state {
        CombatState::WaitForUserAction(_, Some(InputContext::SelectedArea(p, _, _)), _) => Some(*p),

        CombatState::WaitForUserAction((_, a), Some(InputContext::Opportunity(..)), _) => Some(a.pos),

        _ => None,
    }
}

pub fn init_scroll_offset(game: &CombatData, viewport: Rect) -> (i32, i32) {
    let map: Read<Map> = game.world.system_data();
    let num_columns = map.num_columns();
    let num_rows = map.num_rows();
    let map_width = map::TILE_WIDTH * (num_columns + num_rows) / 2;
    let map_height = map::TILE_HEIGHT * (num_columns + num_rows) / 2;

    (
        (viewport.width() as i32 - map_width as i32) / 2,
        (viewport.height() as i32 - map_height as i32) / 2,
    )
}
