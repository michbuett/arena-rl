mod asset;
mod combat_screen;
mod input;
mod start_screen;
mod teams_screen;
mod text;
mod types;

use sdl2::render::Texture;
pub use asset::*;
pub use input::*;
pub use text::*;
pub use types::*;

use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::WindowCanvas;
use std::time::Instant;

use crate::core::{DisplayStr, Game, UserInput, Sprite};

pub fn render(
    cvs: &mut WindowCanvas,
    ui: &UI,
    game: &Game,
    assets: &mut AssetRepo,
) -> Result<ClickAreas, String> {
    // let now = Instant::now();
    let (mut scene, click_areas) = match game {
        Game::Start(..) => start_screen::render(ui.viewport),

        Game::TeamSelection(_, _, game_objects) => {
            let (_, _, w, h) = ui.viewport;
            teams_screen::render((w, h), game_objects)
        }

        Game::Combat(combat_data) => {
            let scroll_offset = ui.scrolling.as_ref().map(|s| s.offset).unwrap_or((0, 0));
            combat_screen::render(ui.viewport, scroll_offset, combat_data)
        }
    };

    scene.texts.push(
        ScreenText::new(
            DisplayStr::new(format!("FPS: {}", ui.fps)),
            ScreenPos(10, ui.viewport.3 as i32 - 60),
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

    for (tex_name, ScreenSprite(pos, align, sprite)) in scene.images {
        if let Some(ref mut t) = assets.textures.get_mut(&tex_name) {
            draw_sprite(pos, align, sprite, t, cvs)?;
        }
    }

    for ScreenSprite(pos, align, sprite) in scene.sprites {
        let mut texture = assets.texture.as_mut();

        if let Some(ref mut t) = texture {
            draw_sprite(pos, align, sprite, t, cvs)?;
        }
    }

    for txt in scene.texts {
        let font = assets.fonts[txt.font as usize].as_mut().unwrap();
        font.draw(txt, cvs)?;
    }

    cvs.present();

    Ok(())
}

pub fn init_ui(viewport: (i32, i32, u32, u32), pixel_ratio: u8) -> UI {
    UI {
        viewport,
        pixel_ratio,
        fps: 0,
        frames: 0,
        last_check: Instant::now(),
        scrolling: None,
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
    let (_, _, w, h) = ui.viewport;

    UI {
        scrolling: match (scrolling, g, i) {
            (Some(sd), Game::Combat(..), None) => Some(sd),

            (None, Game::Combat(combat_data), _) => Some(ScrollData {
                is_scrolling: false,
                has_scrolled: false,
                offset: combat_screen::init_scroll_offset(combat_data, (w, h)),
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

fn draw_sprite(
    pos: ScreenPos,
    align: Align,
    sprite: Sprite,
    t: &mut Texture,
    cvs: &mut WindowCanvas,
) -> Result<(), String> {
    let (x, y) = sprite.source;
    let (dx, dy) = sprite.offset;
    let (w, h) = sprite.dim;
    let tw = (w as f32 * sprite.scale).round() as u32;
    let th = (h as f32 * sprite.scale).round() as u32;
    let pos = pos.align(align, tw, th);
    let from = Rect::new(x, y, w, h);
    let to = Rect::new(pos.0 + dx, pos.1 + dy, tw, th);

    t.set_alpha_mod(sprite.alpha);

    cvs.copy(t, from, to)
}
