use specs::prelude::*;

use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::WindowCanvas;

use crate::components::*;
use crate::core::*;
use crate::ui::*;

pub const TILE_WIDTH: u32 = 96;
pub const TILE_HEIGHT: u32 = 96;

pub type SystemData<'a> = (
    ReadStorage<'a, Position>,
    ReadStorage<'a, Sprites>,
    ReadStorage<'a, Text>,
    Read<'a, Map>,
);

pub fn render(
    cvs: &mut WindowCanvas,
    viewport: &Rect,
    scroll_offset: (i32, i32),
    focus_pos: Option<(i32, i32)>,
    game: &CombatData,
    assets: &AssetRepo,

) -> Result<Vec<ClickArea>, String> {
    let (pos, sprites, texts, map): SystemData = game.world.system_data();

    cvs.set_draw_color(Color::RGB(252, 246, 218));
    cvs.fill_rect(viewport.clone())?;
    cvs.set_viewport(viewport.clone());

    render_map(cvs, scroll_offset, focus_pos, assets, &map)?;
    render_character(cvs, scroll_offset, &pos, &sprites, &texts, assets)?;
        
    cvs.set_viewport(None);

    let default_action = if let CombatState::WaitForUserAction(_, Some(InputContext::SelectedArea(pos, _, actions_at))) = &game.state {
        actions_at.iter().cloned().next().map(|(action, cost)| (*pos, action, cost))
    } else {
        None
    };

    Ok(vec!(ClickArea {
        clipping_area: viewport.clone(),
        action: Box::new(move |screen_pos| {
            let clicked_pos = screen_pos_to_map_pos(screen_pos, scroll_offset);

            if let Some((wp, action, cost)) = &default_action {
                if clicked_pos.0.floor() == wp.0.floor() && clicked_pos.1.floor() == wp.1.floor() {
                    return UserInput::SelectAction((action.clone(), *cost));
                }
            }
            UserInput::SelectWorldPos(screen_pos_to_map_pos(screen_pos, scroll_offset))
        })
    }))
}

fn render_sprite(
    cvs: &mut WindowCanvas,
    assets: &AssetRepo,
    screen_pos: (i32, i32),
    sprite: &Sprite,
) -> Result<(), String> {
    let sprite_tex = assets.texture(sprite.texture.as_str())?;
    let (xs, ys, w, h) = sprite.region;
    let (offset_x, offset_y) = sprite.offset;
    let (xt, yt) = screen_pos;
    let source = Rect::new(xs, ys, w, h);
    let target = Rect::new(xt + offset_x, yt + offset_y, TILE_WIDTH, TILE_HEIGHT);

    cvs.copy(&sprite_tex, Some(source), Some(target))
}

fn render_text<'a>(
    cvs: &mut WindowCanvas,
    assets: &AssetRepo,
    screen_pos: (i32, i32),
    text: &Text,
) -> Result<(), String> {
    let mut target_pos = screen_pos;
    let mut txt = assets.font(text.font)?.text(text.txt.to_owned());

    if let Some(offset) = text.offset {
        target_pos = (target_pos.0 + offset.0, target_pos.1 + offset.1);
    }

    if let Some((r, g, b, a)) = text.background {
        txt = txt.background(Color::RGBA(r, g, b, a));
    }

    if let Some((w, (r, g, b, a))) = text.border {
        txt = txt.border(w, Color::RGBA(r, g, b, a));
    }

    if let Some(padding) = text.padding {
        txt = txt.padding(padding);
    }

    txt.prepare().draw(cvs, target_pos)
}

fn render_character<'a>(
    cvs: &mut WindowCanvas,
    offset: (i32, i32),
    positions: &ReadStorage<Position>,
    sprites_storage: &ReadStorage<Sprites>,
    texts: &ReadStorage<Text>,
    assets: &AssetRepo,
) -> Result<(), String> {

    for (pos, Sprites(sprites)) in (positions, sprites_storage).join() {
        let screen_pos = map_pos_to_screen_pos(pos.0, offset);

        for sprite in sprites.iter() {
            render_sprite(cvs, assets, screen_pos, sprite)?;
        }
    }

    for (pos, text) in (positions, texts).join() {
        render_text(cvs, assets, map_pos_to_screen_pos(pos.0, offset), text)?;
    }

    Ok(())
}

fn render_map(
    cvs: &mut WindowCanvas,
    offset: (i32, i32),
    focus_pos: Option<(i32, i32)>,
    assets: &AssetRepo,
    map: &Map,
) -> Result<(), String> {
    let (w, h) = (TILE_WIDTH, TILE_HEIGHT);

    for tile in map.tiles() {
        if let Some(tex_name) = map_tile_to_texture(tile) {
            let tex = assets.texture(&tex_name)?;

            let tile_pos = tile.to_world_pos();
            let (x, y) = map_pos_to_screen_pos(tile_pos, offset);
            let target = Rect::new(x, y, w, h);

            cvs.copy(tex, None, Some(target))?;
        }
    }

    if let Some((x, y)) = focus_pos {
        let target = Rect::new(x, y, w, h);
        cvs.copy(assets.texture("selected")?, None, Some(target))?;
    }

    Ok(())
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
