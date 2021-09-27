use specs::prelude::*;

use crate::components::*;
use crate::core::{Action, Attr, GameObject, Actor, Item, TextureMap};

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

fn lazy_insert_components<'a>(
    entity: Entity,
    obj: GameObject,
    data: SystemData,
) {
    match obj {
        GameObject::Actor(a) =>
            lazy_insert_component_for_actor(entity, a, data),

        GameObject::Item(p, i) =>
            lazy_insert_component_for_items(entity, p, i, data),
    }
}

fn lazy_insert_component_for_actor<'a>(
    entity: Entity,
    a: Actor,
    data: SystemData,
) {
    let (updater, texture_map, positions) = data;
    let melee_def = a.attr(Attr::MeleeDefence).val();
    let range_def = a.attr(Attr::RangeDefence).val();


    if positions.get(entity).is_none() {
        updater.insert(entity, Position(a.pos));
    }

    updater.insert(entity, get_txt_actor(&a));
    updater.insert(entity, get_sprites_actor(&a, &texture_map));
    updater.insert(
        entity,
        ObstacleCmp {
            restrict_movement: Restriction::ForAll(Some(i8::MAX)),
            restrict_melee_attack: Restriction::ForTeam(a.team.clone(), None, Some(melee_def)),
            restrict_ranged_attack: Restriction::ForAll(Some(range_def)),
        },
    );

    updater.insert(entity, ZLayerGameObject);
    updater.insert(entity, GameObjectCmp(GameObject::Actor(a)));
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
            restrict_movement: Restriction::ForAll(Some(i8::MAX)),
            restrict_melee_attack: Restriction::ForAll(None),
            restrict_ranged_attack: Restriction::ForAll(None),
        },
    );

    updater.insert(entity, ZLayerFloor);
    updater.insert(entity, GameObjectCmp(GameObject::Item(pos, item)));
}

fn map_action_to_sprite(a: &Action) -> String {
    match a {
        Action::MoveTo(_) => "icon-action-MoveTo",
        Action::MeleeAttack(..) => "icon-action-MeleeAttack",
        Action::RangeAttack(..) => "icon-action-RangedAttack",
        Action::Charge(..) => "icon-action-Charge",
        Action::Done(..) => "icon-action-Done",
        _ => "icon-action-Unknown",
    }
    .to_string()
}

fn get_txt_actor(a: &Actor) -> Text {
    let health = a.health();
    if health.wounds.0 > 0 {
        return Text::new(format!("{}", health.wounds.0), FontFace::Normal)
            .color(200, 21, 22, 255)
            .offset(39, 85);
    }

    if health.pain > 0 {
        return Text::new(format!("-{}", health.pain), FontFace::Normal)
            .color(200, 201, 22, 255)
            .offset(39, 85);
    }

    if health.focus > 0 {
        return Text::new(format!("+{}", health.focus), FontFace::Normal)
            .color(20, 201, 22, 255)
            .offset(39, 85);
    }

    return Text::new("-".to_string(), FontFace::Normal).offset(39, 85);
}

fn get_sprites_actor(a: &Actor, texture_map: &TextureMap) -> Sprites {
    let mut visual_elements = vec![];

    if a.active {
        visual_elements.push(format!("team_{}_active", a.team.1));
    } else {
        visual_elements.push(format!("team_{}_inactive", a.team.1));
    }

    for l in a.look() {
        visual_elements.push(l);
    }

    if let Some((action, _)) = &a.pending_action {
        visual_elements.push(map_action_to_sprite(action));
    }

    let sprites = visual_elements
        .iter()
        .filter_map(|key| texture_map.get(key))
        .cloned()
        .collect();

    Sprites::new(sprites)
}

fn get_sprites_items(item: &Item, texture_map: &TextureMap) -> Sprites {
    let sprites = item.look
        .iter()
        .filter_map(|(_, key)| texture_map.get(key))
        .cloned()
        .collect();

    Sprites::new(sprites)
}
