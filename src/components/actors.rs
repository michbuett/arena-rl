use std::convert::TryInto;

use specs::prelude::{Builder, Entities, Entity, LazyUpdate, Read, ReadStorage, World};

use crate::components::{
    GameObjectCmp, ObstacleCmp, Position, Restriction, Sprites, WorldPos,
    ZLayerFloor, ZLayerGameObject,
};
use crate::core::{Act, Action, Actor, Attr, GameObject, Item, SpriteConfig, TextureMap, AttrVal};

type SystemData<'a> = (
    Read<'a, LazyUpdate>,
    Read<'a, TextureMap>,
    ReadStorage<'a, Position>,
);

pub fn insert_game_object_components(obj: GameObject, world: &World) {
    let (updater, positions, texture_map, entities): (
        Read<LazyUpdate>,
        ReadStorage<Position>,
        Read<TextureMap>,
        Entities,
    ) = world.system_data();
    let e = updater.create_entity(&entities).build();

    lazy_insert_components(e, obj, (updater, texture_map, positions));
}

pub fn update_components(entity: Entity, obj: GameObject, world: &World) {
    lazy_insert_components(entity, obj, world.system_data());
}

pub fn remove_components(entity: Entity, world: &World) {
    let entities: Entities = world.system_data();
    let _ = entities.delete(entity);
}

fn lazy_insert_components<'a>(entity: Entity, obj: GameObject, data: SystemData) {
    match obj {
        GameObject::Actor(a) => lazy_insert_component_for_actor(entity, a, data),

        GameObject::Item(p, i) => lazy_insert_component_for_items(entity, p, i, data),
    }
}

fn lazy_insert_component_for_actor<'a>(entity: Entity, a: Actor, data: SystemData) {
    let (updater, texture_map, positions) = data;
    let melee_def = a.attr(Attr::MeleeDefence);
    let range_def = a.attr(Attr::RangeDefence);

    if positions.get(entity).is_none() {
        updater.insert(entity, Position(a.pos));
    }

    updater.insert(entity, get_sprites_actor(&a, &texture_map));
    updater.insert(
        entity,
        ObstacleCmp {
            restrict_movement: Restriction::ForAll(u8::MAX),
            restrict_melee_attack: Restriction::ForTeam(a.team.clone(), 0, to_difficulty(melee_def)),
            restrict_ranged_attack: Restriction::ForAll(to_difficulty(range_def)),
        },
    );

    updater.insert(entity, ZLayerGameObject);
    updater.insert(entity, GameObjectCmp(GameObject::Actor(a)));
}

fn to_difficulty(av: AttrVal) -> u8 {
    (4 + av.val()).try_into().unwrap_or(0)
}

fn lazy_insert_component_for_items<'a>(
    entity: Entity,
    pos: WorldPos,
    item: Item,
    data: SystemData,
) {
    let (updater, texture_map, positions) = data;

    if positions.get(entity).is_none() {
        updater.insert(entity, Position(pos));
    }

    updater.insert(entity, get_sprites_items(&item, &texture_map));

    updater.insert(
        entity,
        ObstacleCmp {
            restrict_movement: Restriction::ForAll(u8::MAX),
            restrict_melee_attack: Restriction::ForAll(0),
            restrict_ranged_attack: Restriction::ForAll(0),
        },
    );

    updater.insert(entity, ZLayerFloor);
    updater.insert(entity, GameObjectCmp(GameObject::Item(pos, item)));
}

// fn get_txt_actor(a: &Actor) -> Option<Text> {
//     let unformated_text = if a.health.recieved_wounds > 0 {
//         Some(
//             Text::new(format!("{}", a.health.recieved_wounds), FontFace::Big)
//                 .color(200, 21, 22, 255),
//         )
//     } else if a.health.focus < 0 {
//         Some(Text::new("-".to_string(), FontFace::Normal).color(200, 201, 22, 255))
//     } else if a.health.focus > 0 {
//         Some(Text::new("+".to_string(), FontFace::Normal).color(200, 201, 22, 255))
//     } else {
//         None
//     };

//     unformated_text.map(|t: Text| {
//         t.align(Align::MidCenter)
//             .background(200, 201, 202, 128)
//             .padding(5)
//             .offset(0, 16)
//     })
// }

fn get_sprites_actor(a: &Actor, texture_map: &TextureMap) -> Sprites {
    let mut visual_elements = vec![format!("bg_team_{}", a.team.1)];

    if a.active {
        visual_elements.push("bg_active".to_string());
    }

    for l in a.look() {
        visual_elements.push(l);
    }

    if !a.is_concious() {
        visual_elements.push("char-fx-ko".to_string());
    }

    let mut sprites: Vec<SpriteConfig> = visual_elements
        .iter()
        .filter_map(|key| texture_map.get(key))
        .cloned()
        .collect();

    sprites.append(&mut get_icons_actor(a, texture_map));

    Sprites::new(sprites)
}

fn get_icons_actor(a: &Actor, tm: &TextureMap) -> Vec<SpriteConfig> {
    let mut icons = vec![];
    let num_wounds = &a.health.recieved_wounds;
    let pain = &a.health.pain;

    icons.append(&mut (0..*num_wounds).map(|_| "icon-dot-red").collect());
    icons.append(&mut (0..*pain).map(|_| "icon-dot-yellow").collect());

    match &a.pending_action {
        Some(Act {
            action: Action::Attack(_, attack, _, _),
            ..
        }) => {
            icons.append(
                &mut (0..attack.required_effort)
                    .map(|_| "icon-action-Attack")
                    .collect(),
            );
        }

        _ => {}
    }

    icons.append(&mut (0..a.available_effort()).map(|_| "icon-dot-blue").collect());

    icons
        .iter()
        .enumerate()
        .map(|(i, icon_name)| {
            let mut s = tm.get(*icon_name).unwrap().clone();
            s.offset = get_icon_pos_left(i as u8);
            s
        })
        .collect()
}

fn get_sprites_items(item: &Item, texture_map: &TextureMap) -> Sprites {
    let sprites = item
        .look
        .iter()
        .filter_map(|(_, key)| texture_map.get(key))
        .cloned()
        .collect();

    Sprites::new(sprites)
}

fn get_icon_pos_left(num: u8) -> (i32, i32) {
    let pi = std::f64::consts::PI;
    let alpha = std::f64::consts::FRAC_PI_8;
    let (sin, cos) = (pi + alpha * num as f64).sin_cos();

    ((40.0 * cos).round() as i32, (-40.0 * sin).round() as i32)
}
