mod asset;
mod combat_screen;
mod input;
mod start_screen;
mod teams_screen;
mod text;
mod types;

pub use asset::AssetRepo;
pub use input::*;
pub use text::*;
// pub use screen::{Screen};
pub use types::*;

use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::WindowCanvas;
use std::time::Instant;
// use specs::prelude::*;

use crate::core::{Game, UserInput, DisplayStr};
// use specs::prelude::*;

pub fn render(
    cvs: &mut WindowCanvas,
    ui: &UI,
    game: &Game,
    assets: &AssetRepo,
) -> Result<ClickAreas, String> {
    cvs.set_draw_color(Color::RGB(252, 251, 250));
    cvs.clear();

    let click_areas = match game {
        Game::Start =>
            start_screen::render(cvs, &ui.viewport, assets)?,

        Game::TeamSelection(game_objects) =>
            teams_screen::render(cvs, &ui.viewport, game_objects, assets)?,
        
        Game::Combat(combat_data) =>
            combat_screen::render(cvs, &ui.viewport, ui.scroll_offset, combat_data, assets)?,
    };

    assets
        .font("normal")?
        .text(DisplayStr::new(format!("FPS: {}", ui.fps)))
        .padding(10)
        .background(Color::RGB(252, 251, 250))
        .prepare()
        .draw(cvs, (10, ui.viewport.height() as i32 - 60))?;

    cvs.present();

    Ok(click_areas)
}

pub fn init_ui(game: &Game, viewport: Rect, pixel_ratio: u8) -> UI {
    let scroll_offset = match game {
        Game::Combat(combat_data) =>
            combat_screen::init_scroll_offset(combat_data, viewport),
        _ => (0, 0),
    };
    
    UI {
        viewport,
        pixel_ratio,
        fps: 0,
        frames: 0,
        last_check: Instant::now(),
        is_scrolling: false,
        has_scrolled: false,
        scroll_offset,
    }
}

pub fn step_ui(ui: UI, i: &Option<UserInput>) -> UI {
    let ui = update_fps(ui);

    if let Some(i) = i {
        update_scrolling(ui, i)
    } else {
        ui
    }
}

fn update_fps(ui: UI) -> UI {
    if ui.frames == 50 {
        let time_for_50_frames = ui.last_check.elapsed().as_nanos();

        UI {
            frames: 0,
            fps: 50_000_000_000 / time_for_50_frames,
            last_check: std::time::Instant::now(),
            ..ui
        }
    } else {
        UI {
            frames: ui.frames + 1,
            ..ui
        }
    }
}

fn update_scrolling(ui: UI, i: &UserInput) -> UI {
    if ui.is_scrolling {
        return match i {
            UserInput::ScrollTo(dx, dy) => UI {
                scroll_offset: (ui.scroll_offset.0 + dx, ui.scroll_offset.1 + dy),
                has_scrolled: true,
                ..ui
            },

            _ => UI {
                is_scrolling: false,
                ..ui
            },
        };
    } else {
        if let UserInput::StartScrolling() = i {
            return UI {
                is_scrolling: true,
                has_scrolled: false,
                ..ui
            };
        }
    }

    ui
}
