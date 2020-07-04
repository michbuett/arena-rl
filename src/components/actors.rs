use specs::prelude::*;

use crate::components::*;
use crate::core::{Effect, GameObject};

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

    if let Some(Position(curr_pos)) = positions.get(entity) {
        updater.insert(
            entity,
            MovementAnimation {
                start: Instant::now(),
                duration: Duration::from_millis(250),
                loops: 1,
                from: *curr_pos,
                to: get_target_position(&obj),
            },
        );
    } else {
        updater.insert(entity, Position(get_target_position(&obj)));
    }

    if let Some(txt) = get_txt(&obj) {
        updater.insert(entity, txt);
    }

    if let Some(sprites) = get_sprites(&obj) {
        updater.insert(entity, sprites);
    }

    updater.insert(entity, GameObjectCmp(obj));
}

fn get_target_position(obj: &GameObject) -> WorldPos {
    match obj {
        GameObject::Actor(a) => a.pos,
        GameObject::Item(pos, _) => *pos,
    }
}

fn get_sprites(obj: &GameObject) -> Option<Sprites> {
    match obj {
        GameObject::Actor(a) => {
            let mut sprites: Vec<Sprite> = vec![];
            let team_num = a.team.1 as i32;

            if a.active {
                sprites.push(Sprite {
                    texture: "teams".to_string(),
                    region: (0, 0, 128, 128),
                    offset: (0, 0),
                });
            }

            sprites.push(Sprite {
                texture: "teams".to_string(),
                region: (team_num * 128, 0, 128, 128),
                offset: (0, 0),
            });

            for key in a.look().iter() {
                sprites.push(map_sprite(key));
            }

            Some(Sprites(sprites))
        }

        GameObject::Item(_, item) => Some(Sprites(
            item.look.iter().map(|key| map_sprite(key)).collect(),
        )),
    }
}

fn map_sprite(s: &str) -> Sprite {
    // dbg!(s);
    match s {
        "player" => Sprite {
            texture: "player".to_string(),
            region: (256, 0, 128, 128),
            offset: (0, -32),
        },

        "enemy" => Sprite {
            texture: "enemy".to_string(),
            region: (0, 0, 128, 128),
            offset: (0, -32),
        },

        _ => Sprite {
            texture: s.to_string(),
            region: (0, 0, 128, 128),
            offset: (0, -32),
        },
    }
}

fn get_txt(obj: &GameObject) -> Option<Text> {
    match obj {
        GameObject::Actor(a) => {
            if a.has_effect(&Effect::Dying()) {
                return Some(
                    Text::new("Dying\nbreaths...".to_string(), "normal")
                        .background(252, 134, 31, 200)
                        .padding(5)
                        .offset(10, 30),
                );
            }

            return Some(
                Text::new(format!("{}/{}", a.energy(), a.num_wounds(),), "normal").offset(39, 90),
            );
        }

        _ => None,
    }
}

trait ToSprites {
    fn to_sprites(&self) -> Sprites;
}
