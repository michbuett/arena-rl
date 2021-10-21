use specs::prelude::*;

use crate::components::{Position, Sprites, Text, ZLayerFX, ZLayerFloor, ZLayerGameObject};
use crate::core::{
    Act, Action, CombatData, CombatState, DisplayStr, InputContext, Map, MapPos, TextureMap, Tile,
    TileType, UserInput, WorldPos,
};
use crate::ui::{
    Align, ClickArea, Scene, ScreenCoord, ScreenPos, ScreenSprite, ScreenText, TILE_WIDTH,
};

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

type DefaultAction = (Option<MapPos>, Option<Act>);

const EFFORT_BTN_SIZE: u8 = 50;

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

    let click_areas = render_action_buttons(&mut scene, viewport, scroll_offset, &default_action);

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

    (scene, click_areas)
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
    scene: &mut Scene,
    viewport: (i32, i32, u32, u32),
    scroll_offset: (i32, i32),
    default_action: &DefaultAction,
) -> Vec<ClickArea> {
    let mut click_areas = vec![];

    if let (Some(mp), Some(act)) = default_action.clone() {
        if let Some(max_effort) = act.allocated_effort {
            let default_effort = ((max_effort + 1) / 2) as u8;
            let p = ScreenCoord::from_world_pos(mp.to_world_pos()).to_screen_pos(scroll_offset);
            let total_height = max_effort as i32 * EFFORT_BTN_SIZE as i32;
            let x = p.0 + (TILE_WIDTH / 2) as i32;
            let y0 = p.1 - (total_height / 2);

            for i in 1..=max_effort {
                let a = act.clone();
                let y = y0 + ((i - 1) * EFFORT_BTN_SIZE) as i32;
                let pos = ScreenPos(x, y);
                let txt = DisplayStr::new(format!("{}", i));

                scene.texts.push(
                    ScreenText::new(txt, pos)
                        .background((250, 251, 252, 180))
                        .text_align(Align::MidCenter)
                        .width(EFFORT_BTN_SIZE.into())
                        .height(EFFORT_BTN_SIZE.into()),
                );


                click_areas.push(ClickArea {
                    clipping_area: (x, y, EFFORT_BTN_SIZE.into(), EFFORT_BTN_SIZE.into()),
                    action: Box::new(move |_| {
                        // let act = Act::new(a.action.clone()).delay(a.delay).effort(i);
                        // let act = Act::new(a.action.clone()).delay(a.delay).effort(i);
                        return UserInput::SelectAction(a.clone().effort(i));
                    }),
                });
            }
        }

        click_areas.push(ClickArea {
            clipping_area: viewport,
            action: Box::new(move |screen_pos| {
                let clicked_pos = screen_pos_to_map_pos(screen_pos, scroll_offset);

                if clicked_pos == mp {
                    return UserInput::SelectAction(act.clone());
                }

                UserInput::SelectWorldPos(clicked_pos)
            }),
        });

    } else {
        click_areas.push(ClickArea {
            clipping_area: viewport,
            action: Box::new(move |screen_pos| {
                UserInput::SelectWorldPos(screen_pos_to_map_pos(screen_pos, scroll_offset))
            }),
        });
    }

    // if let Some(action) = default_action.1  {
    // }

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
    match &game.state {
        CombatState::WaitForUserAction(_, Some(InputContext::SelectedArea(pos, _, actions_at))) => {
            let selected_pos = Some(*pos);
            let selected_action = actions_at.iter().cloned().next().map(|a| a);

            (selected_pos, selected_action)
        }

        _ => (None, None),
    }
}

fn get_icons(action: &DefaultAction) -> Vec<(WorldPos, String)> {
    match action {
        (_, Some(Act { action: Action::MoveTo(p), .. })) => p
            .iter()
            .map(|tile| (tile.to_world_pos(), "icon-floor-MoveTo".to_string()))
            .collect(),

        (_, Some(Act { action: Action::RangeAttack(_, _, attack_vector, _), .. })) => attack_vector
            .iter()
            .map(|(map_pos, _is_target, obs)| {
                let num = if let Some(..) = obs { 2 } else { 1 };

                (
                    map_pos.to_world_pos(),
                    format!("icon-floor-RangedAttack-{}", num),
                )
            })
            .collect(),

        _ => vec![],
    }
}
