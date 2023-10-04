use std::collections::HashMap;
use std::vec;

use specs::prelude::*;

use crate::components::{Position, Sprites, Text, ZLayerFX, ZLayerFloor, ZLayerGameObject};
use crate::core::{
    CombatData, CombatState, InputContext, Map, MapPos, TextureMap, Tile, TileType, UserInput,
    WorldPos,
};
use crate::ui::{Align, ClickArea, Scene, ScreenCoord, ScreenPos, ScreenSprite};

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

type DefaultAction = (Option<MapPos>, HashMap<MapPos, UserInput>);

pub fn render(
    viewport: (i32, i32, u32, u32),
    scroll_offset: (i32, i32),
    game: &CombatData,
) -> (Scene, Vec<ClickArea>) {
    let (pos, sprites, texts, zlayer_floor, zlayer_gameobj, zlayer_fx, map, texture_map): SystemData = game.world.system_data();
    let mut scene = Scene::empty().set_background(252, 246, 218);
    let default_action = get_default_action(&game);

    // (NOTE: sprites added first to the scene appear "behind" sprites which are added later)
    // (1) draw map tiles
    render_map(
        &mut scene,
        scroll_offset,
        default_action.0,
        &map,
        &texture_map,
    );

    // (2) draw items on the ground (e.g. blood drops, ...)
    render_floor_objects(&mut scene, scroll_offset, &pos, &sprites, &zlayer_floor);

    // (3) draw game objects (e.g. characters, obstacles, ...)
    render_game_objects(&mut scene, scroll_offset, &pos, &sprites, &zlayer_gameobj);

    // (4) draw visual effects
    render_fx(
        &mut scene,
        scroll_offset,
        &default_action,
        &texture_map,
        &pos,
        &sprites,
        &zlayer_fx,
    );

    // (5) draw texts which are positioned relative to game objects
    render_texts(&mut scene, scroll_offset, &pos, &texts);

    (
        scene,
        render_action_buttons(viewport, scroll_offset, default_action),
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

        scene
            .sprites
            .push(ScreenSprite(p, Align::MidCenter, icon_sprite.sample(0)));
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

        scene.texts.push(text.into_screen_text(ScreenPos(x, y)));
    }
}

fn render_map(
    scene: &mut Scene,
    offset: (i32, i32),
    selected_pos: Option<MapPos>,
    map: &Map,
    texture_map: &TextureMap,
) {
    for tile in map.tiles() {
        if let Some(sprite_config) = map_tile_to_texture(tile).and_then(|tn| texture_map.get(&tn)) {
            let tile_pos = tile.to_world_pos();
            let p = ScreenCoord::from_world_pos(tile_pos).to_screen_pos(offset);

            scene
                .sprites
                .push(ScreenSprite(p, Align::MidCenter, sprite_config.sample(0)));
        }
    }

    if let Some(p) = selected_pos {
        if let Some(sprite_config) = texture_map.get("selected") {
            let p = ScreenCoord::from_world_pos(p.to_world_pos()).to_screen_pos(offset);

            scene
                .sprites
                .push(ScreenSprite(p, Align::MidCenter, sprite_config.sample(0)));
        }
    }
}

fn render_action_buttons<'a>(
    viewport: (i32, i32, u32, u32),
    scroll_offset: (i32, i32),
    default_action: DefaultAction,
) -> Vec<ClickArea> {
    let mut click_areas = vec![];

    click_areas.push(ClickArea {
        clipping_area: viewport,
        action: Box::new(move |screen_pos| {
            let clicked_pos = screen_pos_to_map_pos(screen_pos, scroll_offset);

            if let Some(selected_pos) = default_action.0 {
                if clicked_pos == selected_pos {
                    if let Some(input) = default_action.1.get(&clicked_pos) {
                        return input.clone();
                    }
                }
            }

            UserInput::SelectWorldPos(clicked_pos)
        }),
    });

    click_areas
}

fn screen_pos_to_map_pos(screen_pos: ScreenPos, scroll_offset: (i32, i32)) -> MapPos {
    MapPos::from_world_pos(
        ScreenCoord::new(
            screen_pos.0 - scroll_offset.0,
            screen_pos.1 - scroll_offset.1,
        )
        .to_world_pos(),
    )
}

fn map_tile_to_texture(t: Tile) -> Option<String> {
    match t.tile_type() {
        TileType::Floor => Some(String::from("floor")),
        _ => None,
    }
}

fn get_default_action(game: &CombatData) -> DefaultAction {
    if let CombatState::WaitForUserInput(ctxt, selected_pos) = &game.state {
        let selected_mpos = selected_pos.as_ref().map(|sp| sp.pos.clone());

        match ctxt {
            InputContext::ActivateActor {
                team,
                possible_actors,
                selected_card_idx: Some(idx),
                ..
            } => {
                let card = game.turn.get_team(*team).hand[*idx];
                let activations = possible_actors
                    .iter()
                    .filter_map(|(pos, (id, max))| {
                        if card.value <= *max {
                            Some((*pos, UserInput::AssigneActivation(*id, *team, *idx)))
                        } else {
                            None
                        }
                    })
                    .collect::<HashMap<_, _>>();

                return (selected_mpos, activations);
            }

            InputContext::SelectAction { options } => {
                let mut user_inputs = HashMap::new();
                for (pos, player_actions) in options.iter() {
                    if !player_actions.is_empty() {
                        user_inputs.insert(
                            *pos,
                            UserInput::SelectPlayerAction(player_actions.first().unwrap().clone()),
                        );
                    }
                }
                return (selected_mpos, user_inputs);
            }

            _ => {
                return (selected_mpos, HashMap::new());
            }
        }
    }

    (None, HashMap::new())
}

fn get_icons(action: &DefaultAction) -> Vec<(WorldPos, String)> {
    match &action.1 {
        // Some(player_action) => map_player_action_to_floor_icon(player_action),
        _ => vec![],
    }
}
