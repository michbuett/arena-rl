use specs::prelude::*;

use sdl2::rect::Rect;

use crate::components::{Position, Text, Sprites};
use crate::core::{
    Action, CombatData, CombatState, DisplayStr, InputContext, Map, Tile, TileType, UserInput,
    WorldPos, TextureMap,
};
use crate::ui::{ClickArea, Scene, ScreenPos, ScreenText, ScreenSprite, TILE_WIDTH};

pub type SystemData<'a> = (
    ReadStorage<'a, Position>,
    ReadStorage<'a, Sprites>,
    ReadStorage<'a, Text>,
    Read<'a, Map>,
    Read<'a, TextureMap>,
);

pub fn render(
    viewport: &Rect,
    scroll_offset: (i32, i32),
    game: &CombatData,
) -> (Scene, Vec<ClickArea>) {
    let (pos, sprites, texts, map, texture_map): SystemData = game.world.system_data();

    let mut scene = Scene {
        sprites: Vec::new(),
        texts: vec![],
        background: (252, 246, 218),
    };

    render_map(&mut scene, scroll_offset, game, &map, &texture_map);
    render_character(&mut scene, scroll_offset, &pos, &sprites);
    render_texts(&mut scene, scroll_offset, &pos, &texts);

    let default_action = get_default_action(&game);

    (
        scene,
        vec![ClickArea {
            clipping_area: viewport.clone(),
            action: Box::new(move |screen_pos| {
                let screen_pos = ScreenPos(screen_pos.0 - TILE_WIDTH as i32 / 2, screen_pos.1);
                let clicked_pos = screen_pos.to_world_pos(scroll_offset);

                if let Some((wp, action, cost)) = &default_action {
                    if clicked_pos.0.floor() == wp.0.floor()
                        && clicked_pos.1.floor() == wp.1.floor()
                    {
                        return UserInput::SelectAction((action.clone(), *cost));
                    }
                }

                UserInput::SelectWorldPos(clicked_pos)
            }),
        }],
    )
}

fn render_character<'a>(
    scene: &mut Scene,
    offset: (i32, i32),
    positions: &ReadStorage<Position>,
    visuals: &ReadStorage<Sprites>,
) {
    for (pos, sprite_cmp) in (positions, visuals).join() {
        let p = ScreenPos::from_world_pos(pos.0, offset);

        for sprite in sprite_cmp.sample(p) {
            scene.sprites.push(sprite);
        }
    }
}

fn render_texts<'a>(
    scene: &mut Scene,
    offset: (i32, i32),
    positions: &ReadStorage<Position>,
    texts: &ReadStorage<Text>,
) {
    for (pos, text) in (positions, texts).join() {
        let ScreenPos(mut x, mut y) = ScreenPos::from_world_pos(pos.0, offset);
        if let Some((dx, dy)) = text.offset {
            x += dx;
            y += dy;
        }

        scene.texts.push(ScreenText {
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
    game: &CombatData,
    map: &Map,
    texture_map: &TextureMap,
) {
    for tile in map.tiles() {
        if let Some(sprite_config) = map_tile_to_texture(tile).and_then(|tn| texture_map.get(&tn)) {
            let tile_pos = tile.to_world_pos();
            let p = ScreenPos::from_world_pos(tile_pos, offset);

            scene.sprites.push(ScreenSprite(p, sprite_config.sample(0)));
            // scene.sprites.push(sprite_config.into_screen_sprite(p, 0));
        }
    }

    if let Some(wp) = get_focus_pos(&game.state) {
        if let Some(sprite_config) = texture_map.get("selected") {
            let p = ScreenPos::from_world_pos(wp, offset);

            scene.sprites.push(ScreenSprite(p, sprite_config.sample(0)));
            // scene.sprites.push(sprite_config.into_screen_sprite(p, 0));
        }
    }
}

fn map_tile_to_texture(t: Tile) -> Option<String> {
    match t.tile_type() {
        TileType::Floor => Some(String::from("floor")),
        _ => None,
    }
}

fn get_focus_pos<'a>(game_state: &CombatState) -> Option<WorldPos> {
    match game_state {
        CombatState::WaitForUserAction(_, Some(InputContext::SelectedArea(p, _, _))) => Some(*p),
        _ => None,
    }
}

fn get_default_action(game: &CombatData) -> Option<(WorldPos, Action, u8)> {
    match &game.state {
        CombatState::WaitForUserAction(_, Some(InputContext::SelectedArea(pos, _, actions_at))) => {
            actions_at
                .iter()
                .cloned()
                .next()
                .map(|(action, cost)| (*pos, action, cost))
        }

        _ => None,
    }
}
