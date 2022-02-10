use specs::prelude::{Builder, Entities, Entity, LazyUpdate, Read, ReadStorage, World};

use crate::components::{
    Align, FontFace, GameObjectCmp, ObstacleCmp, Position, Restriction,
    Sprites, Text, WorldPos, ZLayerFloor, ZLayerGameObject,
};
use crate::core::{Act, Action, Actor, Attr, GameObject, Item, TextureMap};

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
    let melee_def = a.attr(Attr::MeleeDefence).val();
    let range_def = a.attr(Attr::RangeDefence).val();

    if positions.get(entity).is_none() {
        updater.insert(entity, Position(a.pos));
    }

    if let Some(txt) = get_txt_actor(&a) {
        updater.insert(entity, txt);
    } else {
        updater.remove::<Text>(entity);
    }

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
        Action::Ambush(..) => "icon-action-Charge",
        Action::Done(..) => "icon-action-Done",
        _ => "icon-action-Unknown",
    }
    .to_string()
}

fn get_txt_actor(a: &Actor) -> Option<Text> {
    let unformated_text = if a.health.recieved_wounds > 0 {
        Some(
            Text::new(format!("{}", a.health.recieved_wounds), FontFace::Big)
                .color(200, 21, 22, 255),
        )
    } else if a.health.focus < 0 {
        Some(Text::new("-".to_string(), FontFace::Normal).color(200, 201, 22, 255))
    } else if a.health.focus > 0 {
        Some(Text::new("+".to_string(), FontFace::Normal).color(200, 201, 22, 255))
    } else {
        None
    };

    unformated_text.map(|t: Text| {
        t.align(Align::MidCenter)
            .background(200, 201, 202, 128)
            .padding(5)
            .offset(0, 16)
    })
}

fn get_sprites_actor(a: &Actor, texture_map: &TextureMap) -> Sprites {
    let mut visual_elements = vec![format!("bg_team_{}", a.team.1)];

    if a.active {
        visual_elements.push("bg_active".to_string());
    }

    for l in a.look() {
        visual_elements.push(l);
    }

    if let Some(Act { action, .. }) = &a.pending_action {
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
    let sprites = item
        .look
        .iter()
        .filter_map(|(_, key)| texture_map.get(key))
        .cloned()
        .collect();

    Sprites::new(sprites)
}
