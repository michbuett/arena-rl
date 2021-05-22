mod details;
mod map;

use specs::prelude::*;

// use sdl2::pixels::Color;
use sdl2::rect::Rect;

use crate::core::{CombatData, Map};
use super::types::{TextureMap, ClickAreas, Scene, TILE_WIDTH, TILE_HEIGHT};

pub fn render(
    viewport: &Rect,
    scroll_offset: (i32, i32),
    game: &CombatData,
    texture_map: &TextureMap,
) -> (Scene, ClickAreas) {
    let mut click_areas: ClickAreas = vec!();
    let (mut scene, mut map_clicks) = map::render(viewport, scroll_offset, game, texture_map);

    details::render(&mut scene, &mut click_areas, viewport, game);

    // render_screen_texts(cvs, assets, viewport, game)?;

    click_areas.append(&mut map_clicks);
    
    (scene, click_areas)
}

// fn render_screen_texts(
//     cvs: &mut WindowCanvas,
//     assets: &AssetRepo,
//     viewport: &Rect,
//     game: &CombatData,
// ) -> Result<(), String> {
//     assets
//         .font("normal")?
//         .text(DisplayStr::new(format!("Turn: {}", game.turn)))
//         .padding(10)
//         .background(Color::RGB(252, 251, 250))
//         .prepare()
//         .draw(cvs, (10, 10))?;

//     let mut y = 75;
//     for s in &game.log {
//         let msg = assets
//             .font("normal")?
//             .text(s.clone())
//             .max_width(500)
//             .padding(10)
//             .background(Color::RGBA(252, 251, 250, 100))
//             .prepare();

//         let txt_height = msg.dimension().1 as i32;
        
//         if y + txt_height > viewport.height() as i32 {
//             break;
//         }

//         msg.draw(cvs, (10, y))?;
//         y += txt_height;
//     }

//     // if let CombatState::Win(Team(_, num, ..)) = game.state {
//     //     let text = assets
//     //         .font("very big")?
//     //         .text(format!("Team #{} wins!", num))
//     //         .padding(50)
//     //         .background(Color::RGBA(252, 251, 250, 150))
//     //         .prepare();

//     //     let (w, h) = text.dimension();
//     //     let x = (viewport.width() - w) / 2;
//     //     let y = (viewport.height() - h) / 2;

//     //     text.draw(cvs, (x as i32, y as i32))?;
//     // }

//     Ok(())
// }

pub fn init_scroll_offset(game: &CombatData, viewport: Rect) -> (i32, i32) {
    let map: Read<Map> = game.world.system_data();
    let num_columns = map.num_columns();
    let num_rows = map.num_rows();
    let map_width = TILE_WIDTH * (num_columns + num_rows) / 2;
    let map_height = TILE_HEIGHT * (num_columns + num_rows) / 2;

    (
        (viewport.width() as i32 - map_width as i32) / 2  as i32 + map_width as i32 / 2 - TILE_WIDTH as i32,
        (viewport.height() as i32 - map_height as i32) / 2,
    )
}
