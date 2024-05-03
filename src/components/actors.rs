use std::num::NonZeroU8;

use specs::prelude::*;

use crate::components::{
    GameObjectCmp, ObstacleCmp, Position, Sprites, WorldPos, ZLayerFloor, ZLayerGameObject,
};
use crate::core::{Actor, Card, GameObject, Item, Obstacle, SpriteConfig, Suite, TextureMap};

use super::{Hitbox, HoverAnimation, Text};

pub fn insert_game_object(obj: &GameObject, entities: &Entities, updater: &LazyUpdate) -> Entity {
    let e = updater.create_entity(&entities).build();

    match obj {
        GameObject::Actor(a) => insert_actor(e, a, updater),
        GameObject::Item(pos, _) => insert_item(e, *pos, updater),
    }

    e
}

fn insert_actor<'a>(entity: Entity, a: &Actor, updater: &LazyUpdate) {
    updater.insert(entity, Position(a.pos));

    updater.insert(
        entity,
        ObstacleCmp {
            movement: (
                Some(Obstacle::Blocker),
                Some(Obstacle::Blocker),
                Some(Obstacle::Blocker),
            ),
            reach: Some(Hitbox::new_normal_actor()),
        },
    );

    updater.insert(entity, ZLayerGameObject);
}

fn insert_item<'a>(entity: Entity, pos: WorldPos, updater: &LazyUpdate) {
    updater.insert(entity, Position(pos));
    updater.insert(entity, ZLayerFloor);
    updater.insert(
        entity,
        ObstacleCmp {
            movement: (
                Some(Obstacle::Impediment(NonZeroU8::new(1).unwrap(), 0)),
                None,
                None,
            ),
            reach: None,
        },
    );
}

pub fn update_game_object(
    entity: Entity,
    obj: &GameObject,
    texture_map: &TextureMap,
    updater: &LazyUpdate,
) {
    // println!("[DEBUG] update_game_object {:?}", entity);
    match obj {
        GameObject::Actor(a) => update_actor(entity, a, texture_map, updater),

        GameObject::Item(pos, item) => {
            updater.insert(entity, get_sprites_items(&item, &texture_map));
            updater.insert(entity, GameObjectCmp(GameObject::Item(*pos, item.clone())));
        }
    }
}

fn update_actor(entity: Entity, a: &Actor, texture_map: &TextureMap, updater: &LazyUpdate) {
    // println!("[DEBUG] update_actor {:?} (at {:?})", a.name, a.pos);

    if let Some(text) = get_text_actor(a) {
        updater.insert(entity, text);
    } else {
        updater.remove::<Text>(entity);
    }

    updater.insert(entity, get_sprites_actor(a, texture_map));

    if a.is_flying() {
        updater.insert(entity, HoverAnimation::start());
    } else {
        updater.remove::<HoverAnimation>(entity);
    }

    updater.insert(entity, GameObjectCmp(GameObject::Actor(a.clone())));
}

fn get_text_actor(a: &Actor) -> Option<Text> {
    return a
        .activations
        .iter()
        .map(activation_str)
        .fold(None, |acc, s| match acc {
            Some(beginng) => Some(format!("{}, {}", beginng, s)),
            None => Some(s),
        })
        .map(|txt| Text::new(txt, crate::ui::FontFace::Normal).offset(-32, 16));
}

fn activation_str(card: &Card) -> String {
    format!(
        "[{}{}]",
        card.value,
        match card.suite {
            Suite::PhysicalStr => "C",
            Suite::MentalStr => "H",
            Suite::PhysicalAg => "S",
            Suite::MentalAg => "D",
            _ => panic!("Suite not allowed: {:?}", card.suite),
        }
    )
}

fn get_sprites_actor(a: &Actor, texture_map: &TextureMap) -> Sprites {
    let mut sprites: Vec<SpriteConfig> = vec![];

    sprites.append(
        &mut get_visual_elements(a)
            .iter()
            .filter_map(|key| texture_map.get(key))
            .cloned()
            .collect(),
    );

    append_status_icons(&mut sprites, a, texture_map);

    Sprites::new(sprites)
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

// fn get_circle_pos(angle_radian: f64, radius: f64) -> (i32, i32) {
//     let (sin, cos) = angle_radian.sin_cos();
//     ((radius * cos).round() as i32, (radius * sin).round() as i32)
// }

// fn append_attack_indicator(
//     sprites: &mut Vec<SpriteConfig>,
//     actor_pos: WorldPos,
//     attack: &AttackOption,
//     attack_vector: &AttackVector,
//     // charge: u8,
//     texture_map: &TextureMap,
// ) {
//     debug_assert!(!attack_vector.is_empty());

//     let sprite_name = if let AttackType::Melee(..) = &attack.attack_type {
//         "action-indicator-MeleeAttack"
//     } else {
//         "action-indicator-RangedAttack"
//     };

//     let p0 = actor_pos;
//     let p1 = attack_vector.last().unwrap().0.to_world_pos();
//     let dir = Direction::from_point(p0, p1);
//     let angle = dir.as_radian();
//     let mut s = texture_map.get(sprite_name).unwrap().clone();
//     s.rotate = Some(Direction::from_point(p0, p1));
//     s.offset = get_circle_pos(angle, 65.0);

//     sprites.push(s);

//     let num_icons = attack.allocated_effort;
//     // let num_icons = attack.required_effort;
//     let icon_space = std::f64::consts::FRAC_PI_8;
//     let icon_offset = 0.5 * (num_icons as f64 - 1.0) * icon_space;

//     for i in 0..num_icons {
//         let mut icon = texture_map.get("icon-dot-red").unwrap().clone();
//         let icon_angle = angle - icon_offset + (i as f64) * icon_space;
//         icon.offset = get_circle_pos(icon_angle, 50.0);
//         sprites.push(icon)
//     }
// }

fn append_status_icons(sprites: &mut Vec<SpriteConfig>, a: &Actor, texture_map: &TextureMap) {
    let icons = (0..a.health.recieved_wounds)
        .map(|_| "icon-dot-red")
        .chain((0..a.health.pain).map(|_| "icon-dot-yellow"))
        .collect::<Vec<_>>();

    let icon_space = 16;
    let icon_offset = (icons.len() as i32 - 1) * icon_space / 2;

    for (i, icon_name) in icons.iter().enumerate() {
        let mut icon = texture_map.get(*icon_name).unwrap().clone();
        let xpos = i as i32 * icon_space - icon_offset;
        icon.offset = (xpos, 16);
        sprites.push(icon)
    }
}

fn get_visual_elements(a: &Actor) -> Vec<String> {
    let mut visual_elements = vec![];

    if a.active {
        visual_elements.push("bg_active".to_string());
    }

    for l in a.visuals() {
        visual_elements.push(l.to_string());
    }

    if a.is_alive() && !a.is_concious() {
        visual_elements.push("char-fx-ko".to_string());
    }

    visual_elements
}
