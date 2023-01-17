mod details;
mod map;

use specs::prelude::*;

use super::types::*;

use crate::core::{CombatData, CombatPhase, DisplayStr, Map, TurnData};

pub fn render(
    (x, y, w, h): (i32, i32, u32, u32),
    scroll_offset: (i32, i32),
    game: &CombatData,
) -> (Scene, ClickAreas) {
    let mut click_areas: ClickAreas = vec![];
    let (mut scene, mut map_clicks) = map::render((x, y, w, h), scroll_offset, game);

    details::render(&mut scene, &mut click_areas, (w, h), game);

    render_screen_texts(&mut scene, game);

    click_areas.append(&mut map_clicks);

    (scene, click_areas)
}

fn render_screen_texts(
    scene: &mut Scene,
    // viewport: (i32, i32, u32, u32),
    game: &CombatData,
) {
    scene.texts.push(
        ScreenText::new(
            DisplayStr::new(format!(
                "Turn: {}, Score: {}",
                render_turn_data(&game.turn_data),
                game.score
            )),
            ScreenPos(10, 10),
        )
        .color((20, 150, 20, 255))
        .padding(10)
        .background((252, 251, 250, 255)),
    );

    // let mut y = 75;
    // for s in &game.log {
    //     let msg = assets
    //         .font("normal")?
    //         .text(s.clone())
    //         .max_width(500)
    //         .padding(10)
    //         .background(Color::RGBA(252, 251, 250, 100))
    //         .prepare();

    //     let txt_height = msg.dimension().1 as i32;

    //     if y + txt_height > viewport.height() as i32 {
    //         break;
    //     }

    //     msg.draw(cvs, (10, y))?;
    //     y += txt_height;
    // }

    // if let CombatState::Win(Team(_, num, ..)) = game.state {
    //     let text = assets
    //         .font("very big")?
    //         .text(format!("Team #{} wins!", num))
    //         .padding(50)
    //         .background(Color::RGBA(252, 251, 250, 150))
    //         .prepare();

    //     let (w, h) = text.dimension();
    //     let x = (viewport.width() - w) / 2;
    //     let y = (viewport.height() - h) / 2;

    //     text.draw(cvs, (x as i32, y as i32))?;
    // }
}

pub fn init_scroll_offset(
    game: &CombatData,
    (viewport_width, viewport_height): (u32, u32),
) -> (i32, i32) {
    let map: Read<Map> = game.world.system_data();
    let num_columns = map.num_columns();
    let num_rows = map.num_rows();
    let map_width = TILE_WIDTH * (num_columns + num_rows) / 2;
    let map_height = TILE_HEIGHT * (num_columns + num_rows) / 2;

    (
        (viewport_width as i32 - map_width as i32) / 2 as i32 + map_width as i32 / 2
            - TILE_WIDTH as i32,
        (viewport_height as i32 - map_height as i32) / 2,
    )
}

fn render_turn_data(td: &TurnData) -> String {
    let phase_str = match td.phase {
        CombatPhase::Plan => format!("Team {} selects actions ...", td.active_team().0),
        CombatPhase::React => format!("Team {} reacting ...", td.active_team().0),
        CombatPhase::Resolve => "Resolving results".to_string(),
    };
    format!("{} ({})", td.turn_number, phase_str)
}
