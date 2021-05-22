mod asset;
mod combat_screen;
mod input;
mod start_screen;
mod teams_screen;
mod text;
mod types;

use std::collections::HashMap;
pub use asset::*;
pub use input::*;
pub use text::*;
pub use types::*;

use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::WindowCanvas;
use std::time::Instant;

use crate::core::{DisplayStr, Game, UserInput};

pub fn render(
    cvs: &mut WindowCanvas,
    ui: &UI,
    game: &Game,
    assets: &mut AssetRepo,
) -> Result<ClickAreas, String> {
    // let now = Instant::now();
    let (mut scene, click_areas) = match game {
        Game::Start => start_screen::render(&ui.viewport),

        Game::TeamSelection(game_objects) => teams_screen::render(&ui.viewport, game_objects),

        Game::Combat(combat_data) => {
            let scroll_offset = ui.scrolling.as_ref().map(|s| s.offset).unwrap_or((0, 0));
            combat_screen::render(&ui.viewport, scroll_offset, combat_data, &ui.textures.1)
        }
    };

    scene.texts.push(
        ScreenText::new(
            DisplayStr::new(format!("FPS: {}", ui.fps)),
            ScreenPos(10, ui.viewport.height() as i32 - 60),
        )
        .color((20, 150, 20, 255))
        .padding(10)
        .background((252, 251, 250, 255)),
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
    cvs: &mut WindowCanvas,
    assets: &mut AssetRepo,
    scene: Scene,
) -> Result<(), String> {
    let (r, g, b) = scene.background;

    cvs.set_draw_color(Color::RGB(r, g, b));
    cvs.clear();

    for ScreenSprite {
        source,
        pos,
        offset,
        alpha,
        target_size,
    } in scene.sprites
    {
        let mut texture = if source.0.is_empty() {
            assets.texture.as_mut()
        } else {
            assets.textures.get_mut(&source.0)
        };

        if let Some(ref mut t) = texture {
            let (_, x, y, w, h) = source;
            let from = Rect::new(x, y, w, h);
            let to = Rect::new(
                pos.0 + offset.0,
                pos.1 + offset.1,
                target_size.0,
                target_size.1,
            );

            t.set_alpha_mod(alpha);

            cvs.copy(t, from, to)?;

            // println!("render_sprite from {:?} to {:?}", from, to);
            // if source.0 == "floor" {
            //     cvs.set_draw_color(Color::RGB(0, 0, 0));
            //     cvs.draw_rect(to)?;
            // }
        }
    }

    for txt in scene.texts {
        let font = assets.fonts[txt.font as usize].as_mut().unwrap();
        font.draw(txt, cvs)?;
    }

    cvs.present();

    Ok(())
}

pub fn init_ui(viewport: Rect, pixel_ratio: u8, texture_map: HashMap<String, SpriteConfig>) -> UI {
    UI {
        viewport,
        pixel_ratio,
        fps: 0,
        frames: 0,
        last_check: Instant::now(),
        scrolling: None,
        textures: ("combat", texture_map),
    }
}

pub fn step_ui(mut ui: UI, g: &Game, i: &Option<UserInput>) -> UI {
    ui = update_fps(ui);
    ui = update_scrolling(ui, g, i);
    ui
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

fn update_scrolling(ui: UI, g: &Game, i: &Option<UserInput>) -> UI {
    let scrolling = ui.scrolling;

    UI {
        scrolling: match (scrolling, g, i) {
            (Some(sd), Game::Combat(..), None) => Some(sd),

            (None, Game::Combat(combat_data), _) => Some(ScrollData {
                is_scrolling: false,
                has_scrolled: false,
                offset: combat_screen::init_scroll_offset(combat_data, ui.viewport),
            }),

            (Some(sd), Game::Combat(..), Some(i)) => Some(get_scrolling(sd, i)),

            _ => None,
        },
        ..ui
    }

    // if ui.is_scrolling {
    //     return match i {
    //         UserInput::ScrollTo(dx, dy) => UI {
    //             scroll_offset: (ui.scroll_offset.0 + dx, ui.scroll_offset.1 + dy),
    //             has_scrolled: true,
    //             ..ui
    //         },

    //         _ => UI {
    //             is_scrolling: false,
    //             ..ui
    //         },
    //     };
    // } else {
    //     if let UserInput::StartScrolling() = i {
    //         return UI {
    //             is_scrolling: true,
    //             has_scrolled: false,
    //             ..ui
    //         };
    //     }
    // }
}
fn get_scrolling(sd: ScrollData, i: &UserInput) -> ScrollData {
    if sd.is_scrolling {
        return match i {
            UserInput::ScrollTo(dx, dy) => ScrollData {
                offset: (sd.offset.0 + dx, sd.offset.1 + dy),
                has_scrolled: true,
                ..sd
            },

            _ => ScrollData {
                is_scrolling: false,
                ..sd
            },
        };
    } else {
        if let UserInput::StartScrolling() = i {
            return ScrollData {
                is_scrolling: true,
                has_scrolled: false,
                ..sd
            };
        }
    }

    sd
}

// fn get_scrolling(ui: &UI, game: &Game) -> Option<ScrollData> {
//     if let Some
// }
