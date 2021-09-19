use specs::prelude::*;

use crate::components::{Position, Sprites, Text, ZLayerFX, ZLayerFloor, ZLayerGameObject};
use crate::core::{
    Action, CombatData, CombatState, DisplayStr, InputContext, Map, TextureMap, Tile, TileType,
    UserInput, WorldPos, MapPos, 
};
use crate::ui::{ClickArea, Scene, ScreenPos, ScreenCoord, ScreenSprite, ScreenText, TILE_WIDTH};

pub type SystemData<'a> = (
    ReadStorage<'a, Position>,
    ReadStorage<'a, Sprites>,
    ReadStorage<'a, Text>,
    ReadStorage<'a, ZLayerFloor>,
    ReadStorage<'a, ZLayerGameObject>,
    ReadStorage<'a, ZLayerFX>,
    Read<'a, Map>,
    Read<'a, TextureMap>,
);

type DefaultAction = (Option<WorldPos>, Option<(Action, u8)>);

pub fn render(
    viewport: (i32, i32, u32, u32),
    scroll_offset: (i32, i32),
    game: &CombatData,
) -> (Scene, Vec<ClickArea>) {
    let (pos, sprites, texts, zlayer_floor, zlayer_gameobj, zlayer_fx, map, texture_map): SystemData = game.world.system_data();
    let mut scene = Scene::empty().set_background(252, 246, 218);
    let default_action = get_default_action(&game);

    render_map(
        &mut scene,
        scroll_offset,
        &default_action,
        &map,
        &texture_map,
    );

    render_floor_objects(&mut scene, scroll_offset, &pos, &sprites, &zlayer_floor);
    render_game_objects(&mut scene, scroll_offset, &pos, &sprites, &zlayer_gameobj);
    render_fx(&mut scene, scroll_offset, &default_action, &texture_map, &pos, &sprites, &zlayer_fx);
    render_texts(&mut scene, scroll_offset, &pos, &texts);

    (
        scene,
        vec![ClickArea {
            clipping_area: viewport,
            action: Box::new(move |screen_pos| {
                let screen_coord = ScreenCoord::new(
                    screen_pos.0 - TILE_WIDTH as i32 / 2 - scroll_offset.0,
                    screen_pos.1 - scroll_offset. 1
                );
                let clicked_pos = screen_coord.to_world_pos();

                if let (Some(wp), Some((action, cost))) = &default_action {
                    if MapPos::from_world_pos(clicked_pos) == MapPos::from_world_pos(*wp) {
                        return UserInput::SelectAction((action.clone(), *cost));
                    }
                }

                UserInput::SelectWorldPos(clicked_pos)
            }),
        }],
    )
}

fn render_floor_objects<'a>(
    scene: &mut Scene,
    offset: (i32, i32),
    positions: &ReadStorage<Position>,
    visuals: &ReadStorage<Sprites>,
    zlayer_floor: &ReadStorage<ZLayerFloor>,
) {
    for (_, p, sprite_cmp) in (zlayer_floor, positions, visuals).join() {
        for sprite in sprite_cmp.sample(ScreenCoord::from_world_pos(p.0).to_screen_pos(offset)) {
            scene.sprites.push(sprite);
        }
    }
}

fn render_fx<'a>(
    scene: &mut Scene,
    offset: (i32, i32),
    default_action: &DefaultAction,
    texture_map: &TextureMap,
    positions: &ReadStorage<Position>,
    visuals: &ReadStorage<Sprites>,
    zlayer_fx: &ReadStorage<ZLayerFX>,
) {
    for (_, p, sprite_cmp) in (zlayer_fx, positions, visuals).join() {
        for sprite in sprite_cmp.sample(ScreenCoord::from_world_pos(p.0).to_screen_pos(offset)) {
            scene.sprites.push(sprite);
        }
    }

    for (wp, icon_name) in get_icons(default_action) {
        let icon_sprite = texture_map.get(&icon_name).unwrap();
        let p = ScreenCoord::from_world_pos(wp).to_screen_pos(offset);

        scene.sprites.push(ScreenSprite(p, icon_sprite.sample(0)));
    }
}

fn render_game_objects<'a>(
    scene: &mut Scene,
    offset: (i32, i32),
    positions: &ReadStorage<Position>,
    visuals: &ReadStorage<Sprites>,
    zlayer_gameobj: &ReadStorage<ZLayerGameObject>,
) {
    let mut data = (zlayer_gameobj, positions, visuals)
        .join()
        .map(|(_, Position(p), sc)| (ScreenCoord::from_world_pos(*p), sc))
        .collect::<Vec<_>>();

    data.sort_by(|(p1, _), (p2, _)| {
        if p1.z_layer() < p2.z_layer() {
            std::cmp::Ordering::Less
        } else {
            std::cmp::Ordering::Greater
        }
    });

    for (p, sprite_cmp) in data {
        for sprite in sprite_cmp.sample(p.to_screen_pos(offset)) {
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
        let ScreenPos(mut x, mut y) = ScreenCoord::from_world_pos(pos.0).to_screen_pos(offset);
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
            alpha: text.alpha,
            min_width: 0,
            max_width: u32::max_value(),
            scale: text.scale,
        });
    }
}

fn render_map(
    scene: &mut Scene,
    offset: (i32, i32),
    default_action: &DefaultAction,
    map: &Map,
    texture_map: &TextureMap,
) {
    for tile in map.tiles() {
        if let Some(sprite_config) = map_tile_to_texture(tile).and_then(|tn| texture_map.get(&tn)) {
            let tile_pos = tile.to_world_pos();
            let p = ScreenCoord::from_world_pos(tile_pos).to_screen_pos(offset);

            scene.sprites.push(ScreenSprite(p, sprite_config.sample(0)));
        }
    }

    if let Some(wp) = get_selected_tile(default_action) {
        if let Some(sprite_config) = texture_map.get("selected") {
            let p = ScreenCoord::from_world_pos(wp).to_screen_pos(offset);

            scene.sprites.push(ScreenSprite(p, sprite_config.sample(0)));
        }
    }
    // for wp in get_highlighted_tiles(default_action) {
    //     if let Some(sprite_config) = texture_map.get("selected") {
    //         let p = ScreenCoord::from_world_pos(wp).to_screen_pos(offset);

    //         scene.sprites.push(ScreenSprite(p, sprite_config.sample(0)));
    //     }
    // }
}

fn map_tile_to_texture(t: Tile) -> Option<String> {
    match t.tile_type() {
        TileType::Floor => Some(String::from("floor")),
        _ => None,
    }
}

fn get_default_action(game: &CombatData) -> DefaultAction {
    match &game.state {
        CombatState::WaitForUserAction(_, Some(InputContext::SelectedArea(pos, _, actions_at))) => {
            let selected_pos = Some(*pos);
            let selected_action = actions_at
                .iter()
                .cloned()
                .next()
                .map(|(action, cost)| (action, cost));

            (selected_pos, selected_action)
        }

        _ => (None, None),
    }
}

fn get_selected_tile(default_action: &DefaultAction) -> Option<WorldPos> {
    match default_action {
        (Some(pos), _) => Some(*pos),
        _ => None,
    }
}


fn get_icons(action: &DefaultAction) -> Vec<(WorldPos, String)> {
    match action {
        (_, Some((Action::MoveTo(p), _))) => p
            .iter()
            .map(|tile| (tile.to_world_pos(), "icon-floor-MoveTo".to_string()))
            .collect(),

        (_, Some((Action::RangeAttack(_, _, attack_vector, _), _))) => attack_vector
            .iter()
            .map(|(map_pos, _is_target, obs)| {
                let num = if let Some(..) = obs {
                    2
                } else {
                    1
                };

                (map_pos.to_world_pos(), format!("icon-floor-RangedAttack-{}", num))
            })
            .collect(),

        _ => vec![],
    }
}
