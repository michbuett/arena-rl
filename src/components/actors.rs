use std::convert::TryInto;

use specs::prelude::{Builder, Entities, Entity, LazyUpdate, Read, ReadStorage, World};

use crate::components::{
    GameObjectCmp, ObstacleCmp, Position, Restriction, Sprites, WorldPos, ZLayerFloor,
    ZLayerGameObject,
};
use crate::core::{
    Act, Action, Actor, AttackOption, AttackType, Attr, AttrVal, Direction, GameObject, Item,
    MapPos, Obstacle, SpriteConfig, TextureMap, ID,
};

use super::HoverAnimation;

type SysData<'a> = (
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

fn lazy_insert_components<'a>(entity: Entity, obj: GameObject, data: SysData) {
    match obj {
        GameObject::Actor(a) => lazy_insert_component_for_actor(entity, a, data),

        GameObject::Item(p, i) => lazy_insert_component_for_items(entity, p, i, data),
    }
}

fn lazy_insert_component_for_actor<'a>(entity: Entity, a: Actor, data: SysData) {
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
            restrict_melee_attack: Restriction::ForTeam(
                a.team.clone(),
                0,
                to_difficulty(melee_def),
            ),
            restrict_ranged_attack: Restriction::ForAll(to_difficulty(range_def)),
        },
    );

    if a.is_flying() {
        updater.insert(entity, HoverAnimation::start(a.pos));
    } else {
        updater.remove::<HoverAnimation>(entity);
    }

    updater.insert(entity, ZLayerGameObject);
    updater.insert(entity, GameObjectCmp(GameObject::Actor(a)));
}

fn to_difficulty(av: AttrVal) -> u8 {
    (4 + av.val()).try_into().unwrap_or(0)
}

fn lazy_insert_component_for_items<'a>(entity: Entity, pos: WorldPos, item: Item, data: SysData) {
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

fn get_sprites_actor(a: &Actor, texture_map: &TextureMap) -> Sprites {
    let mut sprites: Vec<SpriteConfig> = vec![];

    sprites.append(
        &mut get_visual_elements(a)
            .iter()
            .filter_map(|key| texture_map.get(key))
            .cloned()
            .collect(),
    );

    if let Some(Act {
        action: Action::Attack(_, attack, attack_vector, _),
        ..
    }) = &a.pending_action
    {
        append_attack_indicator(&mut sprites, a.pos, attack, attack_vector, texture_map);
    }

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

fn get_circle_pos(angle_radian: f64, radius: f64) -> (i32, i32) {
    let (sin, cos) = angle_radian.sin_cos();
    ((radius * cos).round() as i32, (radius * sin).round() as i32)
}

fn append_attack_indicator(
    sprites: &mut Vec<SpriteConfig>,
    actor_pos: WorldPos,
    attack: &AttackOption,
    attack_vector: &Vec<(MapPos, bool, Option<(Obstacle, Option<ID>)>)>,
    texture_map: &TextureMap,
) {
    debug_assert!(!attack_vector.is_empty());

    let sprite_name = if let AttackType::Melee(..) = &attack.attack_type {
        "action-indicator-MeleeAttack"
    } else {
        "action-indicator-RangedAttack"
    };

    let p0 = actor_pos;
    let p1 = attack_vector.last().unwrap().0.to_world_pos();
    let dir = Direction::from_point(p0, p1);
    let angle = dir.as_radian();
    let mut s = texture_map.get(sprite_name).unwrap().clone();
    s.rotate = Some(Direction::from_point(p0, p1));
    s.offset = get_circle_pos(angle, 65.0);

    sprites.push(s);

    let num_icons = attack.required_effort;
    let icon_space = std::f64::consts::FRAC_PI_8;
    let icon_offset = 0.5 * (num_icons as f64 - 1.0) * icon_space;

    for i in 0..num_icons {
        let mut icon = texture_map.get("icon-dot-red").unwrap().clone();
        let icon_angle = angle - icon_offset + (i as f64) * icon_space;
        icon.offset = get_circle_pos(icon_angle, 50.0);
        sprites.push(icon)
    }
}

fn append_status_icons(sprites: &mut Vec<SpriteConfig>, a: &Actor, texture_map: &TextureMap) {
    let mut icons = vec![];
    let mut icons_wounds = (0..a.health.recieved_wounds)
        .map(|_| "icon-dot-red")
        .collect::<Vec<_>>();
    let mut icons_pain = (0..a.health.pain)
        .map(|_| "icon-dot-yellow")
        .collect::<Vec<_>>();
    let reserve_icon_name = if a.pending_action.is_none() {
        "icon-dot-blue"
    } else {
        "icon-action-Defend"
    };
    let mut icons_reserve = (0..a.available_effort())
        .map(|_| reserve_icon_name)
        .collect::<Vec<_>>();

    icons.append(&mut icons_wounds);
    icons.append(&mut icons_pain);
    icons.append(&mut icons_reserve);

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

    if let Some(Act {
        action: Action::Ambush(..),
        ..
    }) = &a.pending_action
    {
        visual_elements.push("action-indicator-Ambush".to_string());
    }

    for l in a.visuals() {
        visual_elements.push(l.to_string());
    }

    if !a.is_concious() {
        visual_elements.push("char-fx-ko".to_string());
    }

    visual_elements
}
