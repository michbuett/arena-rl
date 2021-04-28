use specs::prelude::*;

use sdl2::rect::Rect;

use crate::components::{Position, Sprites, Text};
use crate::core::{CombatData, CombatState, DisplayStr, InputContext, Map, Tile, TileType, UserInput, WorldPos};
use crate::ui::{ClickArea, Scene, ScreenPos, ScreenSprite, ScreenText};

pub const TILE_WIDTH: u32 = 96;
pub const TILE_HEIGHT: u32 = 96;

pub type SystemData<'a> = (
    ReadStorage<'a, Position>,
    ReadStorage<'a, Sprites>,
    ReadStorage<'a, Text>,
    Read<'a, Map>,
);

pub fn render(
    viewport: &Rect,
    scroll_offset: (i32, i32),
    focus_pos: Option<(i32, i32)>,
    game: &CombatData,
) -> (Scene, Vec<ClickArea>) {
    let (pos, sprites, texts, map): SystemData = game.world.system_data();

    let mut scene = Scene {
        sprites: Vec::new(),
        texts: vec!(),
        // texts: [Vec::new(), Vec::new(), Vec::new()],
        background: (252, 246, 218),
    };

    render_map(&mut scene, scroll_offset, focus_pos, &map);

    render_character(&mut scene, scroll_offset, &pos, &sprites, &texts);

    let default_action = if let CombatState::WaitForUserAction(
        _,
        Some(InputContext::SelectedArea(pos, _, actions_at)),
    ) = &game.state
    {
        actions_at
            .iter()
            .cloned()
            .next()
            .map(|(action, cost)| (*pos, action, cost))
    } else {
        None
    };

    (
        scene,
        vec![ClickArea {
            clipping_area: viewport.clone(),
            action: Box::new(move |screen_pos| {
                let clicked_pos = screen_pos_to_map_pos(screen_pos, scroll_offset);

                if let Some((wp, action, cost)) = &default_action {
                    if clicked_pos.0.floor() == wp.0.floor()
                        && clicked_pos.1.floor() == wp.1.floor()
                    {
                        return UserInput::SelectAction((action.clone(), *cost));
                    }
                }
                UserInput::SelectWorldPos(screen_pos_to_map_pos(screen_pos, scroll_offset))
            }),
        }],
    )
}

fn render_character<'a>(
    scene: &mut Scene,
    offset: (i32, i32),
    positions: &ReadStorage<Position>,
    sprites_storage: &ReadStorage<Sprites>,
    texts: &ReadStorage<Text>,
) {
    for (pos, Sprites(sprites)) in (positions, sprites_storage).join() {
        let screen_pos = map_pos_to_screen_pos(pos.0, offset);

        for sprite in sprites.iter() {
            let (sx, sy, sw, sh) = sprite.region;

            scene.sprites.push(ScreenSprite {
                alpha: 255,
                pos: ScreenPos(screen_pos.0, screen_pos.1),
                offset: sprite.offset,
                source: (sprite.texture.clone(), sx, sy, sw, sh),
                target_size: (TILE_WIDTH, TILE_HEIGHT),
            });
        }
    }

    for (pos, text) in (positions, texts).join() {
        let (mut x, mut y) = map_pos_to_screen_pos(pos.0, offset);
        if let Some((dx, dy)) = text.offset {
            x += dx;
            y += dy;
        }

        scene.texts.push(ScreenText {
        // scene.texts[text.font as usize].push(ScreenText {
            font: text.font,
            text: DisplayStr::new(text.txt.clone()),
            pos: ScreenPos(x, y),
            color: text.color,
            background: text.background,
            padding: text.padding,
            border: text.border,
            min_width: 0,
            max_width: u32::max_value(),
        });
    }
}

fn render_map(
    scene: &mut Scene,
    offset: (i32, i32),
    focus_pos: Option<(i32, i32)>,
    map: &Map,
) {
    let (w, h) = (TILE_WIDTH, TILE_HEIGHT);

    for tile in map.tiles() {
        if let Some(tex_name) = map_tile_to_texture(tile) {
            let tile_pos = tile.to_world_pos();
            let (x, y) = map_pos_to_screen_pos(tile_pos, offset);

            scene.sprites.push(ScreenSprite {
                source: (tex_name, 0, 0, w, h),
                alpha: 255,
                offset: (0, 0),
                pos: ScreenPos(x, y),
                target_size: (w, h),
            })
        }
    }

    if let Some((x, y)) = focus_pos {
        scene.sprites.push(ScreenSprite {
            source: ("selected".to_string(), 0, 0, w, h),
            alpha: 255,
            offset: (0, 0),
            pos: ScreenPos(x, y),
            target_size: (w, h),
        })
    }
}

fn screen_pos_to_map_pos(p: ScreenPos, offset: (i32, i32)) -> WorldPos {
    let xs = (p.0 - offset.0) as f32;
    let ys = (p.1 - offset.1) as f32;
    let tw = TILE_WIDTH as f32;
    let th = TILE_HEIGHT as f32;
    let x = (xs - 832.0) / tw + ys / th;
    let y = ys / th - (xs - 832.0) / tw;

    WorldPos(x, y)
}

pub fn map_pos_to_screen_pos(wp: WorldPos, offset: (i32, i32)) -> (i32, i32) {
    let WorldPos(xw, yw) = wp;
    let tw = TILE_WIDTH as f32;
    let th = TILE_HEIGHT as f32;
    let x = tw * (xw - yw) / 2.0 - tw / 2.0 + 832.0;
    let y = th * (xw + yw) / 2.0;

    (x.round() as i32 + offset.0, y.round() as i32 + offset.1)
}

fn map_tile_to_texture(t: Tile) -> Option<String> {
    match t.tile_type() {
        TileType::Floor => Some(String::from("floor")),
        _ => None,
    }
}
