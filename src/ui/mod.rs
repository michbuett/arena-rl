mod asset;
mod combat_screen;
mod input;
mod start_screen;
mod teams_screen;
mod text;
mod types;

// use crate::components::Sprites;
pub use asset::AssetRepo;
pub use input::*;
pub use text::*;
pub use types::*;

use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::WindowCanvas;
use std::time::Instant;
// use specs::prelude::*;

use crate::core::{Game, UserInput, DisplayStr};
// use specs::prelude::*;

pub fn render<'a>(
    cvs: &mut WindowCanvas,
    ui: &UI,
    game: &Game,
    // assets: &'a mut AssetRepo<'a>,
    assets: &mut AssetRepo,
) -> Result<ClickAreas, String> {
    // let now = Instant::now();
    let (mut scene, click_areas) = match game {
        Game::Start =>
            start_screen::render(&ui.viewport),

        Game::TeamSelection(game_objects) =>
            teams_screen::render(&ui.viewport, game_objects),
        
        Game::Combat(combat_data) =>
            combat_screen::render(&ui.viewport, ui.scroll_offset, combat_data),
    };

    scene.texts.push(
    // scene.texts[FontFace::Normal as usize].push(
        ScreenText::new(DisplayStr::new(format!("FPS: {}", ui.fps)), ScreenPos(10, ui.viewport.height() as i32 - 60))
            .color((20, 150, 20, 255))
            .padding(10)
            .background((252, 251, 250, 255))
    );
    // let time_create_scene = Instant::now() - now;
    
    // let now = Instant::now();
    draw_scene(cvs, assets, scene)?;
    // let time_draw_scene = Instant::now() - now;

    // cvs.present();
    // println!("create scene: {}ms, draw scene {}ms", time_create_scene.as_millis(), time_draw_scene.as_millis());

    Ok(click_areas)
}
    
fn draw_scene(
// fn draw_scene<'a>(
    cvs: &mut WindowCanvas,
    assets: &mut AssetRepo,
    // assets: &'a mut AssetRepo<'a>,
    mut scene: Scene,
) -> Result<(), String> {
    let (r, g, b) = scene.background;
    cvs.set_draw_color(Color::RGB(r, g, b));
    cvs.clear();

    let mut last_texture = String::new();
    let mut texture = None;

    for ScreenSprite { source, pos, offset, alpha, target_size } in scene.sprites {
        if last_texture != source.0 {
            last_texture = source.0.clone();
            texture = assets.textures.get_mut(&source.0);
        }

        if let Some(ref mut t) = texture {
            let (_, x, y, w, h) = source;
            let from = Rect::new(x, y, w, h);
            let to = Rect::new(pos.0 + offset.0, pos.1 + offset.1, target_size.0, target_size.1);
            t.set_alpha_mod(alpha);
            cvs.copy(t, from, to)?;
        }
    }

    // for (font_face, texts) in scene.texts.iter_mut().enumerate() {
    //     let ff_str = match font_face {
    //         0 => "normal",
    //         1 => "big",
    //         2 => "very big",
    //         _ => "none",
    //     };

    //     let font = assets.fonts.get_mut(ff_str).unwrap();
    //     for txt in texts.drain(..) {
    //         font.draw(txt, cvs)?;
    //     }
    // }

    for txt in scene.texts {
        let font = assets.fonts[txt.font as usize].as_mut().unwrap();
        // for txt in texts.drain(..) {
            font.draw(txt, cvs)?;
        // }
    }

    cvs.present();

    Ok(())
}

// fn test_draw(cvs: &mut WindowCanvas, assets: &mut AssetRepo, font_face: usize, txt: ScreenText) -> Result<(), String> {
//     let ff_str = match font_face {
//         0 => "normal",
//         1 => "big",
//         2 => "very big",
//         _ => "none",
//     };

//     assets.fonts.get_mut(ff_str).unwrap().draw(txt, cvs)
// }

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
