mod details;
mod map;

use specs::prelude::*;

use super::types::*;

use crate::core::{CombatData, CombatPhase, DisplayStr, Map, TurnState};

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
                "Turn: {}, Score: {}, {}",
                render_turn_data(&game.turn),
                game.score,
                render_reinforcements_hint(&game.turn),
            )),
            ScreenPos(10, 10),
        )
        .color((20, 150, 20, 255))
        .padding(10)
        .background((252, 251, 250, 255)),
    );
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

fn render_turn_data(td: &TurnState) -> String {
    let phase_str = match td.phase {
        CombatPhase::Planning => "Plan your turn",
        CombatPhase::Action => "Time for action",
    };
    format!("{} ({})", td.turn_number, phase_str)
}

fn render_reinforcements_hint(td: &TurnState) -> String {
    if let Some(num) = td.next_reinforcements {
        format!("Reinforcements incomming (ETA: {})", num)
    } else {
        "Final wave!".to_string()
    }
}
