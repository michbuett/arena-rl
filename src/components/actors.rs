use specs::prelude::*;

use crate::components::*;
use crate::core::{GameObject, Action, };

pub fn insert_game_object_components(obj: GameObject, world: &World) {
    let (updater, positions, entities): (Read<LazyUpdate>, ReadStorage<Position>, Entities) =
        world.system_data();
    let e = updater.create_entity(&entities).build();

    lazy_insert_components(e, obj, (updater, positions));
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
    data: (Read<'a, LazyUpdate>, ReadStorage<'a, Position>),
) {
    let (updater, positions) = data;

    if positions.get(entity).is_none() {
        updater.insert(entity, Position(get_target_position(&obj)));
    }

    if let Some(txt) = get_txt(&obj) {
        updater.insert(entity, txt);
    }

    updater.insert(entity, get_visual_components(&obj));
    updater.insert(entity, GameObjectCmp(obj));
}

fn get_target_position(obj: &GameObject) -> WorldPos {
    match obj {
        GameObject::Actor(a) => a.pos,
        GameObject::Item(pos, _) => *pos,
    }
}


fn map_action_to_sprite(a: &Action) -> String {
    // dbg!(s);
    match a {
        Action::MoveTo(_) => "icon-action-MoveTo",
        Action::MeleeAttack(..) => "icon-action-MeleeAttack",
        Action::Charge(..) => "icon-action-Charge",
        Action::Wait(..) => "icon-action-Wait",
        _ => "icon-action-Unknown",
    }.to_string()
}

fn get_txt(obj: &GameObject) -> Option<Text> {
    match obj {
        GameObject::Actor(a) => {
            let (pain, wounds) = a.health();
            return Some(
                Text::new(format!("{} - {}", pain, wounds), FontFace::Normal).offset(39, 95),
            );
        }

        _ => None,
    }
}

fn get_visual_components(obj: &GameObject) -> VisualCmp {
    match obj {
        GameObject::Actor(a) => {
            let mut visual_elements = vec!();
            
            if a.active {
                visual_elements.push(format!("team_{}_active", a.team.1));
            } else {
                visual_elements.push(format!("team_{}_inactive", a.team.1));
            }

            for l in a.look() {
                visual_elements.push(visual_element(l));
            }

            if let Some((action, _)) = &a.pending_action {
                visual_elements.push(map_action_to_sprite(action));
            }

            VisualCmp(Instant::now(), visual_elements)
        }
        
        GameObject::Item(_, item) => 
            VisualCmp(Instant::now(), item.look.iter().map(visual_element).collect()),
    }
}

fn visual_element((name, num): &(&str, u16)) -> String {
    format!("{}_{}", name, num)
}
